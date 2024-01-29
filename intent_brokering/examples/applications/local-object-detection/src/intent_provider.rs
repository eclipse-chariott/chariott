// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use async_trait::async_trait;
use intent_brokering_proto::{
    common::{FulfillmentEnum, FulfillmentMessage, IntentEnum},
    provider::{provider_service_server::ProviderService, FulfillRequest, FulfillResponse},
};
use tonic::{Request, Response, Status};
use tracing::error;

use crate::detection::DetectionLogic;

use examples_common::{
    examples::detection::DetectRequest,
    intent_brokering::inspection::{fulfill, Entry},
};

pub struct IntentProvider {
    internal_logic: DetectionLogic,
}

impl IntentProvider {
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
impl ProviderService for IntentProvider {
    async fn fulfill(
        &self,
        request: Request<FulfillRequest>,
    ) -> Result<Response<FulfillResponse>, Status> {
        let response = match request
            .into_inner()
            .intent
            .and_then(|i| i.intent)
            .ok_or_else(|| Status::invalid_argument("Intent must be specified"))?
        {
            IntentEnum::Inspect(inspect) => fulfill(inspect.query, &*INSPECT_FULFILLMENT_SCHEMA),
            IntentEnum::Invoke(intent) => {
                let arg = DetectRequest::try_from(intent)
                    .map_err(|e| Status::invalid_argument(e.to_string()))?;

                let result = self.internal_logic.detect_local(arg).map_err(|e| {
                    error!("Error when running detection: '{e:?}'.");
                    Status::unknown(format!("Error when invoking function: '{}'", e))
                })?;

                FulfillmentEnum::Invoke(result.into())
            }
            _ => Err(Status::not_found(""))?,
        };

        Ok(Response::new(FulfillResponse {
            fulfillment: Some(FulfillmentMessage { fulfillment: Some(response) }),
        }))
    }
}
