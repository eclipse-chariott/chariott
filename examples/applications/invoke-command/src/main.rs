// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod chariott_provider;

use std::sync::Arc;

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

    let provider = Arc::new(ChariottProvider::new(url.clone()));

    Server::builder()
        .add_service(ProviderServiceServer::from_arc(Arc::clone(&provider)))
        .serve_with_ctrl_c_shutdown(socket_address)
        .await
}
