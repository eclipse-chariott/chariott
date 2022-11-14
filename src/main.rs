// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use chariott::chariott_grpc::ChariottServer;
use chariott::registry::{self, Registry};
use chariott::streaming::StreamingEss;
use chariott::IntentBroker;
use chariott_common::config::{env, try_env};
use chariott_common::ext::OptionExt as _;
use chariott_common::shutdown::{ctrl_c_cancellation, RouterExt as _};
use chariott_proto::{
    runtime::chariott_service_server::ChariottServiceServer,
    streaming::channel_service_server::ChannelServiceServer,
};
use registry::Composite;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::{select, time::sleep_until, time::Instant as TokioInstant};
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[cfg(build = "debug")]
pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor");

#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    const EXTERNAL_HOST_NAME_ENV: &str = "EXTERNAL_HOST_NAME";
    const PORT: u16 = 4243;

    let collector = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .finish();

    collector.init();

    let streaming_ess = StreamingEss::new();
    let broker = IntentBroker::new(
        format!(
            "http://{}:{}",
            env::<String>(EXTERNAL_HOST_NAME_ENV).as_deref().unwrap_or("localhost"),
            PORT
        )
        .parse()
        .unwrap(),
        streaming_ess.clone(),
    );

    let registry_config = try_env::<u64>("CHARIOTT_REGISTRY_TTL_SECS")
        .ok()?
        .map(Duration::from_secs)
        .map(|v| registry::Config::default().set_entry_ttl_bounded(v))
        .unwrap_or_default();

    tracing::debug!("Registry entry TTL = {} (seconds)", registry_config.entry_ttl().as_secs_f64());

    let registry =
        Registry::new(Composite::new(broker.clone(), streaming_ess.clone()), registry_config);

    #[cfg(build = "debug")]
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()?;

    tracing::info!("starting grpc services");
    let addr = format!("0.0.0.0:{PORT}").parse().unwrap();
    tracing::info!("chariott listening on {addr}");

    let server = Arc::new(ChariottServer::new(registry, broker));
    let router = Server::builder()
        .add_service(ChariottServiceServer::from_arc(Arc::clone(&server)))
        .add_service(ChannelServiceServer::new(streaming_ess));

    #[cfg(build = "debug")]
    let router = router.add_service(reflection_service);

    let error_cancellation_token = CancellationToken::new();
    let ctrl_c_cancellation_token = ctrl_c_cancellation();

    let registry_prune_loop = registry_prune_loop(
        server,
        ctrl_c_cancellation_token.clone(),
        error_cancellation_token.child_token(),
    );

    let router_serve = async {
        match router.serve_with_cancellation(addr, ctrl_c_cancellation_token).await {
            err @ Err(_) => {
                error_cancellation_token.cancel();
                err
            }
            res => res,
        }
    };

    let (router_serve_result, _) = tokio::join!(router_serve, registry_prune_loop);

    router_serve_result?;

    Ok(())
}

async fn registry_prune_loop(
    server: Arc<ChariottServer<Composite<IntentBroker, StreamingEss>>>,
    ctrl_c_cancellation_token: CancellationToken,
    error_cancellation_token: CancellationToken,
) {
    tracing::debug!("Prune loop running.");
    loop {
        let (_, wakeup_deadline) = server.registry_do(|reg| {
            let now = Instant::now();
            reg.prune(now)
        });
        select! {
            _ = sleep_until(TokioInstant::from_std(wakeup_deadline)) => {}
            _ = error_cancellation_token.cancelled() => {
                tracing::debug!("Prune loop aborting due to server error.");
                break;
            }
            _ = ctrl_c_cancellation_token.cancelled() => {
                tracing::debug!("Prune loop aborting due to cancellation.");
                break;
            }
        }
    }
}
