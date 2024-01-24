// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod intent_provider;

use std::sync::Arc;

use examples_common::intent_brokering;
use intent_brokering_common::error::Error;
use intent_brokering_common::shutdown::RouterExt as _;
use intent_brokering_proto::{
    provider::provider_service_server::ProviderServiceServer,
    runtime::{intent_registration::Intent, intent_service_registration::ExecutionLocality},
    streaming::channel_service_server::ChannelServiceServer,
};
use tonic::transport::Server;

use crate::intent_provider::{IntentProvider, StreamingStore};

intent_brokering::provider::main!(wain);

async fn wain() -> Result<(), Error> {
    let (url, socket_address) = intent_brokering::provider::register(
        "sdv.key-value-store",
        "0.0.1",
        "sdv.kvs",
        [Intent::Read, Intent::Write, Intent::Subscribe, Intent::Discover],
        "KVS_URL",
        "http://0.0.0.0:50064", // DevSkim: ignore DS137138
        ExecutionLocality::Local,
    )
    .await?;

    tracing::info!("Application listening on: {url}");

    let streaming_store = Arc::new(StreamingStore::new());
    let provider = Arc::new(IntentProvider::new(url.clone(), Arc::clone(&streaming_store)));

    Server::builder()
        .add_service(ProviderServiceServer::from_arc(Arc::clone(&provider)))
        .add_service(ChannelServiceServer::new(streaming_store.ess().clone()))
        .serve_with_ctrl_c_shutdown(socket_address)
        .await
}
