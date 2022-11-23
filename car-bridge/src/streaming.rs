use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
};

use chariott_common::{chariott_api::ChariottCommunication, error::Error};
use chariott_proto::streaming::Event;
use ess::{EventSubSystem, NotReadingEvents};
use examples_common::chariott::api::{Chariott, ChariottCommunicationExt as _};
use paho_mqtt::QOS_1;
use prost::Message as _;
use tokio::spawn;
use tokio_stream::StreamExt as _;
use tracing::{debug, warn};

use crate::{Message, Metadata, ResponseSender};

/// Identifies a namespace
type Namespace = String;
/// Identifies a topic
type Topic = String;
/// Identifies a subscription source (a.k.a. key or event ID).
type Source = String;

#[derive(Clone)]
pub enum Action {
    /// Open a connection a provider's streaming endpoint.
    Listen(Namespace),
    /// Subscribe to a certain type of event from a provider.
    Subscribe(Namespace, Source),
    /// Link a provider to a topic.
    Link(Namespace, Topic),
    /// Subscribe to an event for a topic link.
    Route(Namespace, Topic, Source),
}

/// Tracks the active subscriptions for each namespace and allows calculating
/// which steps to take to support streaming intents from a certain provider to
/// a topic based on the current state.
pub struct SubscriptionState {
    sources_by_namespace: HashMap<Namespace, HashSet<Source>>,
    links: HashSet<(Namespace, Topic)>,
    routes: HashSet<(Namespace, Topic, Source)>,
}

impl SubscriptionState {
    pub fn new() -> Self {
        Self { sources_by_namespace: HashMap::new(), links: HashSet::new(), routes: HashSet::new() }
    }

    /// Commits an action in case it was executed successfully.
    pub fn commit(&mut self, action: Action) {
        match action {
            Action::Listen(namespace) => {
                self.sources_by_namespace.insert(namespace.clone(), HashSet::new());
            }
            Action::Subscribe(namespace, source) => {
                if let Some(sources) = self.sources_by_namespace.get_mut(&namespace) {
                    sources.insert(source.clone());
                };
            }
            Action::Link(namespace, topic) => {
                self.links.insert((namespace, topic));
            }
            Action::Route(namespace, topic, source) => {
                self.routes.insert((namespace, topic, source));
            }
        };
    }

    /// Calculates the next action to take based on the current subscription
    /// state.
    pub fn next_action(
        &self,
        namespace: Namespace,
        source: Source,
        topic: Topic,
    ) -> Option<Action> {
        // Check if listening
        if !self.sources_by_namespace.contains_key(&namespace) {
            return Some(Action::Listen(namespace.clone()));
        }

        // Check if subscribed
        if self
            .sources_by_namespace
            .get(&namespace)
            .and_then(|sources| sources.get(&source))
            .is_none()
        {
            return Some(Action::Subscribe(namespace.clone(), source.clone()));
        }

        // Check if linked
        let link = (namespace, topic);
        if !self.links.contains(&link) {
            let (namespace, topic) = link;
            return Some(Action::Link(namespace, topic));
        }

        // Check if routed
        let (namespace, topic) = link;
        let route = (namespace, topic, source);
        if !self.routes.contains(&route) {
            let (namespace, topic, source) = route;
            return Some(Action::Route(namespace, topic, source));
        }

        None
    }
}

type Ess = EventSubSystem<Topic, Source, Event, Event>;

/// Tracks `EventProvider` instances by namespace.
pub struct ProviderRegistry {
    event_provider_by_namespace: HashMap<Namespace, Provider>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self { event_provider_by_namespace: HashMap::new() }
    }

    pub async fn register_event_provider(
        &mut self,
        chariott: &mut impl ChariottCommunication,
        namespace: Namespace,
    ) -> Result<(), Error> {
        if let Entry::Vacant(e) = self.event_provider_by_namespace.entry(namespace.clone()) {
            let event_provider = Provider::listen(chariott, namespace).await?;
            e.insert(event_provider);
        }

        Ok(())
    }

    pub fn get_event_provider(&self, namespace: &Namespace) -> Option<&Provider> {
        self.event_provider_by_namespace.get(namespace)
    }

    pub fn get_event_provider_mut(&mut self, namespace: &Namespace) -> Option<&mut Provider> {
        self.event_provider_by_namespace.get_mut(namespace)
    }
}

/// Represents events coming from a given provider, identified by its namespace.
/// The components allows distributing events coming from the provider to
/// multiple consumers.
pub struct Provider {
    channel_id: String,
    namespace: Namespace,
    ess: Arc<Ess>,
}

impl Provider {
    /// Create a new instance by starting to listen to events coming from a
    /// provider.
    pub async fn listen(
        chariott: &mut impl ChariottCommunication,
        namespace: Namespace,
    ) -> Result<Self, Error> {
        let (mut stream, channel_id) = chariott.open(namespace.clone()).await?;

        let ess = Arc::new(Ess::new());

        {
            // Publish all subscribed values from a provider
            // into its ESS.

            let namespace = namespace.clone();
            let ess = Arc::clone(&ess);

            spawn(async move {
                while let Some(event) = stream.next().await {
                    let event = event.unwrap();
                    ess.publish(event.source.clone().as_str(), event);
                }

                warn!("Stream for channel '{namespace}' broke.");
            });
        }

        Ok(Self { ess, namespace, channel_id })
    }

    /// Links a certain topic to events coming from this provider. By calling
    /// `route`, all events with a given identifier are routed to that topic.
    pub fn link(&self, topic: Topic, response_sender: ResponseSender) {
        let (_, mut topic_stream) = self.ess.read_events(topic.clone());

        spawn(async move {
            while let Some(e) = topic_stream.next().await {
                let mut buffer = vec![];

                if let Err(err) = e.encode(&mut buffer) {
                    debug!("Failed to encode event: '{err:?}'.");
                }

                response_sender
                    .send(Message::Default(
                        buffer,
                        topic.clone(),
                        Metadata {
                            content_type: "application/x-proto+chariott.streaming.v1.Event",
                            qos: QOS_1,
                        },
                    ))
                    .await;
            }

            warn!("Stream for topic '{topic}' broke.");
        });
    }

    /// Routes a certain event from this provider to a topic. This returns a
    /// subscription that can be served to establish the route. Depends on a
    /// `link` to be present.
    pub fn route(&self, topic: Topic, source: Source) -> Result<(), NotReadingEvents> {
        let subscriptions = self.ess.register_subscriptions(topic, vec![source])?;
        spawn(subscriptions.into_iter().next().unwrap().serve(|e, _| e));
        Ok(())
    }

    /// Subscribes to a certain kind of event from the provider.
    pub async fn subscribe(
        &self,
        chariott: &mut impl ChariottCommunication,
        source: Source,
    ) -> Result<(), Error> {
        chariott
            .subscribe(self.namespace.clone(), self.channel_id.clone(), vec![source.into()])
            .await
    }
}
