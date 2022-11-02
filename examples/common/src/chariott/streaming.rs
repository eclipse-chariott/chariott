// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use async_trait::async_trait;
use chariott_common::{
    ess::Ess as InnerEss,
    proto::common::{
        fulfillment::Fulfillment, ReadFulfillment, ReadIntent, SubscribeFulfillment,
        SubscribeIntent,
    },
};
use keyvalue::{InMemoryKeyValueStore, Observer};
use std::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};

use crate::chariott::proto::{
    common::value::Value as ProtoValue,
    common::Value as ValueMessage,
    streaming::{channel_service_server::ChannelService, Event as ProtoEvent, OpenRequest},
};

type EventId = Box<str>;

#[derive(Clone)]
struct Ess<T>(InnerEss<(EventId, T)>);

impl<T: Clone + Send + 'static> Observer<EventId, T> for Ess<T> {
    fn on_set(&mut self, key: &EventId, value: &T) {
        self.0.as_ref().publish(key, (key.clone(), value.clone()));
    }
}

impl<T> AsRef<InnerEss<(EventId, T)>> for Ess<T> {
    fn as_ref(&self) -> &InnerEss<(EventId, T)> {
        &self.0
    }
}

/// Represents an in-memory store that contains a blanket implementation for
/// integration with the Chariott streaming API. It generalizes over any type of
/// value to be published, as long as that value can be transformed into a value
/// which is compatible with the Proto contract.
pub struct StreamingStore<T> {
    ess: Ess<T>,
    store: RwLock<InMemoryKeyValueStore<EventId, T, Ess<T>>>,
}

impl<T: Clone + Send + 'static> StreamingStore<T> {
    pub fn new() -> Self {
        let ess = Ess(InnerEss::new());
        let store = RwLock::new(InMemoryKeyValueStore::new(Some(ess.clone())));
        Self { ess, store }
    }
}

impl<T: Clone + Send + 'static> Default for StreamingStore<T> {
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
        self.ess.as_ref().serve_subscriptions(client_id, requested_subscriptions, |(_, v)| v.into())
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
        request: tonic::Request<OpenRequest>,
    ) -> Result<Response<Self::OpenStream>, Status> {
        self.ess.as_ref().open(request).await
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
        Fulfillment::Read(ReadFulfillment {
            value: Some(ValueMessage { value: value.map(|v| v.into()) }),
        })
    }
}
