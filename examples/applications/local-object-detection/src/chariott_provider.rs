// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use async_trait::async_trait;
use tonic::{Request, Response, Status};
use tracing::error;

use crate::detection::DetectionLogic;

use examples_common::{
    chariott::{
        inspection::{fulfill, Entry},
        proto::*,
    },
    examples::detection::DetectRequest,
};

pub struct ChariottProvider {
    internal_logic: DetectionLogic,
}

impl ChariottProvider {
    pub fn new() -> Self {
        let internal_logic = DetectionLogic::new();
        Self { internal_logic }
    }
}

lazy_static::lazy_static! {
    static ref INSPECT_FULFILLMENT_SCHEMA: Vec<Entry> = vec![
        Entry::new("detect", [
            ("member_type", "command"),
            ("request", "examples.detection.v1.DetectRequest"),
            ("response", "examples.detection.v1.DetectResponse"),
        ])
    ];
}

#[async_trait]
impl provider::provider_service_server::ProviderService for ChariottProvider {
    async fn fulfill(
        &self,
        request: Request<provider::FulfillRequest>,
    ) -> Result<Response<provider::FulfillResponse>, Status> {
        let response = match request
            .into_inner()
            .intent
            .and_then(|i| i.intent)
            .ok_or_else(|| Status::invalid_argument("Intent must be specified"))?
        {
            common::intent::Intent::Inspect(inspect) => {
                fulfill(inspect.query, &*INSPECT_FULFILLMENT_SCHEMA)
            }
            common::intent::Intent::Invoke(intent) => {
                let arg = DetectRequest::try_from(intent)
                    .map_err(|e| Status::invalid_argument(e.to_string()))?;

                let result = self.internal_logic.detect_local(arg).map_err(|e| {
                    error!("Error when running detection: '{e:?}'.");
                    Status::unknown(format!("Error when invoking function: '{}'", e))
                })?;

                common::fulfillment::Fulfillment::Invoke(result.into())
            }
            _ => Err(Status::not_found(""))?,
        };

        Ok(Response::new(provider::FulfillResponse {
            fulfillment: Some(common::Fulfillment { fulfillment: Some(response) }),
        }))
    }
}
