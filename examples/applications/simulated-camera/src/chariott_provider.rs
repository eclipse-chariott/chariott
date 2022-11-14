// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chariott_proto::{
    common::{
        discover_fulfillment::Service, fulfillment::Fulfillment as FulfillmentEnum, intent::Intent,
        DiscoverFulfillment, Fulfillment as FulfillmentMessage,
    },
    provider::{provider_service_server::ProviderService, FulfillRequest, FulfillResponse},
};
use tonic::{Request, Response, Status};
use url::Url;

use examples_common::chariott::{
    self,
    inspection::{fulfill, Entry},
    streaming::ProtoExt as _,
    value::Value,
};

pub type StreamingStore = chariott::streaming::StreamingStore<Value>;

const SCHEMA_VERSION_STREAMING: &str = "chariott.streaming.v1";
const SCHEMA_REFERENCE: &str = "grpc+proto";

pub struct ChariottProvider {
    url: Url,
    store: Arc<StreamingStore>,
}

impl ChariottProvider {
    pub fn new(url: Url, store: Arc<StreamingStore>) -> Self {
        Self { url, store }
    }
}

lazy_static::lazy_static! {
    static ref CAMERA_SCHEMA: Vec<Entry> = vec![
        property("camera.2fpm", 2),
        property("camera.6fpm", 6),
        property("camera.12fpm", 12),
    ];
}

fn property(path: &str, fpm: i32) -> Entry {
    Entry::new(
        path,
        [
            ("member_type", "property".into()),
            ("type", "blob".into()),
            ("frames_per_minute", fpm.into()),
            ("write", Value::FALSE),
            ("watch", Value::TRUE),
        ],
    )
}

#[async_trait]
impl ProviderService for ChariottProvider {
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
            Intent::Discover(_) => FulfillmentEnum::Discover(DiscoverFulfillment {
                services: vec![Service {
                    url: self.url.to_string(),
                    schema_kind: SCHEMA_REFERENCE.to_owned(),
                    schema_reference: SCHEMA_VERSION_STREAMING.to_owned(),
                    metadata: HashMap::new(),
                }],
            }),
            Intent::Inspect(inspect) => fulfill(inspect.query, &*CAMERA_SCHEMA),
            Intent::Subscribe(subscribe) => self.store.subscribe(subscribe)?,
            Intent::Read(read) => self.store.read(read),
            _ => Err(Status::not_found(""))?,
        };

        Ok(Response::new(FulfillResponse {
            fulfillment: Some(FulfillmentMessage { fulfillment: Some(response) }),
        }))
    }
}
