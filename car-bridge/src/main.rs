// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{error::Error, time::Duration};

use chariott_common::{config::env, shutdown::ctrl_c_cancellation};
use paho_mqtt::{
    AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder, Message, MQTT_VERSION_5, QOS_2,
};
use tokio::{select, time::sleep};
use tokio_stream::StreamExt as _;
use tracing::{info, Level};
use tracing_subscriber::{util::SubscriberInitExt as _, EnvFilter};

const VIN_ENV_NAME: &str = "VIN";
const DEFAULT_VIN: &str = "1";
const BROKER_URL_ENV_NAME: &str = "BROKER_URL";
const DEFAULT_BROKER_URL: &str = "tcp://localhost:1883";
const MQTT_CLIENT_BUFFER_SIZE: usize = 25;

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
    let host = env::<String>(BROKER_URL_ENV_NAME);
    let host = host.as_deref().unwrap_or(DEFAULT_BROKER_URL);

    info!("Connecting to MQTT broker on '{host}'.");

    let mut client = AsyncClient::new(
        CreateOptionsBuilder::new()
            .mqtt_version(MQTT_VERSION_5)
            .server_uri(host)
            .client_id(format!("car-bridge-{vin}"))
            .finalize(),
    )?;

    let mut message_stream = client.get_stream(MQTT_CLIENT_BUFFER_SIZE);

    client.connect(ConnectOptionsBuilder::new().mqtt_version(MQTT_VERSION_5).finalize()).await?;

    let c2d_topic = format!("c2d/{vin}");
    info!("Subscribing to topic '{c2d_topic}'.");
    client.subscribe(c2d_topic.clone(), QOS_2).await?;

    // Publish a phony message.
    client.publish(Message::new(c2d_topic, "Example payload", QOS_2)).await?;

    let cancellation_token = ctrl_c_cancellation();

    loop {
        select! {
            message = message_stream.next() => {
                let Some(message) = message else {
                    break;
                };
                if let Some(message) = message {
                    info!("(R) {message:?}");
                }
                else {
                    info!("Connection temporarily lost.");

                    while let Err(err) = client.reconnect().await {
                        info!("Trying to reconnect: {}.", err);
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            }
            _ = cancellation_token.cancelled() => {
                break;
            }
        }
    }

    info!("Disconnecting the client.");
    client.disconnect(None).await?;

    Ok(())
}
