// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use async_trait::async_trait;
use chariott_common::proto::common::{
    fulfillment::Fulfillment, ReadFulfillment, ReadIntent, SubscribeFulfillment, SubscribeIntent,
};
use keyvalue::{InMemoryKeyValueStore, Observer};
use std::{
    fmt::Display,
    hash::Hash,
    sync::{Arc, RwLock},
    time::SystemTime,
};
use tokio::spawn;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};

use crate::chariott::proto::{
    common::Value as ProtoValue,
    streaming::{channel_service_server::ChannelService, Event as ProtoEvent, OpenRequest},
};

const METADATA_KEY: &str = "x-chariott-channel-id";

type ClientId = Box<str>;
type Ess<T> = ess::EventSubSystem<ClientId, EventId, (EventId, T), Result<ProtoEvent, Status>>;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct EventId(Box<str>);

impl<T: Into<Box<str>>> From<T> for EventId {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Clone> Observer<EventId, T> for Arc<Ess<T>> {
    fn on_set(&mut self, key: &EventId, value: &T) {
        self.publish(key, (key.clone(), value.clone()));
    }
}

/// Represents an in-memory store that contains a blanket implementation for
/// integration with the Chariott streaming API. It generalizes over any type of
/// value to be published, as long as that value can be transformed into a value
/// which is compatible with the Proto contract.
pub struct StreamingStore<T> {
    ess: Arc<Ess<T>>,
    store: RwLock<InMemoryKeyValueStore<EventId, T, Arc<Ess<T>>>>,
}

impl<T: Clone> StreamingStore<T> {
    pub fn new() -> Self {
        let ess = Arc::new(Ess::new());
        let store = RwLock::new(InMemoryKeyValueStore::new(Some(Arc::clone(&ess))));
        Self { ess, store }
    }
}

impl<T: Clone> Default for StreamingStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StreamingStore<T>
where
    T: Into<ProtoValue> + Clone + Send + Sync + 'static,
{
    /// Subscribes to the specified event identifiers and serves the
    /// subscriptions on detached tasks.
    pub fn serve(
        &self,
        client_id: impl Into<Box<str>>,
        requested_subscriptions: impl IntoIterator<Item = EventId>,
    ) -> Result<(), Status> {
        let subscriptions = self
            .ess
            .register_subscriptions(client_id.into(), requested_subscriptions)
            .map_err(|_| Status::failed_precondition("Channel not open"))?;

        for subscription in subscriptions {
            spawn(subscription.serve(move |(key, value), seq| {
                Ok(ProtoEvent {
                    source: key.to_string(),
                    value: Some(value.into()),
                    seq,
                    timestamp: Some(SystemTime::now().into()),
                })
            }));
        }

        Ok(())
    }

    /// Read a value from the store.
    pub fn get(&self, key: &EventId) -> Option<T> {
        self.store.read().unwrap().get(key).cloned()
    }

    /// Write a value to the store.
    pub fn set(&self, key: EventId, value: T) {
        self.store.write().unwrap().set(key, value)
    }
}

#[async_trait]
impl<T> ChannelService for StreamingStore<T>
where
    T: Clone + Send + Sync + 'static,
{
    type OpenStream = ReceiverStream<Result<ProtoEvent, Status>>;

    async fn open(
        &self,
        _: tonic::Request<OpenRequest>,
    ) -> Result<Response<Self::OpenStream>, Status> {
        let id: ClientId = uuid::Uuid::new_v4().to_string().into();
        let (_, receiver_stream) = self.ess.read_events(id.clone());
        let mut response = Response::new(receiver_stream);
        response.metadata_mut().insert(METADATA_KEY, id.to_string().try_into().unwrap());
        Ok(response)
    }
}

pub trait ProtoExt {
    fn subscribe(&self, subscribe_intent: SubscribeIntent) -> Result<Fulfillment, Status>;
    fn read(&self, intent: ReadIntent) -> Fulfillment;
}

impl<T> ProtoExt for StreamingStore<T>
where
    T: Into<ProtoValue> + Clone + Send + Sync + 'static,
{
    fn subscribe(&self, subscribe_intent: SubscribeIntent) -> Result<Fulfillment, Status> {
        self.serve(
            subscribe_intent.channel_id,
            subscribe_intent.sources.into_iter().map(|v| v.into()),
        )?;

        Ok(Fulfillment::Subscribe(SubscribeFulfillment {}))
    }

    fn read(&self, intent: ReadIntent) -> Fulfillment {
        let value = self.get(&intent.key.into());
        Fulfillment::Read(ReadFulfillment { value: value.map(|v| v.into()) })
    }
}
