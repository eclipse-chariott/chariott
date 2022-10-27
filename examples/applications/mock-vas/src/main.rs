// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

mod chariott_provider;
mod communication;
mod simulation;

use chariott_common::error::Error;
use examples_common::chariott::{
    self,
    proto::runtime_api::{
        intent_registration::Intent, intent_service_registration::ExecutionLocality,
    },
};

chariott::provider::main!(wain);

async fn wain() -> Result<(), Error> {
    let (url, socket_address) = chariott::provider::register(
        "sdv.mock-vas",
        "0.0.1",
        "sdv.vdt",
        [Intent::Discover, Intent::Invoke, Intent::Inspect, Intent::Subscribe, Intent::Read],
        "VAS_URL",
        "http://0.0.0.0:50051",
        ExecutionLocality::Local,
    )
    .await?;

    tracing::info!("Application listening on: {url}");

    communication::serve(url, socket_address).await
}
