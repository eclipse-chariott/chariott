// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod detection;
mod intent_provider;

use examples_common::intent_brokering;
use intent_brokering_common::error::Error;
use intent_brokering_common::shutdown::RouterExt as _;
use intent_brokering_proto::{
    provider::provider_service_server::ProviderServiceServer,
    runtime::{intent_registration::Intent, intent_service_registration::ExecutionLocality},
};
use tonic::transport::Server;

use crate::intent_provider::IntentProvider;

intent_brokering::provider::main!(wain);

async fn wain() -> Result<(), Error> {
    let (url, socket_address) = intent_brokering::provider::register(
        "sdv.local-detection",
        "0.0.1",
        "sdv.detection",
        [Intent::Inspect, Intent::Invoke],
        "LOCAL_DETECTION_URL",
        "http://0.0.0.0:50061", // DevSkim: ignore DS137138
        ExecutionLocality::Local,
    )
    .await?;

    tracing::info!("Application application listening: {url}");

    Server::builder()
        .add_service(ProviderServiceServer::new(IntentProvider::new()))
        .serve_with_ctrl_c_shutdown(socket_address)
        .await
}
