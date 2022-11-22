use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
};

use chariott_common::{chariott_api::ChariottCommunication, error::Error};
use chariott_proto::streaming::Event;
use ess::{EventSubSystem, NotReadingEvents, Subscription as EssSubscription};
use examples_common::chariott::api::ChariottCommunicationExt as _;
use futures::Stream;
use tokio::spawn;
use tokio_stream::StreamExt as _;
use tracing::warn;

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
    pub fn next_subscribe_action(
        &mut self,
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
        let link = (namespace.clone(), topic.clone());
        if !self.links.contains(&link) {
            let (namespace, topic) = link;
            return Some(Action::Link(namespace, topic));
        }

        // Check if routed
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

        Ok(Self { ess, channel_id })
    }

    /// Links a certain topic to events coming from this provider. By calling
    /// `route`, all events with a given identifier are routed to that topic.
    pub fn link(&self, topic: Topic) -> impl Stream<Item = Event> {
        let (_, topic_stream) = self.ess.read_events(topic);
        topic_stream
    }

    /// Routes a certain event from this provider to a topic. This returns a
    /// subscription that can be served to establish the route. Depends on a
    /// `link` to be present.
    pub fn route(
        &self,
        topic: Topic,
        source: Source,
    ) -> Result<EssSubscription<Topic, Source, Event, Event>, NotReadingEvents> {
        let subscriptions = self.ess.register_subscriptions(topic, vec![source])?;
        Ok(subscriptions.into_iter().next().unwrap())
    }

    /// Gets the channel ID for the gRPC connection between the provider and
    /// this component.
    pub fn channel_id(&self) -> &str {
        &self.channel_id
    }
}
