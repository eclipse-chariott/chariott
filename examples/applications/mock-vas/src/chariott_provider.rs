// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::collections::HashMap;
use std::sync::Arc;
use std::vec;

use async_trait::async_trait;
use examples_common::chariott;
use examples_common::chariott::inspection::{fulfill, Entry};
use examples_common::chariott::streaming::ProtoExt as _;
use examples_common::chariott::value::Value;
use tonic::{Request, Response, Status};
use url::Url;

use crate::simulation::VehicleSimulation;
use examples_common::chariott::proto::*;

pub const CABIN_TEMPERATURE_PROPERTY: &str = "Vehicle.Cabin.HVAC.AmbientAirTemperature";
pub const BATTERY_LEVEL_PROPERTY: &str = "Vehicle.OBD.HybridBatteryRemaining";
pub const AIR_CONDITIONING_STATE_PROPERTY: &str = "Vehicle.Cabin.HVAC.IsAirConditioningActive";
pub const ACTIVATE_AIR_CONDITIONING_COMMAND: &str = "Vehicle.Cabin.HVAC.IsAirConditioningActive";
pub const SEND_NOTIFICATION_COMMAND: &str = "send_notification";
pub const SET_UI_MESSAGE_COMMAND: &str = "set_ui_message";

const SCHEMA_VERSION_STREAMING: &str = "chariott.streaming.v1";
const SCHEMA_REFERENCE: &str = "grpc+proto";

pub type StreamingStore = chariott::streaming::StreamingStore<Value>;

pub struct ChariottProvider {
    url: Url,
    vehicle_simulation: VehicleSimulation,
    streaming_store: Arc<StreamingStore>,
}

impl ChariottProvider {
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
            common::intent::Intent::Discover(_) => {
                common::fulfillment::Fulfillment::Discover(common::DiscoverFulfillment {
                    services: vec![common::discover_fulfillment::Service {
                        url: self.url.to_string(),
                        schema_kind: SCHEMA_REFERENCE.to_owned(),
                        schema_reference: SCHEMA_VERSION_STREAMING.to_owned(),
                        metadata: HashMap::new(),
                    }],
                })
            }
            common::intent::Intent::Invoke(intent) => {
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

                common::fulfillment::Fulfillment::Invoke(common::InvokeFulfillment {
                    r#return: Some(common::Value { value: Some(result) }),
                })
            }
            common::intent::Intent::Inspect(inspect) => fulfill(inspect.query, &*VDT_SCHEMA),
            common::intent::Intent::Subscribe(subscribe) => {
                self.streaming_store.subscribe(subscribe)?
            }
            common::intent::Intent::Read(read) => self.streaming_store.read(read),
            _ => return Err(Status::unknown("Unknown or unsupported intent!")),
        };

        Ok(Response::new(provider::FulfillResponse {
            fulfillment: Some(common::Fulfillment { fulfillment: Some(response) }),
        }))
    }
}
