// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::sync::Arc;

use async_trait::async_trait;
use examples_common::chariott::{self, streaming::ProtoExt as _};
use tonic::{Request, Response, Status};

use url::Url;

use examples_common::chariott::proto::{
    self,
    common::{
        fulfillment, intent, value::Value, DiscoverFulfillment, Fulfillment, WriteFulfillment,
        WriteIntent,
    },
    provider::*,
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
impl provider_service_server::ProviderService for ChariottProvider {
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
            intent::Intent::Read(intent) => Ok(self.streaming_store.read(intent)),
            intent::Intent::Write(intent) => {
                self.write(intent).map(fulfillment::Fulfillment::Write)
            }
            intent::Intent::Subscribe(intent) => self.streaming_store.subscribe(intent),
            intent::Intent::Discover(_intent) => {
                use proto::common::discover_fulfillment::Service;
                use std::collections::HashMap;

                Ok(fulfillment::Fulfillment::Discover(DiscoverFulfillment {
                    services: vec![Service {
                        url: self.url.to_string(),
                        schema_kind: "grpc+proto".to_owned(),
                        schema_reference: "chariott.streaming.v1".to_owned(),
                        metadata: HashMap::new(),
                    }],
                }))
            }
            _ => Err(Status::unknown("Unsupported or unknown intent."))?,
        };

        fulfillment.map(|f| {
            Response::new(FulfillResponse {
                fulfillment: Some(Fulfillment { fulfillment: Some(f) }),
            })
        })
    }
}
