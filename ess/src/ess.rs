// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
#[cfg(test)]
use tests::{mpsc, ReceiverStream};
#[cfg(not(test))]
use tokio::sync::mpsc;
#[cfg(not(test))]
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

use tokio::sync::broadcast;

/// Represents the result of an upsert opertion, indicating whether the result
/// ended up inserting a new entry or updating an existing entry.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum UpsertResult {
    Inserted,
    Updated,
}

/// Represents the (error) status that a client is not reading events,
/// when adding or removing a subscription.
///
/// This usually means that [`EventSubSystem<ClientId, EventId, Event,
/// ClientEvent>::read_events`] has not been called for a client.
#[derive(Debug, Eq, PartialEq)]
pub struct NotReadingEvents;

// Represents a single client with one ore more subscriptions.
struct Client<EventId, ClientEvent> {
    sender: mpsc::Sender<ClientEvent>,
    subscriptions: HashMap<EventId, CancellationToken>,
}

/// Default size of the buffer for publishing events to all subscriptions.
pub const DEFAULT_PUBLISH_BUFFER_SIZE: usize = 10;

/// Default size of the buffer for delivering events to a client.
pub const DEFAULT_CLIENT_BUFFER_SIZE: usize = 200;

/// Represents the configuration for the event sub-system, such as the sizes
/// of the pub-sub channels.
#[derive(Clone, Debug)]
pub struct Config {
    publish_buffer_size: usize,
    client_buffer_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            publish_buffer_size: DEFAULT_PUBLISH_BUFFER_SIZE,
            client_buffer_size: DEFAULT_CLIENT_BUFFER_SIZE,
        }
    }
}

impl Config {
    /// Sets the size of the channel used to publish events to all
    /// subscriptions.
    pub fn set_publish_buffer_size(&mut self, value: usize) -> &mut Self {
        self.publish_buffer_size = value;
        self
    }

    /// Sets the size of the channel used to deliver events to a client.
    pub fn set_client_buffer_size(&mut self, value: usize) -> &mut Self {
        self.client_buffer_size = value;
        self
    }
}

/// Implementation of an eventing/pub-sub system that can be used to publish
/// events and register subscriptions from multiple clients for multiple
/// events.
///
/// # Type Arguments
///
/// - `ClientId`: An identifier representing a client.
/// - `EventId`: An identifier representing an event type.
/// - `Event`: The type of the _published_ event.
/// - `ClientEvent`: The type of the event delivered to the client.
#[derive(Default)]
pub struct EventSubSystem<ClientId, EventId, Event, ClientEvent> {
    config: Config,
    sender_by_event_id: Arc<RwLock<HashMap<EventId, broadcast::Sender<Event>>>>,
    client_by_id: Arc<RwLock<HashMap<ClientId, Client<EventId, ClientEvent>>>>,
}

impl<ClientId, EventId, Event, ClientEvent> EventSubSystem<ClientId, EventId, Event, ClientEvent>
where
    ClientId: Clone + Eq + Hash,
    EventId: Clone + Eq + Hash,
    Event: Clone,
{
    /// Initializes the event sub-system with no subscriptions.
    pub fn new() -> Self {
        Self {
            config: Default::default(),
            sender_by_event_id: Default::default(),
            client_by_id: Default::default(),
        }
    }

    /// Initializes the event sub-system with no subscriptions.
    pub fn new_with_config(config: Config) -> Self {
        Self { config, sender_by_event_id: Default::default(), client_by_id: Default::default() }
    }

    /// Publishes an event instance for an event type. Returns a Boolean
    /// indicating whether the event was published to _at least_ one active
    /// subscription.
    pub fn publish<Q>(&self, event_id: &Q, event: Event) -> bool
    where
        EventId: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(sender) = self.sender_by_event_id.read().unwrap().get(event_id) {
            // Ignore send errors, which can only occur if there are no receivers.
            _ = sender.send(event);
            true
        } else {
            false
        }
    }

    /// Registers a client for reading events and returns a stream on which
    /// the events will be delivered.
    ///
    /// This call must be made before any subscriptions can be registered by
    /// the client.
    ///
    /// Note that if the client abandons the stream returned then housekeeping
    /// of associated state is not done until the next attempt to deliver to
    /// the client.
    pub fn read_events(&self, client_id: ClientId) -> (UpsertResult, ReceiverStream<ClientEvent>) {
        let (tx, rx) = mpsc::channel::<ClientEvent>(self.config.client_buffer_size);
        let mut client_by_id = self.client_by_id.write().unwrap();
        let upsert = if client_by_id
            .insert(client_id, Client { sender: tx, subscriptions: HashMap::new() })
            .is_some()
        {
            UpsertResult::Updated
        } else {
            UpsertResult::Inserted
        };
        (upsert, ReceiverStream::new(rx))
    }

    /// Registers one or more subscriptions for a client and returns a
    /// sequence of subscriptions in the same order as the requested
    /// subscriptions.
    ///
    /// In order for the subscriptions to be _served_ (meaning for theirs events
    /// to be delivered), the caller must call [`Subscription<ClientId,
    /// EventId, Event, ClientEvent>::serve`] on each of the returned
    /// subscriptions.
    ///
    /// If [`Self::read_events`] has not been called for the client prior
    /// to subscription registration then an error of type
    /// [`NotReadingEvents`] is returned.
    pub fn register_subscriptions(
        &self,
        client_id: ClientId,
        requested_subscriptions: impl IntoIterator<Item = EventId>,
    ) -> Result<
        impl IntoIterator<Item = Subscription<ClientId, EventId, Event, ClientEvent>>,
        NotReadingEvents,
    > {
        let mut client_by_id = self.client_by_id.write().unwrap();

        let client = client_by_id.get_mut(&client_id).ok_or(NotReadingEvents)?;

        let mut new_subscriptions = Vec::new();

        let subscriptions = &mut client.subscriptions;

        for event_id in requested_subscriptions {
            if subscriptions.contains_key(&event_id) {
                continue; // already subscribed
            }

            let receiver = {
                let mut sender_by_event_id = self.sender_by_event_id.write().unwrap();

                sender_by_event_id
                    .entry(event_id.clone())
                    .or_insert_with(|| {
                        let (sender, _) = broadcast::channel(self.config.publish_buffer_size);
                        sender
                    })
                    .subscribe()
            };

            let subscription_cancellation_token = CancellationToken::new();
            subscriptions.insert(event_id.clone(), subscription_cancellation_token.clone());

            new_subscriptions.push(Subscription {
                id: SubscriptionId { client_id: client_id.clone(), event_id },
                cancellation_token: subscription_cancellation_token,
                receiver,
                sender: client.sender.clone(),
                client_by_id: Arc::clone(&self.client_by_id),
            });
        }

        Ok(new_subscriptions)
    }

    /// Returns the identifiers of the events to which the given client has
    /// subscriptions.
    pub fn get_subscriptions<Q>(&self, client_id: &Q) -> impl IntoIterator<Item = EventId>
    where
        ClientId: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let client_by_id = self.client_by_id.read().unwrap();
        match client_by_id.get(client_id) {
            Some(client) => client.subscriptions.iter().map(|(id, _)| id.clone()).collect(),
            None => vec![],
        }
    }
}

impl<ClientId, EventId, Event, ClientEvent> EventSubSystem<ClientId, EventId, Event, ClientEvent>
where
    ClientId: Clone + Eq + Hash,
    EventId: Clone + Display + Eq + Hash,
    Event: Clone,
{
    /// Deregisters one or more subscriptions for a client.
    ///
    /// If [`Self::read_events`] has not been called for the client prior to
    /// subscription registration then an error of type [`NotReadingEvents`]
    /// is returned.
    pub fn deregister_subscriptions<Q>(
        &self,
        client_id: &Q,
        event_ids: impl IntoIterator<Item = EventId>,
    ) -> Result<(), NotReadingEvents>
    where
        ClientId: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut client_by_id = self.client_by_id.write().unwrap();
        let client = client_by_id.get_mut(client_id).ok_or(NotReadingEvents)?;
        let subscriptions = &mut client.subscriptions;
        for id in event_ids {
            let succeeded = if let Some(cancellation_token) = subscriptions.remove(&id) {
                cancellation_token.cancel();
                let mut senders = self.sender_by_event_id.write().unwrap();
                if senders.get(&id).map(|s| s.receiver_count()) == Some(0) {
                    senders.remove(&id);
                }
                true
            } else {
                false
            };
            tracing::debug!("Deregistration for \"{}\": {}", id, succeeded);
        }
        Ok(())
    }
}

/// Represents an identifier for a single and unique event subscription.
pub struct SubscriptionId<ClientId, EventId> {
    client_id: ClientId,
    event_id: EventId,
}

impl<ClientId, EventId> SubscriptionId<ClientId, EventId> {
    /// The identifier of subscription's client.
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    /// The identifier of subscription's event type.
    pub fn event_id(&self) -> &EventId {
        &self.event_id
    }
}

impl<ClientId, EventId> Display for SubscriptionId<ClientId, EventId>
where
    ClientId: Display,
    EventId: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.client_id(), self.event_id()))
    }
}

/// Represents a single and unique event subscription.
pub struct Subscription<ClientId, EventId, Event, ClientEvent> {
    id: SubscriptionId<ClientId, EventId>,
    cancellation_token: CancellationToken,
    receiver: broadcast::Receiver<Event>,
    sender: mpsc::Sender<ClientEvent>,
    client_by_id: Arc<RwLock<HashMap<ClientId, self::Client<EventId, ClientEvent>>>>,
}

impl<ClientId, EventId, Event, ClientEvent> Subscription<ClientId, EventId, Event, ClientEvent> {
    /// Returns the event identifier.
    pub fn event_id(&self) -> &EventId {
        self.id.event_id()
    }
}

impl<ClientId, EventId, Event, ClientEvent> Subscription<ClientId, EventId, Event, ClientEvent>
where
    Event: Clone,
    ClientId: Display + Eq + Hash,
    EventId: Display + Eq + Hash,
{
    /// Returns a future that, when spawned, serves the subscription. The
    /// future remains pending until the subscription terminates due to either
    /// deregistration, client disconnection or client abandonment.
    ///
    /// The supplied closure `f` will receive the published event and the
    /// event sequence number (monotonically increasing number from 1 that is
    /// local to the subscription) and it must return the client event to be
    /// delivered.
    pub fn serve(
        self,
        f: impl Fn(Event, u64) -> ClientEvent,
    ) -> impl std::future::Future<Output = ()> {
        use tracing::*;

        self.serve_with_handlers(
            f,
            // on_subscription_revoked:
            Some(|id: &SubscriptionId<ClientId, EventId>| {
                debug!("Subscription \"{id}\" is revoked.");
            }),
            // on_client_disconnected:
            Some(|id: &SubscriptionId<ClientId, EventId>| {
                warn!("Receiver of subscription \"{id}\" is closed.");
            }),
            // on_done
            Some(|id: &SubscriptionId<ClientId, EventId>| {
                debug!("Task serving subscription \"{id}\" ended.");
            }),
            // on_client_abandoned:
            Some(|id: &SubscriptionId<ClientId, EventId>, _| {
                debug!("Removing subscription \"{id}\" because its channel is closed.");
            }),
            // on_event_dropped:
            Some(|id: &SubscriptionId<ClientId, EventId>, _| {
                warn!("Dropped event of subscription \"{id}\" because the channel buffer is full.");
            }),
            // on_publisher_lagged:
            Some(|id: &SubscriptionId<ClientId, EventId>, amount| {
                warn!("Receiver of subscription \"{id}\" lagged by {amount}.");
            }),
        )
    }
}

impl<ClientId, EventId, Event, ClientEvent> Subscription<ClientId, EventId, Event, ClientEvent>
where
    Event: Clone,
    ClientId: Eq + Hash,
    EventId: Eq + Hash,
{
    #[allow(clippy::too_many_arguments)] // TODO address too many arguments
    async fn serve_with_handlers(
        mut self,
        f: impl Fn(Event, u64) -> ClientEvent,
        on_subscription_revoked: Option<impl Fn(&SubscriptionId<ClientId, EventId>)>,
        on_client_disconnected: Option<impl Fn(&SubscriptionId<ClientId, EventId>)>,
        on_done: Option<impl Fn(&SubscriptionId<ClientId, EventId>)>,
        on_client_abandoned: Option<impl Fn(&SubscriptionId<ClientId, EventId>, ClientEvent)>,
        on_event_dropped: Option<impl Fn(&SubscriptionId<ClientId, EventId>, ClientEvent)>,
        on_publisher_lagged: Option<impl Fn(&SubscriptionId<ClientId, EventId>, u64)>,
    ) {
        let mut seq = 0_u64;
        loop {
            let rx = &mut self.receiver;
            tokio::select! {
                _ = self.cancellation_token.cancelled() => {
                    if let Some(ref on_subscription_revoked) = on_subscription_revoked {
                        on_subscription_revoked(&self.id);
                    }
                    break;
                }
                event = rx.recv() => {
                    use tokio::sync::broadcast::error::RecvError;
                    use tokio::sync::mpsc::error::TrySendError;

                    match event {
                        Ok(event) => {
                            seq += 1;
                            match self.sender.try_send(f(event, seq)) {
                                Ok(_) => continue,
                                Err(TrySendError::Full(event)) => {
                                    if let Some(ref on_event_dropped) = on_event_dropped {
                                        on_event_dropped(&self.id, event);
                                    }
                                    continue;
                                }
                                Err(TrySendError::Closed(event)) => {
                                    if let Some(ref on_client_abandoned) = on_client_abandoned {
                                        on_client_abandoned(&self.id, event);
                                    }
                                    let mut client_by_id = self.client_by_id.write().unwrap();
                                    if let Some(client) = client_by_id.get_mut(self.id.client_id()) {
                                        client.subscriptions.remove(self.id.event_id());
                                    }
                                    break;
                                }
                            }
                        }
                        Err(RecvError::Closed) => {
                            if let Some(ref on_client_abandoned) = on_client_disconnected {
                                on_client_abandoned(&self.id);
                            };
                            break;
                        }
                        Err(RecvError::Lagged(amount)) => {
                            seq = seq.wrapping_add(amount);
                            if let Some(ref on_publisher_lagged) = on_publisher_lagged {
                                on_publisher_lagged(&self.id, amount);
                            }
                        }
                    }
                }
            }
        }
        if let Some(ref on_done) = on_done {
            on_done(&self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{EventSubSystem, UpsertResult};
    use chariott_common::tokio_runtime_fork;
    use std::time::Duration;

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    struct ClientId(&'static str);

    impl std::fmt::Display for ClientId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.0)
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    enum EventId {
        Foo,
    }

    impl std::fmt::Display for EventId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                EventId::Foo => f.write_str("Foo"),
            }
        }
    }

    #[derive(Debug, Clone)]
    struct SeqNum(u64);

    #[derive(Clone)]
    struct Event(EventId, SeqNum, &'static str);

    type Ess = EventSubSystem<ClientId, EventId, Event, Event>;

    impl super::Client<EventId, Event> {
        pub fn read_event(&self) -> Result<Event, ()> {
            self.sender.dequeue_event()
        }
    }

    pub struct ReceiverStream<T> {
        unused: std::marker::PhantomData<T>,
    }

    impl<T> ReceiverStream<T> {
        pub(crate) fn new(_receiver: ()) -> Self {
            Self { unused: std::marker::PhantomData }
        }
    }

    impl<T> futures::Stream for ReceiverStream<T> {
        type Item = T;

        fn poll_next(
            self: std::pin::Pin<&mut Self>,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            unimplemented!();
        }
    }

    pub(crate) mod mpsc {
        use std::sync::Arc;

        pub(crate) struct Sender<T> {
            events: Arc<std::sync::Mutex<Vec<T>>>,
        }

        impl<T> Sender<T> {
            pub fn try_send(&self, t: T) -> Result<(), tokio::sync::mpsc::error::TrySendError<T>> {
                self.events.lock().unwrap().push(t);
                Ok(())
            }
        }

        impl<T> Sender<T> {
            pub fn dequeue_event(&self) -> Result<T, ()> {
                let mut events = self.events.lock().unwrap();
                if events.is_empty() {
                    return Err(());
                }
                Ok(events.remove(0))
            }
        }

        impl<T> Clone for Sender<T> {
            fn clone(&self) -> Self {
                Self { events: Arc::clone(&self.events) }
            }
        }

        pub(crate) fn channel<T>(_buffer: usize) -> (Sender<T>, ()) {
            let tx = Sender::<T> { events: Arc::new(std::sync::Mutex::new(vec![])) };
            (tx, ())
        }
    }

    trait TestClient {
        fn read_event(&self, client_id: &ClientId) -> Option<Event>;
    }

    impl TestClient for Ess {
        fn read_event(&self, client_id: &ClientId) -> Option<Event> {
            let client_by_id = self.client_by_id.write().unwrap();
            client_by_id.get(client_id).and_then(|c| c.read_event().ok())
        }
    }

    fn sut() -> Ess {
        sut_with_runtime().0
    }

    fn sut_with_runtime() -> (Ess, tokio_runtime_fork::Fork) {
        use tokio_runtime_fork::BuilderExt;
        let runtime_fork =
            tokio::runtime::Builder::new_multi_thread().worker_threads(1).fork().unwrap();
        (Ess::new(), runtime_fork)
    }

    #[test]
    fn read_events_returns_new_stream_on_each_call_for_same_client() {
        // arrange
        const CLIENT_ID: &ClientId = &ClientId("client");
        let sut = sut();
        // act
        let (upsert1, stream1) = sut.read_events(CLIENT_ID.clone());
        let (upsert2, stream2) = sut.read_events(CLIENT_ID.clone());
        // assert
        assert_eq!(UpsertResult::Inserted, upsert1);
        assert_eq!(UpsertResult::Updated, upsert2);
        assert!(!std::ptr::eq(&stream1, &stream2));
    }

    #[test]
    fn read_events_streams_event_on_update() {
        // arrange
        const EVENT_ID: EventId = EventId::Foo;
        const DATA1: &str = "data1";
        const DATA2: &str = "data2";
        const CLIENT1: ClientId = ClientId("client1");
        const CLIENT2: ClientId = ClientId("client2");
        let (sut, runtime_fork) = sut_with_runtime();
        for client_id in [CLIENT1, CLIENT2] {
            _ = sut.read_events(client_id);
        }
        let subscriptions = sut.register_subscriptions(CLIENT1, [EVENT_ID]).unwrap();
        for subscription in subscriptions {
            runtime_fork
                .handle()
                .spawn(subscription.serve(|Event(id, _, data), seq| Event(id, SeqNum(seq), data)));
        }
        // act
        sut.publish(&EVENT_ID, Event(EVENT_ID, SeqNum(0), DATA1));
        sut.publish(&EVENT_ID, Event(EVENT_ID, SeqNum(0), DATA2));
        // TODO investigate how to avoid sleeping here
        // see also: https://github.com/tokio-rs/tokio/issues/2443
        std::thread::sleep(Duration::from_secs_f64(0.1));
        // assert
        assert!(TestClient::read_event(&sut, &CLIENT2).is_none());
        let event = TestClient::read_event(&sut, &CLIENT1).unwrap();
        let Event(id, SeqNum(seq), data) = event;
        assert_eq!(EVENT_ID, id);
        assert_eq!(1, seq);
        assert_eq!(DATA1, data);
        let event = TestClient::read_event(&sut, &CLIENT1).unwrap();
        let Event(id, SeqNum(seq), data) = event;
        assert_eq!(EVENT_ID, id);
        assert_eq!(2, seq);
        assert_eq!(DATA2, data);
        assert!(TestClient::read_event(&sut, &CLIENT1).is_none());
        drop(runtime_fork); // not needed but helps to avoid marking "runtime_fork" as unused
    }

    #[test]
    fn read_events_does_not_stream_events_of_unregistered_subscriptions() {
        // arrange
        let sut = sut();
        let client_id = ClientId("client");
        _ = sut.read_events(client_id.clone());
        // act
        let published = sut.publish(&EventId::Foo, Event(EventId::Foo, SeqNum(0), "data"));
        let event = TestClient::read_event(&sut, &client_id);
        // assert
        assert!(!published);
        assert!(event.is_none());
    }

    #[tokio::test]
    async fn deregistered_subscriptions_terminates_subscription_server() {
        // arrange
        const CLIENT_ID: &ClientId = &ClientId("client");
        let (sut, runtime_fork) = sut_with_runtime();
        _ = sut.read_events(CLIENT_ID.clone());
        let mut subscriptions =
            sut.register_subscriptions(CLIENT_ID.clone(), [EventId::Foo]).unwrap().into_iter();
        let subscription = subscriptions.next().unwrap();
        let subscription_server = runtime_fork
            .handle()
            .spawn(subscription.serve(|Event(id, _, data), seq| Event(id, SeqNum(seq), data)));
        // act
        sut.deregister_subscriptions(CLIENT_ID, [EventId::Foo]).unwrap();
        // assert
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                panic!("Subscription server should have terminated shortly after deregistration of the subscription!")
            }
            result = subscription_server => {
                assert!(result.is_ok());
            }
        }
        drop(runtime_fork); // not needed but helps to avoid marking "runtime_fork" as unused
    }

    #[test]
    fn register_subscriptions_cannot_be_called_if_events_are_not_being_read() {
        // arrange
        let sut = sut();
        // act
        let result = sut.register_subscriptions(ClientId("client"), vec![]);
        // assert
        match result {
            Ok(_) => panic!("Expected error, got okay."),
            Err(super::NotReadingEvents) => {}
        };
    }

    #[test]
    fn deregister_subscriptions_cannot_be_called_if_events_are_not_being_read() {
        // arrange
        let sut = sut();
        // act
        let result = sut.deregister_subscriptions(&ClientId("client"), vec![]);
        // assert
        assert!(result.is_err());
        assert_eq!(super::NotReadingEvents, result.unwrap_err());
    }

    #[test]
    fn get_subscriptions_returns_empty_list_when_no_subscriptions_registered() {
        // arrange
        let sut = sut();
        // act
        let result = sut.get_subscriptions(&ClientId("client"));
        // assert
        assert_eq!(None, result.into_iter().next());
    }
}
