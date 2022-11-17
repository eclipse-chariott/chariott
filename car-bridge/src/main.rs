// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use chariott_common::{
    chariott_api::{ChariottCommunication, GrpcChariott},
    config::env,
    error::Error,
    shutdown::ctrl_c_cancellation,
};
use chariott_proto::{common::IntentEnum, runtime::FulfillRequest};
use messaging::{MqttMessaging, Publisher, Subscriber};
use paho_mqtt::{Message as MqttMessage, MessageBuilder, Properties, PropertyCode, QOS_2};
use prost::Message;
use tokio::{
    select, spawn,
    sync::mpsc::{self, Sender},
};
use tokio_stream::StreamExt as _;
use tracing::{error, warn, Level};
use tracing_subscriber::{util::SubscriberInitExt as _, EnvFilter};

mod messaging;

const VIN_ENV_NAME: &str = "VIN";
const DEFAULT_VIN: &str = "1";
const PUBLISH_BUFFER: usize = 50;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Detach sending the responses from handling the messages to avoid
    // backpressure.

    let (response_sender, mut response_receiver) = mpsc::channel(PUBLISH_BUFFER);

    {
        let cancellation_token = cancellation_token.child_token();
        spawn(async move {
            loop {
                select! {
                    message = response_receiver.recv() => {
                        let Some((topic, message)) = message else {
                            warn!("Response receiver stopped, no more messages will be published.");
                            break;
                        };

                        if let Err(e) = client.publish(topic, message).await {
                            error!("Error when publishing message: '{:?}'.", e);
                        }
                    }
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                }
            }
        });
    }

    loop {
        select! {
            message = messages.next() => {
                if let Some(message) = message {
                    let mut chariott = chariott.clone();
                    let response_sender = response_sender.clone();

                    spawn(async move {
                        handle_message(&mut chariott, response_sender, message).await;
                    });
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

async fn handle_message(
    chariott: &mut impl ChariottCommunication,
    response_sender: Sender<(String, MessageBuilder)>,
    message: MqttMessage,
) {
    async fn inner(
        chariott: &mut impl ChariottCommunication,
        response_sender: Sender<(String, MessageBuilder)>,
        message: MqttMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let correlation_data = message
            .properties()
            .get_binary(PropertyCode::CorrelationData)
            .ok_or_else(|| Error::new("No correlation data found on message."))?;

        let response_topic = message
            .properties()
            .get_string(PropertyCode::ResponseTopic)
            .ok_or_else(|| Error::new("No correlation data found on message."))?;

        let fulfill_request: FulfillRequest = Message::decode(message.payload())?;

        let intent_enum = fulfill_request
            .intent
            .and_then(|intent| intent.intent)
            .ok_or_else(|| Error::new("Message does not contain an intent."))?;

        let response = match intent_enum {
            IntentEnum::Discover(_) => Err(Error::new("Discover is not supported.")),
            IntentEnum::Subscribe(_) => todo!(),
            _ => chariott
                .fulfill(fulfill_request.namespace, intent_enum)
                .await
                .map(|r| r.into_inner()),
        }?;

        let mut buffer = vec![];
        response.encode(&mut buffer)?;

        let mut properties = Properties::new();
        properties.push_binary(PropertyCode::CorrelationData, correlation_data)?;
        properties.push_string(PropertyCode::ContentType, "chariott.runtime.v1.FulfillResponse")?;

        response_sender
            .send((response_topic, MessageBuilder::new().payload(buffer).qos(QOS_2)))
            .await?;

        Ok(())
    }

    if let Err(e) = inner(chariott, response_sender, message).await {
        error!("Error when handling message: '{e:?}'.");
    }
}
