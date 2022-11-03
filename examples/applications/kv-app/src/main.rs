// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

mod chariott_provider;

use std::sync::Arc;

use chariott_common::error::Error;
use chariott_common::shutdown::RouterExt as _;
use examples_common::chariott::proto::runtime_api::intent_service_registration::ExecutionLocality;
use examples_common::chariott::proto::{
    provider::provider_service_server::ProviderServiceServer,
    streaming::channel_service_server::ChannelServiceServer,
};
use examples_common::chariott::{self, proto::runtime_api::intent_registration::Intent};
use tonic::transport::Server;

use crate::chariott_provider::{ChariottProvider, StreamingStore};

chariott::provider::main!(wain);

async fn wain() -> Result<(), Error> {
    let (url, socket_address) = chariott::provider::register(
        "sdv.key-value-store",
        "0.0.1",
        "sdv.kvs",
        [Intent::Read, Intent::Write, Intent::Subscribe, Intent::Discover],
        "KVS_URL",
        "http://0.0.0.0:50064",
        ExecutionLocality::Local,
    )
    .await?;

    tracing::info!("Application listening on: {url}");

    let streaming_store = Arc::new(StreamingStore::new());
    let provider = Arc::new(ChariottProvider::new(url.clone(), Arc::clone(&streaming_store)));

    Server::builder()
        .add_service(ProviderServiceServer::from_arc(Arc::clone(&provider)))
        .add_service(ChannelServiceServer::new(streaming_store.ess().clone()))
        .serve_with_ctrl_c_shutdown(socket_address)
        .await
}
