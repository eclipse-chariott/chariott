// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use chariott::chariott_grpc::ChariottServer;
use chariott::registry::Registry;
use chariott::IntentBroker;
use chariott_common::proto::runtime::chariott_service_server::ChariottServiceServer;
use chariott_common::proto::streaming::channel_service_server::ChannelServiceServer;
use chariott_common::shutdown::RouterExt as _;
use tonic::transport::Server;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[cfg(build = "debug")]
pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor");

#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use chariott::registry::{CompositeRegistryObserver, RegistryChangeEvents};

    let collector = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .finish();

    collector.init();

    let ess = RegistryChangeEvents::new();
    let broker = IntentBroker::new("http://localhost:4243".parse().unwrap(), ess.clone());
    let registry = Registry::new(CompositeRegistryObserver::new(broker.clone(), ess.clone()));

    #[cfg(build = "debug")]
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()?;

    tracing::info!("starting grpc services");
    let addr = "0.0.0.0:4243".parse().unwrap();
    tracing::info!("chariott listening on {addr}");

    let router = Server::builder()
        .add_service(ChariottServiceServer::new(ChariottServer::new(registry, broker)))
        .add_service(ChannelServiceServer::new(ess));

    #[cfg(build = "debug")]
    let router = router.add_service(reflection_service);

    router.serve_with_ctrl_c_shutdown(addr).await?;

    Ok(())
}
