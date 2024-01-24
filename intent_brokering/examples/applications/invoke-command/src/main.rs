// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod intent_provider;

use std::sync::Arc;

use examples_common::chariott;
use intent_brokering_common::error::Error;
use intent_brokering_common::shutdown::RouterExt as _;
use intent_brokering_proto::{
    provider::provider_service_server::ProviderServiceServer,
    runtime::{intent_registration::Intent, intent_service_registration::ExecutionLocality},
};
use tonic::transport::Server;

use crate::intent_provider::IntentProvider;

chariott::provider::main!(wain);

async fn wain() -> Result<(), Error> {
    let (url, socket_address) = chariott::provider::register(
        "sdv.invoke.controller",
        "0.0.1",
        "sdv.invoke.controller",
        [Intent::Discover, Intent::Invoke],
        "MASSAGE_URL",
        "http://0.0.0.0:50064", // DevSkim: ignore DS137138
        ExecutionLocality::Local,
    )
    .await?;

    tracing::info!("Application listening on: {url}");

    let provider = Arc::new(IntentProvider::new(url.clone()));

    Server::builder()
        .add_service(ProviderServiceServer::from_arc(Arc::clone(&provider)))
        .serve_with_ctrl_c_shutdown(socket_address)
        .await
}
