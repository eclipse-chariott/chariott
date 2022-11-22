// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::sync::{Arc, Mutex};

use chariott_common::{
    chariott_api::{ChariottCommunication, GrpcChariott},
    config::env,
    error::Error,
    shutdown::ctrl_c_cancellation,
};
use chariott_proto::{
    common::{FulfillmentEnum, FulfillmentMessage, IntentEnum, SubscribeFulfillment},
    runtime::{FulfillRequest, FulfillResponse},
};
use examples_common::chariott::api::{Chariott as _, ChariottExt as _};
use messaging::{MqttMessaging, Publisher, Subscriber};
use paho_mqtt::{Message as MqttMessage, MessageBuilder, Properties, PropertyCode, QOS_1, QOS_2};
use prost::Message;
use streaming::{Action, Streaming, SubscriptionRegistry};
use tokio::{
    select, spawn,
    sync::mpsc::{self, Sender},
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
    let streaming = Arc::new(Mutex::new(Streaming::new()));
    let ess_registry = Arc::new(Mutex::new(SubscriptionRegistry::new()));

    let mut client = MqttMessaging::connect(vin.to_owned()).await?;
    let mut messages = client.subscribe(format!("c2d/{vin}")).await?;

    let cancellation_token = ctrl_c_cancellation();

    let (response_sender, mut response_receiver) = mpsc::channel(PUBLISH_BUFFER);

    let publish_handle = {
        // Detach sending the responses from handling the messages to avoid
        // backpressure and disconnect the client gracefully.

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
                let streaming = Arc::clone(&streaming);
                let ess_registry = Arc::clone(&ess_registry);

                spawn(async move {
                    handle_message(&mut chariott, response_sender, streaming, ess_registry, message).await;
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
    streaming: Arc<Mutex<Streaming>>,
    ess_registry: Arc<Mutex<SubscriptionRegistry>>,
    message: MqttMessage,
) {
    async fn inner(
        chariott: &mut impl ChariottCommunication,
        response_sender: Sender<(String, MessageBuilder)>,
        streaming: Arc<Mutex<Streaming>>,
        ess_registry: Arc<Mutex<SubscriptionRegistry>>,
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

        let namespace = fulfill_request.namespace;

        let intent_enum = fulfill_request
            .intent
            .and_then(|intent| intent.intent)
            .ok_or_else(|| Error::new("Message does not contain an intent."))?;

        let response = match intent_enum {
            IntentEnum::Discover(_) => Err(Error::new("Discover is not supported.")),
            IntentEnum::Subscribe(subscribe_intent) => {
                let mut actions = vec![];

                {
                    let mut streaming = streaming.lock().unwrap();

                    for source in subscribe_intent.sources {
                        actions.push(streaming.subscribe(
                            namespace.clone(),
                            source,
                            subscribe_intent.channel_id.clone(),
                        ));
                    }
                }

                // TODO: handle errors - roll back subscription state.

                for action in actions.into_iter().flatten() {
                    let ess_registry = Arc::clone(&ess_registry);

                    match action {
                        Action::Listen(namespace) => {
                            // open the stream with the provider
                            let mut stream = chariott.listen(namespace.clone(), vec![]).await?;

                            ess_registry
                                .lock()
                                .unwrap()
                                .set_channel_id(namespace.clone(), "".to_owned());

                            {
                                let namespace = namespace.clone();

                                spawn(async move {
                                    while let Some(event) = stream.next().await {
                                        let event = event.unwrap();

                                        ess_registry
                                            .lock()
                                            .unwrap()
                                            .publish(namespace.as_str(), &event.id.clone(), event)
                                            .unwrap();
                                    }

                                    warn!("Stream for channel '{namespace}' broke.");
                                });
                            }
                        }
                        Action::Subscribe(namespace, source) => {
                            let channel_id = ess_registry
                                .lock()
                                .unwrap()
                                .channel_id(&namespace)
                                .unwrap()
                                .to_owned();

                            chariott
                                .subscribe(
                                    namespace.clone(),
                                    channel_id.clone(),
                                    vec![source.as_str().into()],
                                )
                                .await?;

                            let subscription = ess_registry
                                .lock()
                                .unwrap()
                                .subscribe(namespace.clone(), source, channel_id)
                                .unwrap();

                            spawn(subscription.serve(|event, _| event));
                        }
                        Action::Link(namespace, topic) => {
                            let (_, mut topic_stream) = ess_registry
                                .lock()
                                .unwrap()
                                .ess(&namespace)
                                .read_events(topic.clone());

                            let response_sender = response_sender.clone();

                            spawn(async move {
                                while let Some(_) = topic_stream.next().await {
                                    // TODO: send real data
                                    if let Err(e) = response_sender
                                        .send((
                                            topic.clone(),
                                            MessageBuilder::new().payload(vec![]).qos(QOS_1),
                                        ))
                                        .await
                                    {
                                        // TODO: handle better.
                                        error!(
                                            "Failed to publish event to topic '{topic}': '{e:?}'."
                                        );
                                    }
                                }

                                warn!("Stream for topic '{topic}' broke.");
                            });
                        }
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

    if let Err(e) = inner(chariott, response_sender, streaming, ess_registry, message).await {
        error!("Error when handling message: '{e:?}'.");
    }
}
