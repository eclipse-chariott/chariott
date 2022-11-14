// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use examples_common::chariott::{self, streaming::ProtoExt as _};
use tonic::{Request, Response, Status};

use url::Url;

use chariott_proto::{
    common::{
        discover_fulfillment::Service, value::Value, DiscoverFulfillment, FulfillmentEnum,
        FulfillmentMessage, IntentEnum, WriteFulfillment, WriteIntent,
    },
    provider::{provider_service_server::ProviderService, FulfillRequest, FulfillResponse},
};

pub type StreamingStore = chariott::streaming::StreamingStore<Value>;

pub struct ChariottProvider {
    url: Url,
    streaming_store: Arc<StreamingStore>,
}

impl ChariottProvider {
    pub fn new(url: Url, streaming_store: Arc<StreamingStore>) -> Self {
        Self { url, streaming_store }
    }

    fn write(&self, intent: WriteIntent) -> Result<WriteFulfillment, Status> {
        let key = intent.key.into();
        let value = intent
            .value
            .and_then(|v| v.value)
            .ok_or_else(|| Status::unknown("Value must be specified."))?;
        self.streaming_store.set(key, value);
        Ok(WriteFulfillment {})
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
            IntentEnum::Read(intent) => Ok(self.streaming_store.read(intent)),
            IntentEnum::Write(intent) => self.write(intent).map(FulfillmentEnum::Write),
            IntentEnum::Subscribe(intent) => self.streaming_store.subscribe(intent),
            IntentEnum::Discover(_intent) => Ok(FulfillmentEnum::Discover(DiscoverFulfillment {
                services: vec![Service {
                    url: self.url.to_string(),
                    schema_kind: "grpc+proto".to_owned(),
                    schema_reference: "chariott.streaming.v1".to_owned(),
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
