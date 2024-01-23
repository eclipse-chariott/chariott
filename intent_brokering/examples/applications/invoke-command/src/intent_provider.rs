// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use async_trait::async_trait;
use tonic::{Request, Response, Status};

use url::Url;

use intent_brokering_proto::{
    common::{
        discover_fulfillment::Service, value::Value, DiscoverFulfillment, FulfillmentEnum,
        FulfillmentMessage, IntentEnum, InvokeFulfillment, InvokeIntent, ValueMessage,
    },
    provider::{provider_service_server::ProviderService, FulfillRequest, FulfillResponse},
};

pub struct IntentProvider {
    url: Url,
}

impl IntentProvider {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    // Simple function that parses incoming json and then prints it.
    fn parse_and_print_json(json_string: String) -> Result<String, Status> {
        let json_val: serde_json::Value = serde_json::from_str(&json_string)
            .map_err(|_| Status::unknown("failed to parse json."))?;

        println!("{}", json_val);

        Ok("Successfully processed json".to_string())
    }

    // Simple function that executes on a given invoke intent.
    fn invoke(&self, intent: InvokeIntent) -> Result<InvokeFulfillment, Status> {
        let command = intent.command;

        let result = match command.as_str() {
            "parse_and_print_json" => {
                let value = intent.args[0].value.clone();

                let json_string = match value {
                    Some(Value::String(s)) => s,
                    _ => Err(Status::unknown("unexpected data type."))?,
                };

                let res = Self::parse_and_print_json(json_string).unwrap();
                let ret = ValueMessage { value: Some(Value::String(res)) };
                Ok(InvokeFulfillment { r#return: Some(ret) })
            }
            _ => Err(Status::unknown(format!("No command found for {}", command))),
        };

        result
    }
}

#[async_trait]
impl ProviderService for IntentProvider {
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
                    schema_reference: "invoke.controller.v1".to_owned(),
                    metadata: HashMap::new(),
                }],
            })),
            IntentEnum::Invoke(intent) => self.invoke(intent).map(FulfillmentEnum::Invoke),
            _ => Err(Status::unknown("Unsupported or unknown intent."))?,
        };

        fulfillment.map(|f| {
            Response::new(FulfillResponse {
                fulfillment: Some(FulfillmentMessage { fulfillment: Some(f) }),
            })
        })
    }
}
