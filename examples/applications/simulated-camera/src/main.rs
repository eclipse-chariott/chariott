// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

mod camera;
mod chariott_provider;
mod communication;

use chariott_common::error::Error;
use examples_common::chariott;
use examples_common::chariott::proto::runtime_api::intent_registration::Intent;
use examples_common::chariott::proto::runtime_api::intent_service_registration::ExecutionLocality;

chariott::provider::main!(wain);

async fn wain() -> Result<(), Error> {
    let (url, socket_address) = chariott::provider::register(
        "sdv.cabin.camera",
        "0.0.1",
        "sdv.camera.simulated",
        [Intent::Discover, Intent::Subscribe, Intent::Inspect, Intent::Read],
        "SIMULATED_CAMERA_URL",
        "http://0.0.0.0:50066",
        ExecutionLocality::Local,
    )
    .await?;

    tracing::info!("Application application listening: {url}");

    communication::serve(url, socket_address).await
}
