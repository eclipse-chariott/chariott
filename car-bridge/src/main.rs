// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::sync::Arc;

use chariott_common::{
    chariott_api::{ChariottCommunication, GrpcChariott},
    config::env,
    error::Error,
    shutdown::ctrl_c_cancellation,
};
use chariott_proto::{
    common::{
        FulfillmentEnum, FulfillmentMessage, IntentEnum, SubscribeFulfillment, ValueEnum,
        ValueMessage,
    },
    runtime::{FulfillRequest, FulfillResponse},
};
use examples_common::chariott::api::Chariott as _;
use messaging::{MqttMessaging, Publisher, Subscriber};
use paho_mqtt::{Message as MqttMessage, MessageBuilder, Properties, PropertyCode, QOS_1, QOS_2};
use prost::Message;
use streaming::{Action, ProviderRegistry, SubscriptionState};
use tokio::{
    select, spawn,
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
};
use tokio_stream::StreamExt as _;
use tracing::{debug, error, warn, Level};
use tracing_subscriber::{util::SubscriberInitExt as _, EnvFilter};

mod messaging;
mod streaming;

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
    let subscription_state = Arc::new(Mutex::new(SubscriptionState::new()));
    let provider_registry = Arc::new(Mutex::new(ProviderRegistry::new()));

    let mut client = MqttMessaging::connect(vin.to_owned()).await?;
    let mut messages = client.subscribe(format!("c2d/{vin}")).await?;

    let cancellation_token = ctrl_c_cancellation();

    let (response_sender, mut response_receiver) = mpsc::channel(PUBLISH_BUFFER);

    let publish_handle = {
        // Detach sending the responses from handling the messages to avoid
        // backpressure and disconnect the client gracefully.

        let cancellation_token = cancellation_token.child_token();
        spawn(async move {
            let client = Arc::new(client);

            loop {
                select! {
                    message = response_receiver.recv() => {
                        let Some((topic, message)) = message else {
                            warn!("Response receiver stopped, no more messages will be published.");
                            break;
                        };

                        {
                            let client = Arc::clone(&client);

                            spawn(async move {
                                if let Err(e) = client.publish(topic, message).await {
                                    debug!("Error when publishing message: '{:?}'.", e);
                                }
                            });
                        }
                    }
                    _ = cancellation_token.cancelled() => {
                        debug!("Shutting down the publisher loop.");
                        break;
                    }
                }
            }
        })
    };

    loop {
        select! {
            message = messages.next() => {
                let Some(message) = message else {
                    break;
                };

                let mut chariott = chariott.clone();
                let response_sender = response_sender.clone();
                let subscription_state = Arc::clone(&subscription_state);
                let provider_registry = Arc::clone(&provider_registry);

                spawn(async move {
                    handle_message(&mut chariott, response_sender, subscription_state, provider_registry, message).await;
                });
            }
            _ = cancellation_token.cancelled() => {
                debug!("Shutting down the subscriber loop.");
                break;
            }
        }
    }

    publish_handle.await?;

    Ok(())
}

async fn handle_message(
    chariott: &mut impl ChariottCommunication,
    response_sender: Sender<(String, MessageBuilder)>,
    subscription_state: Arc<Mutex<SubscriptionState>>,
    provider_registry: Arc<Mutex<ProviderRegistry>>,
    message: MqttMessage,
) {
    async fn get_response(
        chariott: &mut impl ChariottCommunication,
        response_sender: &Sender<(String, MessageBuilder)>,
        subscription_state: &Arc<Mutex<SubscriptionState>>,
        provider_registry: &Arc<Mutex<ProviderRegistry>>,
        message: &MqttMessage,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let fulfill_request: FulfillRequest = Message::decode(message.payload())?;

        let namespace = fulfill_request.namespace;

        let intent_enum = fulfill_request
            .intent
            .and_then(|intent| intent.intent)
            .ok_or_else(|| Error::new("Message does not contain an intent."))?;

        let response = match intent_enum {
            IntentEnum::Discover(_) => Err(Error::new("Discover is not supported.")),
            IntentEnum::Subscribe(subscribe_intent) => {
                for source in subscribe_intent.sources {
                    // Hold the lock over the entire action handling, to avoid
                    // race conditions (e.g. two applications with respect to
                    // listening, and especially failing operations).

                    let mut subscription_state = subscription_state.lock().await;

                    while let Some(action) = subscription_state.next_action(
                        namespace.clone(),
                        source.clone(),
                        subscribe_intent.channel_id.clone(),
                    ) {
                        let mut provider_events = provider_registry.lock().await;

                        match action.clone() {
                            Action::Listen(namespace) => {
                                provider_events
                                    .register_event_provider(chariott, namespace)
                                    .await?;
                            }
                            Action::Subscribe(namespace, source) => {
                                let channel_id = provider_events
                                    .get_event_provider_mut(&namespace)
                                    .unwrap()
                                    .channel_id()
                                    .to_owned();

                                chariott
                                    .subscribe(namespace, channel_id, vec![source.into()])
                                    .await?;
                            }
                            Action::Link(namespace, topic) => {
                                let mut stream = provider_events
                                    .get_event_provider(&namespace)
                                    .unwrap()
                                    .link(topic.clone());

                                let response_sender = response_sender.clone();

                                spawn(async move {
                                    while let Some(e) = stream.next().await {
                                        let mut buffer = vec![];

                                        if let Err(err) = e.encode(&mut buffer) {
                                            warn!("Failed to encode event: '{err:?}'.");
                                        }

                                        if let Err(err) = response_sender
                                            .send((
                                                topic.clone(),
                                                MessageBuilder::new().payload(buffer).qos(QOS_1),
                                            ))
                                            .await
                                        {
                                            // TODO: handle better.
                                            error!(
                                                "Failed to publish event to topic '{topic}': '{err:?}'."
                                            );
                                        }
                                    }

                                    warn!("Stream for topic '{topic}' broke.");
                                });
                            }
                            Action::Route(namespace, topic, source) => {
                                let subscription = provider_events
                                    .get_event_provider(&namespace)
                                    .unwrap()
                                    .route(topic, source)
                                    .expect("Prior to routing there we must have set up a link between a namespace and a topic.");

                                spawn(subscription.serve(|e, _| e));
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
            _ => chariott.fulfill(namespace, intent_enum).await.map(|r| r.into_inner()),
        }?;

        let mut payload = vec![];
        response.encode(&mut payload)?;

        Ok(Response {
            payload,
            content_type: "application/x-proto+chariott.common.v1.FulfillResponse",
            is_error: false,
        })
    }

    let correlation_information = match message.get_correlation_information() {
        Ok(cm) => cm,
        Err(error) => {
            debug!("Error when getting correlation information from message: '{error:?}'.");
            return;
        }
    };

    let response = match get_response(
        chariott,
        &response_sender,
        &subscription_state,
        &provider_registry,
        &message,
    )
    .await
    {
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

    properties.push_string(PropertyCode::ContentType, response.content_type).unwrap();

    properties
        .push_string_pair(
            PropertyCode::UserProperty,
            "error",
            match response.is_error {
                true => "1",
                false => "0",
            },
        )
        .unwrap();

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
    fn get_correlation_information(&self) -> Result<CorrelationInformation, Error>;
}

impl MqttExt for MqttMessage {
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
