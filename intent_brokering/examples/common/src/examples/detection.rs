// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use intent_brokering_common::error::{Error, ResultExt};
use intent_brokering_proto::common::{Blob, InvokeFulfillment, InvokeIntent};
use prost::Message;

use crate::intent_brokering::value::Value;

use super::proto::detection::{
    DetectEntry, DetectRequest as DetectRequestMessage, DetectResponse as DetectResponseMessage,
};

pub struct DetectRequest(Vec<u8>);

impl From<DetectRequest> for Vec<u8> {
    fn from(detect_request: DetectRequest) -> Self {
        detect_request.0
    }
}

pub struct DetectResponse(Vec<DetectionObject>);

impl DetectResponse {
    pub fn new(detection_objects: Vec<DetectionObject>) -> Self {
        Self(detection_objects)
    }
}

#[derive(Clone)]
pub struct DetectionObject {
    object: Box<str>,
    confidence: f64,
}

impl DetectionObject {
    pub fn new(object: impl Into<Box<str>>, confidence: f64) -> Self {
        Self { object: object.into(), confidence }
    }
}

impl TryFrom<InvokeIntent> for DetectRequest {
    type Error = Error;

    fn try_from(intent: InvokeIntent) -> Result<Self, Self::Error> {
        if intent.args.len() != 1 || intent.command != "detect" {
            return Err(Error::new("No command found which accepts the invocation arguments."));
        }

        let value: Value =
            intent.args[0].clone().try_into().map_err(|_| Error::new("Could not parse value."))?;

        let (type_url, value) =
            value.into_any().map_err(|_| Error::new("Argument was not of type 'Any'."))?;

        if type_url == "examples.detection.v1.DetectRequest" {
            DetectRequestMessage::decode(&*value)
                .map_err_with("DetectRequest decoding failed.")
                .and_then(|DetectRequestMessage { blob }| {
                    blob.ok_or_else(|| Error::new("No blob was present."))
                })
                .map(|Blob { bytes, .. }| DetectRequest(bytes))
        } else {
            Err(Error::new("Argument was not of type 'examples.detection.v1.DetectRequest'."))
        }
    }
}

impl From<DetectResponse> for InvokeFulfillment {
    fn from(value: DetectResponse) -> Self {
        let entries = value
            .0
            .into_iter()
            .map(|o| DetectEntry { object: o.object.into(), confidence: o.confidence })
            .collect();

        InvokeFulfillment {
            r#return: Some(
                Value::new_any(
                    "examples.detection.v1.DetectResponse".to_string(),
                    DetectResponseMessage { entries }.encode_to_vec(),
                )
                .into(),
            ),
        }
    }
}
