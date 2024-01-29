// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use anyhow::{anyhow, Error};
use async_stream::try_stream;
use examples_common::{
    examples::proto::detection::{DetectRequest, DetectResponse},
    intent_brokering::{
        api::{GrpcIntentBrokering, IntentBrokering, IntentBrokeringExt as _},
        value::Value,
    },
};
use futures::{stream::BoxStream, TryStreamExt};
use intent_brokering_proto::common::Blob;
use tokio_stream::StreamExt;
use tracing::{info, warn};

use crate::{DogModeState, DOG_MODE_STATUS_ID, KEY_VALUE_STORE_NAMESPACE};

pub(crate) async fn stream_dog_mode_status(
    mut intent_broker: GrpcIntentBrokering,
    state: &mut DogModeState,
) -> Result<BoxStream<'static, Result<bool, Error>>, Error> {
    if let ok @ Ok(_) = detect_dog(intent_broker.clone()).await {
        info!("Using automated dog detection.");

        // The dog mode logic application is responsible for updating the
        // dog mode state in the key-value store.
        state.write_dog_mode_status = true;

        ok
    } else {
        warn!("Automatic dog detection failed. Falling back to using an external application to turn on the dog mode.");

        Ok(Box::pin(
            intent_broker
                .listen(KEY_VALUE_STORE_NAMESPACE, [DOG_MODE_STATUS_ID.into()])
                .await?
                .map_err(|e| e.into())
                .map(|r| {
                    r.and_then(|e| {
                        e.data.to_bool().map_err(|_| anyhow!("Result was not of type 'Bool'."))
                    })
                }),
        ))
    }
}

async fn detect_dog(
    mut intent_broker: GrpcIntentBrokering,
) -> Result<BoxStream<'static, Result<bool, Error>>, Error> {
    const CAMERA_NAMESPACE: &str = "sdv.camera.simulated";
    const FRAMES_METADATA_KEY: &str = "frames_per_minute";
    const OBJECT_DETECTION_NAMESPACE: &str = "sdv.detection";
    const DETECT_COMMAND_NAME: &str = "detect";
    const DOG_CATEGORY_NAME: &str = "dog";
    const SYSTEM_REGISTRY_NAMESPACE: &str = "system.registry";

    /// Asserts whether any intents are registered for the specified
    /// namespace. If there are intents, we assume that those are the
    /// supported intent.
    async fn ensure_vehicle_functionality(
        intent_broker: &mut impl IntentBrokering,
        namespace: &str,
    ) -> Result<(), Error> {
        if intent_broker.inspect(SYSTEM_REGISTRY_NAMESPACE, namespace).await?.is_empty() {
            Err(anyhow!("Vehicle does not have registrations for namespace '{namespace}'."))
        } else {
            Ok(())
        }
    }

    ensure_vehicle_functionality(&mut intent_broker, CAMERA_NAMESPACE).await?;
    ensure_vehicle_functionality(&mut intent_broker, OBJECT_DETECTION_NAMESPACE).await?;

    // Stream images from the camera at the highest frame rate.

    let (subscription_key, frames) = intent_broker
        .inspect(CAMERA_NAMESPACE, "**")
        .await?
        .into_iter()
        .filter_map(|entry| {
            entry
                .get(FRAMES_METADATA_KEY)
                .and_then(|frames| frames.to_i32().ok())
                .map(|frames| (entry.path().into(), frames))
        })
        .max_by_key(|(_, frames)| *frames)
        .ok_or_else(|| anyhow!("Could not find an entry with maximum framerate."))?;

    // Stream the images and run object detection on them.

    info!("Streaming with frame rate of {frames}fpm.");

    let images = intent_broker.listen(CAMERA_NAMESPACE, [subscription_key]).await?;

    let dog_mode_state_stream = try_stream! {
        for await image in images {
            yield image_contains_dog(&mut intent_broker, image?.data).await?;
        }
    };

    async fn image_contains_dog(
        intent_broker: &mut impl IntentBrokering,
        image: Value,
    ) -> Result<bool, Error> {
        use prost::Message;

        let (media_type, bytes) = image
            .into_blob()
            .map_err(|_| anyhow!("Unexpected image return type (expected: 'Blob')."))?;

        let detect_request = DetectRequest { blob: Some(Blob { media_type, bytes }) };

        let mut detect_request_bytes = vec![];
        detect_request.encode(&mut detect_request_bytes)?;

        let detection_result = intent_broker
            .invoke(
                OBJECT_DETECTION_NAMESPACE,
                DETECT_COMMAND_NAME,
                [Value::new_any(
                    "examples.detection.v1.DetectRequest".to_owned(),
                    detect_request_bytes,
                )],
            )
            .await?;

        let (_, value) = detection_result
            .into_any()
            .map_err(|_| anyhow!("Detection response was not of type 'Any'."))?;

        let detected_categories: DetectResponse = Message::decode(&value[..])?;
        Ok(detected_categories.entries.iter().any(|e| e.object == DOG_CATEGORY_NAME))
    }

    Ok(Box::pin(dog_mode_state_stream))
}
