// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use chariott_common::error::{Error, ResultExt};
use prost::Message;

use crate::chariott::{proto::common as common_proto, value::Value};

use super::proto::detection as detection_proto;

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

impl TryFrom<common_proto::InvokeIntent> for DetectRequest {
    type Error = Error;

    fn try_from(intent: common_proto::InvokeIntent) -> Result<Self, Self::Error> {
        if intent.args.len() != 1 || intent.command != "detect" {
            return Err(Error::new("No command found which accepts the invocation arguments."));
        }

        let value: Value =
            intent.args[0].clone().try_into().map_err(|_| Error::new("Could not parse value."))?;

        let (type_url, value) =
            value.into_any().map_err(|_| Error::new("Argument was not of type 'Any'."))?;

        if type_url == "examples.detection.v1.DetectRequest" {
            detection_proto::DetectRequest::decode(&*value)
                .map_err_with("DetectRequest decoding failed.")
                .and_then(|detection_proto::DetectRequest { blob }| {
                    blob.ok_or_else(|| Error::new("No blob was present."))
                })
                .map(|common_proto::Blob { bytes, .. }| DetectRequest(bytes))
        } else {
            Err(Error::new("Argument was not of type 'examples.detection.v1.DetectRequest'."))
        }
    }
}

impl From<DetectResponse> for common_proto::InvokeFulfillment {
    fn from(value: DetectResponse) -> Self {
        let entries = value
            .0
            .into_iter()
            .map(|o| detection_proto::DetectEntry {
                object: o.object.into(),
                confidence: o.confidence,
            })
            .collect();

        common_proto::InvokeFulfillment {
            r#return: Some(
                Value::new_any(
                    "examples.detection.v1.DetectResponse".to_string(),
                    detection_proto::DetectResponse { entries }.encode_to_vec(),
                )
                .into(),
            ),
        }
    }
}
