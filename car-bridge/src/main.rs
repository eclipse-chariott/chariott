// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{env, error::Error};

use car_bridge::{
    chariott::fulfill,
    messaging::{Messaging, MqttMessaging},
};
use chariott_common::{chariott_api::GrpcChariott, shutdown::ctrl_c_cancellation};
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

    let vin = env::var(VIN_ENV_NAME).unwrap_or_else(|_| DEFAULT_VIN.to_owned());

    let chariott = GrpcChariott::connect().await?;

    let client = MqttMessaging::connect(format!("{}", vin)).await?;
    let mut messages = client.receive(format!("c2d/{vin}")).await?;

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
                    let message = MessageBuilder::new().topic(message.properties().get_string(PropertyCode::ResponseTopic).unwrap()).payload(buffer).qos(QOS_2).finalize();
                    client.send(message).await?;
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
