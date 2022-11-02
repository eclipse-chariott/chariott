// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use chariott_common::{
    ess::SharedEss as InnerEss,
    proto::common::{fulfillment::Fulfillment, ReadFulfillment, ReadIntent, SubscribeIntent},
};
use keyvalue::{InMemoryKeyValueStore, Observer};
use std::sync::RwLock;
use tonic::Status;

use crate::chariott::proto::{common::value::Value as ProtoValue, common::Value as ValueMessage};

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
    /// Read a value from the store.
    pub fn get(&self, key: &EventId) -> Option<T> {
        self.store.read().unwrap().get(key).cloned()
    }

    /// Write a value to the store.
    pub fn set(&self, key: EventId, value: T) {
        self.store.write().unwrap().set(key, value)
    }
}

impl<T> AsRef<InnerEss<(EventId, T)>> for StreamingStore<T> {
    fn as_ref(&self) -> &InnerEss<(EventId, T)> {
        &self.ess.0
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
        let result = self.ess.as_ref().serve_subscriptions(subscribe_intent, |(_, v)| v.into())?;
        Ok(Fulfillment::Subscribe(result))
    }

    fn read(&self, intent: ReadIntent) -> Fulfillment {
        let value = self.get(&intent.key.into());
        Fulfillment::Read(ReadFulfillment {
            value: Some(ValueMessage { value: value.map(|v| v.into()) }),
        })
    }
}
