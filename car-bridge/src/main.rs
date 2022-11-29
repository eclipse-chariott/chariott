// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{sync::Arc, time::Duration};

use chariott_common::{
    chariott_api::{ChariottCommunication, GrpcChariott},
    config::env,
    error::{Error, ResultExt as _},
    shutdown::ctrl_c_cancellation,
};
use chariott_proto::{
    common::{
        FulfillmentEnum, FulfillmentMessage, IntentEnum, SubscribeFulfillment, ValueEnum,
        ValueMessage,
    },
    runtime::{FulfillRequest, FulfillResponse},
};
use drainage::Drainage;
use messaging::{MqttMessaging, Publisher, Subscriber};
use paho_mqtt::{Message as MqttMessage, MessageBuilder, Properties, PropertyCode, QOS_1, QOS_2};
use prost::Message as ProtoMessage;
use streaming::{Action, ProviderRegistry, SubscriptionState};
use tokio::{
    select, spawn,
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
    time::timeout,
};
use tokio_stream::StreamExt as _;
use tracing::{debug, warn, Level};
use tracing_subscriber::{util::SubscriberInitExt as _, EnvFilter};
use url::Url;

mod drainage;
mod messaging;
mod streaming;

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
    let subscription_state = Arc::new(Mutex::new(SubscriptionState::new()));
    let provider_registry = Arc::new(Mutex::new(ProviderRegistry::new()));

    let mut client = MqttMessaging::connect(broker_url, vin.to_owned()).await?;
    let mut messages = client.subscribe(format!("c2d/{vin}")).await?;
    let client = Arc::new(client);

    let cancellation_token = ctrl_c_cancellation();
    let drainage = Drainage::new();

    let (response_sender, mut response_receiver) = mpsc::channel(PUBLISH_BUFFER);
    let response_sender = ResponseSender(response_sender);

    loop {
        select! {
            message = messages.next() => {
                let Some(message) = message else {
                    break;
                };

                spawn(handle_message(chariott.clone(), response_sender.clone(), Arc::clone(&subscription_state),  Arc::clone(&provider_registry), message));
            }
            message = response_receiver.recv() => {
                let Some((topic, message)) = message else {
                    warn!("Response receiver stopped, no more messages will be published.");
                    break;
                };

                let client = Arc::clone(&client);

                spawn(drainage.track(async move {
                    if let Err(e) = client.publish(topic, message).await {
                        debug!("Error when publishing message: '{:?}'.", e);
                    }
                }));
            }
            _ = cancellation_token.cancelled() => {
                debug!("Shutting down.");
                break;
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
    response_sender: ResponseSender,
    subscription_state: Arc<Mutex<SubscriptionState>>,
    provider_registry: Arc<Mutex<ProviderRegistry>>,
    message: MqttMessage,
) {
    let request: Request = match message.try_into() {
        Ok(r) => r,
        Err(e) => {
            debug!("Failed to parse message: '{e:?}'.");
            return;
        }
    };

    let correlation_information = request.correlation_information.clone();

    let response: Result<_, Box<dyn std::error::Error + Send + Sync>> = async {
        let response = match request.intent_enum {
            IntentEnum::Discover(_) => Err(Error::new("Discover is not supported.")),
            IntentEnum::Subscribe(subscribe_intent) => {
                for source in subscribe_intent.sources {
                    // Hold the lock over the entire action handling, to avoid
                    // race conditions (e.g. two applications with respect to
                    // listening, and especially failing operations).

                    let mut subscription_state = subscription_state.lock().await;

                    while let Some(action) = subscription_state.next_action(
                        request.namespace.clone(),
                        source.clone(),
                        subscribe_intent.channel_id.clone(),
                    ) {
                        let mut provider_events = provider_registry.lock().await;

                        match action.clone() {
                            Action::Listen(namespace) => {
                                provider_events
                                    .register_event_provider(&mut chariott, namespace)
                                    .await?;
                            }
                            Action::Subscribe(namespace, source) => {
                                provider_events
                                    .get_event_provider_mut(&namespace)
                                    .expect(
                                        "Prior to subscribing we must have established listening.",
                                    )
                                    .subscribe(&mut chariott, source)
                                    .await?;
                            }
                            Action::Link(namespace, topic) => {
                                provider_events
                                    .get_event_provider(&namespace)
                                    .expect("Prior to linking we must have established listening.")
                                    .link(topic, response_sender.clone());
                            }
                            Action::Route(namespace, topic, source) => {
                                provider_events
                                    .get_event_provider(&namespace)
                                    .expect("Prior to routing we must have established listening.")
                                    .route(topic, source)
                                    .expect(
                                        "Prior to routing there we must have established linking.",
                                    );
                            }
                        }

                        subscription_state.commit(action);
                    }
                }

                Ok(FulfillResponse {
                    fulfillment: Some(FulfillmentMessage {
                        fulfillment: Some(FulfillmentEnum::Subscribe(SubscribeFulfillment {})),
                    }),
                })
            }
            _ => chariott
                .fulfill(request.namespace, request.intent_enum)
                .await
                .map(|r| r.into_inner()),
        }?;

        let mut payload = vec![];
        response.encode(&mut payload)?;

        Ok(Message::SuccessResponse(payload, request.correlation_information))
    }
    .await;

    let response = match response {
        Ok(message) => message,
        Err(error) => {
            debug!("Error when handling message: '{error:?}'.");
            Message::ErrorResponse(format!("{error:?}"), correlation_information)
        }
    };

    response_sender.send(response).await;
}

struct Request {
    intent_enum: IntentEnum,
    namespace: String,
    correlation_information: CorrelationInformation,
}

impl TryFrom<MqttMessage> for Request {
    type Error = Error;

    fn try_from(value: MqttMessage) -> Result<Self, Self::Error> {
        let correlation_data = value
            .properties()
            .get_binary(PropertyCode::CorrelationData)
            .ok_or_else(|| Error::new("No correlation data found on message."))?;

        let response_topic = value
            .properties()
            .get_string(PropertyCode::ResponseTopic)
            .ok_or_else(|| Error::new("No response topic found on message."))?;

        // We could return the following errors as we know the correlation
        // information, but do not do it because it adds little value for more
        // complexity. If the request is invalid, we do not process it.

        let fulfill_request: FulfillRequest = ProtoMessage::decode(value.payload())
            .map_err_with("Failed to decode message payload as 'FulfillRequest'.")?;

        let intent_enum = fulfill_request
            .intent
            .and_then(|intent| intent.intent)
            .ok_or_else(|| Error::new("Message does not contain an intent."))?;

        Ok(Self {
            intent_enum,
            namespace: fulfill_request.namespace,
            correlation_information: CorrelationInformation { correlation_data, response_topic },
        })
    }
}

#[derive(Clone)]
pub struct ResponseSender(Sender<(String, MessageBuilder)>);

impl ResponseSender {
    /// Queues a message to be published.
    pub async fn send(&self, message: Message) {
        let response = async {
            let message = message.try_into_message()?;

            self.0
                .send(message)
                .await
                .map_err(|e| Error::new(format!("Error when sending message to channel: '{e:?}'.")))
        };

        if let Err(err) = response.await {
            debug!("Failed to send message: '{err:?}'.");
        }
    }
}

type Topic = String;

pub enum Message {
    Event(Vec<u8>, Topic),
    SuccessResponse(Vec<u8>, CorrelationInformation),
    ErrorResponse(String, CorrelationInformation),
}

impl Message {
    fn try_into_message(self) -> Result<(Topic, MessageBuilder), Error> {
        fn get_properties(
            content_type: &str,
            is_error: bool,
            correlation_data: Option<Vec<u8>>,
        ) -> Result<Properties, Error> {
            let mut properties = Properties::new();

            properties
                .push_string(PropertyCode::ContentType, content_type)
                .map_err_with("Could not set content type property.")?;

            properties
                .push_string_pair(
                    PropertyCode::UserProperty,
                    "error",
                    if is_error { "1" } else { "0" },
                )
                .map_err_with("Could not set user-defined error property.")?;

            if let Some(correlation_data) = correlation_data {
                properties
                    .push_binary(PropertyCode::CorrelationData, correlation_data)
                    .map_err_with("Could not set correlation data in properties.")?;
            }

            Ok(properties)
        }

        let (payload, qos, topic, properties) = match self {
            Message::SuccessResponse(payload, correlation_information) => (
                payload,
                QOS_2,
                correlation_information.response_topic,
                get_properties(
                    "application/x-proto+chariott.common.v1.FulfillResponse",
                    false,
                    Some(correlation_information.correlation_data),
                )?,
            ),
            Message::ErrorResponse(message, correlation_information) => {
                let mut payload = vec![];
                let message = ValueMessage { value: Some(ValueEnum::String(message)) };
                message.encode(&mut payload).map_err_with("Failed to encode error response.")?;

                (
                    payload,
                    QOS_2,
                    correlation_information.response_topic,
                    get_properties(
                        "application/x-proto+chariott.common.v1.Value",
                        true,
                        Some(correlation_information.correlation_data),
                    )?,
                )
            }
            Message::Event(payload, topic) => (
                payload,
                QOS_1,
                topic,
                get_properties("application/x-proto+chariott.streaming.v1.Event", false, None)?,
            ),
        };

        Ok((topic, MessageBuilder::new().payload(payload).properties(properties).qos(qos)))
    }
}

#[derive(Clone)]
pub struct CorrelationInformation {
    correlation_data: Vec<u8>,
    response_topic: String,
}
