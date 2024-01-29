// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::sync::Arc;
use std::vec;

use async_trait::async_trait;
use examples_common::intent_brokering;
use examples_common::intent_brokering::inspection::{fulfill, Entry};
use examples_common::intent_brokering::streaming::ProtoExt as _;
use examples_common::intent_brokering::value::Value;
use intent_brokering_proto::{
    common::{
        discover_fulfillment::Service, DiscoverFulfillment, FulfillmentEnum, FulfillmentMessage,
        IntentEnum, InvokeFulfillment, ValueMessage,
    },
    provider::{provider_service_server::ProviderService, FulfillRequest, FulfillResponse},
};
use tonic::{Request, Response, Status};
use url::Url;

use crate::simulation::VehicleSimulation;

pub const CABIN_TEMPERATURE_PROPERTY: &str = "Vehicle.Cabin.HVAC.AmbientAirTemperature";
pub const BATTERY_LEVEL_PROPERTY: &str = "Vehicle.OBD.HybridBatteryRemaining";
pub const AIR_CONDITIONING_STATE_PROPERTY: &str = "Vehicle.Cabin.HVAC.IsAirConditioningActive";
pub const ACTIVATE_AIR_CONDITIONING_COMMAND: &str = "Vehicle.Cabin.HVAC.IsAirConditioningActive";
pub const SEND_NOTIFICATION_COMMAND: &str = "send_notification";
pub const SET_UI_MESSAGE_COMMAND: &str = "set_ui_message";

const SCHEMA_VERSION_STREAMING: &str = "intent_brokering.streaming.v1";
const SCHEMA_REFERENCE: &str = "grpc+proto";

pub type StreamingStore = intent_brokering::streaming::StreamingStore<Value>;

pub struct IntentProvider {
    url: Url,
    vehicle_simulation: VehicleSimulation,
    streaming_store: Arc<StreamingStore>,
}

impl IntentProvider {
    pub fn new(
        url: Url,
        simulation: VehicleSimulation,
        streaming_store: Arc<StreamingStore>,
    ) -> Self {
        Self { url, vehicle_simulation: simulation, streaming_store }
    }
}

lazy_static::lazy_static! {
    static ref VDT_SCHEMA: Vec<Entry> = vec![
        property(CABIN_TEMPERATURE_PROPERTY, "int32"),
        property(BATTERY_LEVEL_PROPERTY, "int32"),
        property(AIR_CONDITIONING_STATE_PROPERTY, "bool"),
        command(ACTIVATE_AIR_CONDITIONING_COMMAND, "IAcmeAirconControl"),
        command(SEND_NOTIFICATION_COMMAND, "ISendNotification"),
        command(SET_UI_MESSAGE_COMMAND, "ISetUiMessage"),
    ];
}

fn property(path: &str, r#type: &str) -> Entry {
    Entry::new(
        path,
        [
            ("member_type", "property".into()),
            ("type", r#type.into()),
            ("read", Value::TRUE),
            ("write", Value::FALSE),
            ("watch", Value::TRUE),
        ],
    )
}

fn command(path: &str, r#type: &str) -> Entry {
    Entry::new(path, [("member_type", "command"), ("type", r#type)])
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
            IntentEnum::Discover(_) => FulfillmentEnum::Discover(DiscoverFulfillment {
                services: vec![Service {
                    url: self.url.to_string(),
                    schema_kind: SCHEMA_REFERENCE.to_owned(),
                    schema_reference: SCHEMA_VERSION_STREAMING.to_owned(),
                    metadata: HashMap::new(),
                }],
            }),
            IntentEnum::Invoke(intent) => {
                let args = intent
                    .args
                    .into_iter()
                    .map(|arg| arg.try_into())
                    .collect::<Result<Vec<Value>, ()>>()
                    .map_err(|_| Status::invalid_argument("Invalid argument."))?;

                let result = self
                    .vehicle_simulation
                    .invoke(&intent.command, args)
                    .await
                    .map_err(|e| {
                        Status::unknown(format!("Error when invoking hardware function: '{}'", e))
                    })?
                    .into();

                FulfillmentEnum::Invoke(InvokeFulfillment {
                    r#return: Some(ValueMessage { value: Some(result) }),
                })
            }
            IntentEnum::Inspect(inspect) => fulfill(inspect.query, &*VDT_SCHEMA),
            IntentEnum::Subscribe(subscribe) => self.streaming_store.subscribe(subscribe)?,
            IntentEnum::Read(read) => self.streaming_store.read(read),
            _ => return Err(Status::unknown("Unknown or unsupported intent!")),
        };

        Ok(Response::new(FulfillResponse {
            fulfillment: Some(FulfillmentMessage { fulfillment: Some(response) }),
        }))
    }
}
