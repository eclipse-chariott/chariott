// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{collections::HashMap};

use async_trait::async_trait;
use tonic::{Request, Response, Status};

use url::Url;

use chariott_proto::{
    common::{
        discover_fulfillment::Service, DiscoverFulfillment, FulfillmentEnum,
        FulfillmentMessage, IntentEnum
    },
    provider::{provider_service_server::ProviderService, FulfillRequest, FulfillResponse},
};

pub struct ChariottProvider {
    url: Url
}

impl ChariottProvider {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

#[async_trait]
impl ProviderService for ChariottProvider {
    async fn fulfill(
        &self,
        request: Request<FulfillRequest>,
    ) -> Result<Response<FulfillResponse>, Status> {
        let fulfillment = match request
            .into_inner()
            .intent
            .and_then(|i| i.intent)
            .ok_or_else(|| Status::invalid_argument("Intent must be specified."))?
        {
            IntentEnum::Discover(_intent) => Ok(FulfillmentEnum::Discover(DiscoverFulfillment {
                services: vec![Service {
                    url: self.url.to_string(),
                    schema_kind: "grpc+proto".to_owned(),
                    schema_reference: "example.provider.v1".to_owned(),
                    metadata: HashMap::new(),
                }],
            })),
            _ => Err(Status::unknown("Unsupported or unknown intent."))?,
        };

        fulfillment.map(|f| {
            Response::new(FulfillResponse {
                fulfillment: Some(FulfillmentMessage { fulfillment: Some(f) }),
            })
        })
    }
}
