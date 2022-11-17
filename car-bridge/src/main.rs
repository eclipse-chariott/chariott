// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::error::Error;

use car_bridge::{
    chariott::fulfill,
    messaging::{MqttMessaging, Publisher, Subscriber},
};
use chariott_common::{chariott_api::GrpcChariott, config::env, shutdown::ctrl_c_cancellation};
use paho_mqtt::{MessageBuilder, Properties, PropertyCode, QOS_2};
use prost::Message as _;
use tokio::select;
use tokio_stream::StreamExt as _;
use tracing::Level;
use tracing_subscriber::{util::SubscriberInitExt as _, EnvFilter};

const VIN_ENV_NAME: &str = "VIN";
const DEFAULT_VIN: &str = "1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder().with_default_directive(Level::INFO.into()).from_env_lossy(),
        )
        .finish()
        .init();

    let vin = env::<String>(VIN_ENV_NAME);
    let vin = vin.as_deref().unwrap_or(DEFAULT_VIN);

    let chariott = GrpcChariott::connect().await?;

    let mut client = MqttMessaging::connect(format!("{}", vin)).await?;
    let mut messages = client.subscribe(format!("c2d/{vin}")).await?;

    let cancellation_token = ctrl_c_cancellation();

    loop {
        select! {
            message = messages.next() => {
                if let Some(message) = message {
                    // TODO: avoid backpressure issues.
                    let response = fulfill(&mut chariott.clone(), message.payload()).await?;
                    let mut buffer = vec![];
                    response.encode(&mut buffer)?;
                    let mut properties = Properties::new();
                    properties.push_binary(PropertyCode::CorrelationData, message.properties().get_binary(PropertyCode::CorrelationData).unwrap())?;
                    let topic = message.properties().get_string(PropertyCode::ResponseTopic).unwrap();
                    client.publish(topic, MessageBuilder::new().payload(buffer).qos(QOS_2)).await?;
                }
                else {
                    break;
                }
            }
            _ = cancellation_token.cancelled() => {
                break;
            }
        }
    }

    Ok(())
}
