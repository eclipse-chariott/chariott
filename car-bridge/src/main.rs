// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{sync::Arc, time::Duration};

use chariott_common::{
    chariott_api::{ChariottCommunication, GrpcChariott},
    config::env,
    error::Error,
    shutdown::ctrl_c_cancellation,
};
use chariott_proto::{
    common::{IntentEnum, ValueEnum, ValueMessage},
    runtime::FulfillRequest,
};
use drainage::Drainage;
use messaging::{MqttMessaging, Publisher, Subscriber};
use paho_mqtt::{Message as MqttMessage, MessageBuilder, Properties, PropertyCode, QOS_2};
use prost::Message;
use tokio::{
    select, spawn,
    sync::mpsc::{self, Sender},
    time::timeout,
};
use tokio_stream::StreamExt as _;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn, Level};
use tracing_subscriber::{util::SubscriberInitExt as _, EnvFilter};
use url::Url;

mod drainage;
mod messaging;

const VIN_ENV_NAME: &str = "VIN";
const DEFAULT_VIN: &str = "1";
const BROKER_URL_ENV_NAME: &str = "BROKER_URL";
const DEFAULT_BROKER_URL: &str = "tcp://localhost:1883";
const PUBLISH_BUFFER: usize = 50;
const DRAIN_TIMEOUT: Duration = Duration::from_secs(10);

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
    let broker_url =
        env::<Url>(BROKER_URL_ENV_NAME).unwrap_or_else(|| DEFAULT_BROKER_URL.try_into().unwrap());

    let chariott = GrpcChariott::connect().await?;

    let mut client = MqttMessaging::connect(broker_url, vin.to_owned()).await?;
    let mut messages = client.subscribe(format!("c2d/{vin}")).await?;
    let client = Arc::new(client);

    let cancellation_token = ctrl_c_cancellation();
    let drainage = Drainage::new();

    let (response_sender, mut response_receiver) = mpsc::channel(PUBLISH_BUFFER);
    let mut response_sender = Some(response_sender);

    loop {
        select! {
            message = messages.next() => {
                let Some(message) = message else {
                    break;
                };

                let Some(response_sender) = response_sender.as_ref() else {
                    // TODO: stop the MQTT client to consume messages.
                    debug!("Message will not be handled. Shutting down the Car Bridge.");
                    continue;
                };

                spawn(handle_message(chariott.clone(), response_sender.clone(), message, cancellation_token.child_token()));
            }
            message = response_receiver.recv() => {
                let Some((topic, message)) = message else {
                    // All senders are dropped and hence the channel is closed.
                    // This shuts down the Car Bridge.
                    debug!("Response receiver stopped, no more messages will be published.");
                    break;
                };

                let client = Arc::clone(&client);

                spawn(drainage.track(async move {
                    if let Err(e) = client.publish(topic, message).await {
                        debug!("Error when publishing message: '{:?}'.", e);
                    }
                }));
            }
            _ = cancellation_token.cancelled(), if response_sender.is_some() => {
                debug!("Shutting down.");
                // Setting the sender to `None` ensures that the
                // `response_receiver` gets a notification for a closed channel
                // as soon as all `handle_message` tasks finished executing.
                // https://docs.rs/tokio/latest/tokio/sync/mpsc/#disconnection
                response_sender = None;
            }
        }
    }

    if timeout(DRAIN_TIMEOUT, drainage.drain()).await.is_err() {
        warn!("In-flight tasks could not be drained within {} seconds.", DRAIN_TIMEOUT.as_secs());
    }

    client.disconnect().await?;

    Ok(())
}

async fn handle_message(
    mut chariott: impl ChariottCommunication,
    response_sender: Sender<(String, MessageBuilder)>,
    message: MqttMessage,
    cancellation_token: CancellationToken,
) {
    let correlation_information = match message.get_correlation_information() {
        Ok(cm) => cm,
        Err(error) => {
            debug!("Error when getting correlation information from message: '{error:?}'.");
            return;
        }
    };

    let response = async {
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

        let mut payload = vec![];
        response.encode(&mut payload)?;

        Ok(Response {
            payload,
            content_type: "application/x-proto+chariott.common.v1.FulfillResponse",
            is_error: false,
        })
    };

    let response: Result<_, Box<dyn std::error::Error + Send + Sync>> = select! {
        response = response => response,
        _ = cancellation_token.cancelled() => Err(Error::new("Operation was cancelled.").into())
    };

    let response = match response {
        Ok(message) => message,
        Err(error) => {
            debug!("Error when handling message: '{error:?}'.");

            let message = ValueMessage { value: Some(ValueEnum::String(format!("{error:?}"))) };

            let mut payload = vec![];
            if let Err(err) = message.encode(&mut payload) {
                debug!("Failed to encode error response: '{err}'.");
            }

            Response {
                payload,
                content_type: "application/x-proto+chariott.common.v1.Value",
                is_error: true,
            }
        }
    };

    let mut properties = Properties::new();

    if let Err(err) = properties.push_string(PropertyCode::ContentType, response.content_type) {
        debug!("Could not set content type in properties: '{err}'.");
        return;
    }

    if let Err(err) = properties.push_string_pair(
        PropertyCode::UserProperty,
        "error",
        if response.is_error { "1" } else { "0" },
    ) {
        debug!("Could not set error user property: '{err}'.");
        return;
    }

    if let Err(err) = properties
        .push_binary(PropertyCode::CorrelationData, correlation_information.correlation_data)
    {
        debug!("Could not set correlation data in properties: '{err}'.");
        return;
    }

    if let Err(err) = response_sender
        .send((
            correlation_information.response_topic,
            MessageBuilder::new().payload(response.payload).properties(properties).qos(QOS_2),
        ))
        .await
    {
        debug!("Failed to send message: '{err:?}'.");
    }
}

struct Response {
    payload: Vec<u8>,
    content_type: &'static str,
    is_error: bool,
}

struct CorrelationInformation {
    correlation_data: Vec<u8>,
    response_topic: String,
}

trait MqttExt {
    fn properties(&self) -> &Properties;

    fn get_correlation_information(&self) -> Result<CorrelationInformation, Error> {
        let correlation_data = self
            .properties()
            .get_binary(PropertyCode::CorrelationData)
            .ok_or_else(|| Error::new("No correlation data found on message."))?;

        let response_topic = self
            .properties()
            .get_string(PropertyCode::ResponseTopic)
            .ok_or_else(|| Error::new("No response topic found on message."))?;

        Ok(CorrelationInformation { correlation_data, response_topic })
    }
}

impl MqttExt for MqttMessage {
    fn properties(&self) -> &Properties {
        self.properties()
    }
}
