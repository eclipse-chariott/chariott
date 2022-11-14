// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

mod chariott_provider;
mod detection;

use chariott_common::error::Error;
use chariott_common::shutdown::RouterExt as _;
use chariott_proto::{
    provider::provider_service_server::ProviderServiceServer,
    runtime::{intent_registration::Intent, intent_service_registration::ExecutionLocality},
};
use examples_common::chariott;
use tonic::transport::Server;

use crate::chariott_provider::ChariottProvider;

chariott::provider::main!(wain);

async fn wain() -> Result<(), Error> {
    let (url, socket_address) = chariott::provider::register(
        "sdv.local-detection",
        "0.0.1",
        "sdv.detection",
        [Intent::Inspect, Intent::Invoke],
        "LOCAL_DETECTION_URL",
        "http://0.0.0.0:50061",
        ExecutionLocality::Local,
    )
    .await?;

    tracing::info!("Application application listening: {url}");

    Server::builder()
        .add_service(ProviderServiceServer::new(ChariottProvider::new()))
        .serve_with_ctrl_c_shutdown(socket_address)
        .await
}
