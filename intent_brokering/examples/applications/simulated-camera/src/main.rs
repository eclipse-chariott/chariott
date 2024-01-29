// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod camera;
mod communication;
mod intent_provider;

use examples_common::intent_brokering;
use intent_brokering_common::error::Error;
use intent_brokering_proto::runtime::{
    intent_registration::Intent, intent_service_registration::ExecutionLocality,
};

intent_brokering::provider::main!(wain);

async fn wain() -> Result<(), Error> {
    let (url, socket_address) = intent_brokering::provider::register(
        "sdv.cabin.camera",
        "0.0.1",
        "sdv.camera.simulated",
        [Intent::Discover, Intent::Subscribe, Intent::Inspect, Intent::Read],
        "SIMULATED_CAMERA_URL",
        "http://0.0.0.0:50066", // DevSkim: ignore DS137138
        ExecutionLocality::Local,
    )
    .await?;

    tracing::info!("Application application listening: {url}");

    communication::serve(url, socket_address).await
}
