// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::time::Duration;

use async_channel::Receiver;
use async_stream::stream;
use async_trait::async_trait;
use chariott_common::{
    config::env,
    error::{Error, ResultExt as _},
};
use futures::{stream::BoxStream, StreamExt as _};
use paho_mqtt::{
    AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder, Message, MessageBuilder,
    MQTT_VERSION_5, QOS_2,
};
use tracing::info;

#[async_trait]
pub trait Subscriber {
    type Message;
    type Topic;

    /// Subscribe to a topic.
    async fn subscribe(
        &mut self,
        topic: String,
    ) -> Result<BoxStream<'static, Self::Message>, Error>;
}

#[async_trait]
pub trait Publisher {
    type Message;
    type Topic;

    /// Publishes a message.
    async fn publish(&self, topic: Self::Topic, message: Self::Message) -> Result<(), Error>;
}

pub struct MqttMessaging {
    client: AsyncClient,
    receiver: Receiver<Option<Message>>,
    is_subscribed: bool,
}

impl Drop for MqttMessaging {
    fn drop(&mut self) {
        // Best-effort disconnect.
        _ = self.client.disconnect(None).wait();
    }
}

impl MqttMessaging {
    /// Connects to the MQTT broker and starts listening on incoming messages.
    /// If there was a persisted session before, messages may delivered before
    /// the `connect` returns. Refer to the `Subscriber` implementation for how
    /// to get access to the buffered messages.
    pub async fn connect(client_id: String) -> Result<Self, Error> {
        const BROKER_URL_ENV_NAME: &str = "BROKER_URL";
        const DEFAULT_BROKER_URL: &str = "tcp://localhost:1883";
        const MQTT_CLIENT_BUFFER_SIZE: usize = 200;

        let host = env::<String>(BROKER_URL_ENV_NAME);
        let host = host.as_deref().unwrap_or(DEFAULT_BROKER_URL);

        // The client ID is used in conjunction with session persistence to
        // re-establish existing subscriptions on disconnect. TODO: if the
        // session was not persisted, the client must reestablish the
        // subscriptions.
        let client_id = format!("car-bridge-{client_id}");

        info!("Connecting client '{client_id}' to MQTT broker at '{host}'.");

        let mut client = AsyncClient::new(
            CreateOptionsBuilder::new()
                .mqtt_version(MQTT_VERSION_5)
                .server_uri(host)
                .client_id(client_id)
                .finalize(),
        )
        .map_err_with("Failed to create MQTT client.")?;

        // Get the stream before connecting the client, as otherwise messages
        // may be lost. If the Car Bridge restarts while the number of
        // outstanding messages is larger than the buffer size, messages may be
        // lost.

        let receiver = client.get_stream(MQTT_CLIENT_BUFFER_SIZE);

        client
            .connect(
                ConnectOptionsBuilder::new()
                    .mqtt_version(MQTT_VERSION_5)
                    .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(60))
                    .finalize(),
            )
            .await
            .map_err_with("Could not connect to MQTT broker.")?;

        Ok(Self { client, receiver, is_subscribed: false })
    }
}

#[async_trait]
impl Subscriber for MqttMessaging {
    type Message = Message;
    type Topic = String;

    /// Subscribes to a topic on the MQTT broker. Currently, this can only be
    /// invoked once on `MqttMessaging`, due to the structure of the underlying
    /// client.
    async fn subscribe<'a>(
        &'a mut self,
        topic: String,
    ) -> Result<BoxStream<'static, Self::Message>, Error> {
        // TODO: By broadcasting the events on the underlying `Receiver` and
        // filtering the broadcasted events by their topic name, we can support
        // multiple subscriptions. Since this is currently not needed, we do not
        // add said complexity to the implementation but fail at runtime
        // instead.

        if self.is_subscribed {
            return Err(Error::new("Already receiving messages. It is currently not possible to subscribe multiple times."));
        }

        self.is_subscribed = true;

        // C2D messages must be delivered with QOS 2, as we cannot assume that
        // the fulfill requests they contain are always idempotent.

        self.client
            .subscribe(topic.clone(), QOS_2)
            .await
            .map_err_with("Could not subscribe to topic for receiving C2D messages.")?;

        let mut receiver = self.receiver.clone();

        let s = stream! {
            while let Some(message) = receiver.next().await {
                if let Some(message) = message {
                    yield message;
                }
                else {
                    // Automatic reconnect is used when connecting the
                    // `AsyncClient`.
                    info!("Connection temporarily lost. Attempting automatic reconnect.");
                }
            }
        };

        Ok(s.boxed())
    }
}

#[async_trait]
impl Publisher for MqttMessaging {
    type Message = MessageBuilder;
    type Topic = String;

    /// Publish a message to an MQTT broker.
    async fn publish(&self, topic: Self::Topic, message: Self::Message) -> Result<(), Error> {
        self.client
            .publish(message.topic(topic).finalize())
            .await
            .map_err_with("Error when publishing a response.")
    }
}
