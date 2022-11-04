// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use chariott_common::{
    proto::common::{fulfillment::Fulfillment, ReadFulfillment, ReadIntent, SubscribeIntent},
    streaming_ess::StreamingEss,
};
use keyvalue::{InMemoryKeyValueStore, Observer};
use std::sync::RwLock;
use tonic::Status;

use crate::chariott::proto::{common::value::Value as ProtoValue, common::Value as ValueMessage};

type EventId = Box<str>;

/// Wrapper around the [`StreamingEss`](StreamingEss) to allow implementing the
/// `Observer` trait for said type.
#[derive(Clone)]
struct InternalStreamingEss<T>(StreamingEss<(EventId, T)>);

impl<T: Clone + Send + 'static> Observer<EventId, T> for InternalStreamingEss<T> {
    fn on_set(&mut self, key: &EventId, value: &T) {
        self.0.publish(key, (key.clone(), value.clone()));
    }
}

/// Represents an in-memory store that contains a blanket implementation for
/// integration with the Chariott streaming API. It generalizes over any type of
/// value to be published, as long as that value can be transformed into a value
/// which is compatible with the Proto contract.
pub struct StreamingStore<T> {
    ess: InternalStreamingEss<T>,
    store: RwLock<InMemoryKeyValueStore<EventId, T, InternalStreamingEss<T>>>,
}

impl<T> StreamingStore<T> {
    pub fn ess(&self) -> &StreamingEss<(EventId, T)> {
        &self.ess.0
    }
}

impl<T: Clone + Send + 'static> StreamingStore<T> {
    pub fn new() -> Self {
        let ess = InternalStreamingEss(StreamingEss::new());
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
    /// Read a value from the store.
    pub fn get(&self, key: &EventId) -> Option<T> {
        self.store.read().unwrap().get(key).cloned()
    }

    /// Write a value to the store.
    pub fn set(&self, key: EventId, value: T) {
        self.store.write().unwrap().set(key, value)
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
        let result = self.ess().serve_subscriptions(subscribe_intent, |(_, v)| v.into())?;
        Ok(Fulfillment::Subscribe(result))
    }

    fn read(&self, intent: ReadIntent) -> Fulfillment {
        let value = self.get(&intent.key.into());
        Fulfillment::Read(ReadFulfillment {
            value: Some(ValueMessage { value: value.map(|v| v.into()) }),
        })
    }
}
