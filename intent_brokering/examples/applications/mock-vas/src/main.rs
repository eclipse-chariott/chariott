// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod communication;
mod intent_provider;
mod simulation;

use examples_common::chariott;
use intent_brokering_common::error::Error;
use intent_brokering_proto::runtime::{
    intent_registration::Intent, intent_service_registration::ExecutionLocality,
};

chariott::provider::main!(wain);

async fn wain() -> Result<(), Error> {
    let (url, socket_address) = chariott::provider::register(
        "sdv.mock-vas",
        "0.0.1",
        "sdv.vdt",
        [Intent::Discover, Intent::Invoke, Intent::Inspect, Intent::Subscribe, Intent::Read],
        "VAS_URL",
        "http://0.0.0.0:50051", // DevSkim: ignore DS137138
        ExecutionLocality::Local,
    )
    .await?;

    tracing::info!("Application listening on: {url}");

    communication::serve(url, socket_address).await
}
