// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{env, error::Error};

use car_bridge::{
    chariott::fulfill,
    messaging::{Messaging, MqttMessaging},
};
use chariott_common::{chariott_api::GrpcChariott, shutdown::ctrl_c_cancellation};
use paho_mqtt::{Message, QOS_2};
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

    let mut messages = MqttMessaging::connect(format!("{}-receiver", vin.clone()))
        .await?
        .receive(format!("c2d/{vin}"))
        .await?;

    let sender = MqttMessaging::connect(format!("{}-sender", vin.clone())).await?;

    let cancellation_token = ctrl_c_cancellation();

    loop {
        select! {
            message = messages.next() => {
                if let Some(message) = message {
                    // TODO: avoid backpressure issues.
                    let response = fulfill(&mut chariott.clone(), message.payload()).await?;
                    let mut buffer = vec![];
                    response.encode(&mut buffer)?;
                    sender.send(Message::new(format!("responses/{vin}"), buffer, QOS_2)).await?;
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
