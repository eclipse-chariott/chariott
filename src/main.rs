// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use chariott::chariott_grpc::ChariottServer;
use chariott::registry::{self, Registry};
use chariott::IntentBroker;
use chariott_common::config::try_env;
use chariott_common::ext::OptionExt as _;
use chariott_common::proto::runtime::chariott_service_server::ChariottServiceServer;
use chariott_common::shutdown::{ctrl_c_cancellation, RouterExt as _};
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
    let collector = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .finish();

    collector.init();

    let broker = IntentBroker::new();

    let registry_config = try_env::<u64>("CHARIOTT_REGISTRY_TTL_SECS")
        .ok()?
        .map(Duration::from_secs)
        .map(|v| registry::Config::default().set_entry_ttl_unchecked(v))
        .unwrap_or_default();

    let registry_entry_ttl = registry_config.entry_ttl();
    let registry = Registry::new(broker.clone(), registry_config);

    #[cfg(build = "debug")]
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()?;

    tracing::info!("starting grpc services");
    let addr = "0.0.0.0:4243".parse().unwrap();
    tracing::info!("chariott listening on {addr}");

    let server = Arc::new(ChariottServer::new(registry, broker));
    let router =
        Server::builder().add_service(ChariottServiceServer::from_arc(Arc::clone(&server)));

    #[cfg(build = "debug")]
    let router = router.add_service(reflection_service);

    let error_cancellation_token = CancellationToken::new();
    let ctrl_c_cancellation_token = ctrl_c_cancellation();

    let prune_loop = {
        let ctrl_c_cancellation_token = ctrl_c_cancellation_token.clone();
        let error_cancellation_token = error_cancellation_token.child_token();

        async move {
            tracing::debug!("Prune loop running (TTL = {registry_entry_ttl:?}).");
            loop {
                let wakeup_deadline = server.registry_do(|reg| {
                    let now = Instant::now();
                    reg.prune(now).unwrap_or_else(|| now + registry_entry_ttl)
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
    };

    let router_serve = async {
        match router.serve_with_cancellation(addr, ctrl_c_cancellation_token).await {
            err @ Err(_) => {
                error_cancellation_token.cancel();
                err
            }
            res => res,
        }
    };

    let (router_serve_result, _) = tokio::join!(router_serve, prune_loop);

    router_serve_result?;

    Ok(())
}
