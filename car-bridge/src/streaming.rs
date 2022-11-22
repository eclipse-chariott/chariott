use std::collections::{HashMap, HashSet};

use chariott_common::error::Error;
use ess::{EventSubSystem, NotReadingEvents, Subscription as EssSubscription};
use examples_common::chariott::api::Event;

/// Identifies a namespace
type Namespace = String;
/// Identifies a topic
type Topic = String;
/// Identifies a subscription source (a.k.a. key or event ID).
type Source = String;

pub enum Action {
    Listen(Namespace),
    Subscribe(Namespace, Source),
    Link(Namespace, Topic),
}

/// Tracks the active subscriptions for each target.
pub struct Streaming {
    sources_by_namespace: HashMap<Namespace, HashSet<Source>>,
    links: HashSet<(Namespace, Topic)>,
}

impl Streaming {
    pub fn new() -> Self {
        Self { sources_by_namespace: HashMap::new(), links: HashSet::new() }
    }

    pub fn subscribe(&mut self, namespace: Namespace, source: Source, topic: Topic) -> Vec<Action> {
        let is_listening = self.sources_by_namespace.contains_key(&namespace);

        let is_subscribed = self
            .sources_by_namespace
            .get(&namespace)
            .and_then(|sources| sources.get(&source))
            .is_some();

        let link = (namespace.clone(), topic.clone());
        let is_linked = self.links.contains(&link);

        match (is_listening, is_subscribed, is_linked) {
            (false, _, _) => {
                self.sources_by_namespace
                    .insert(namespace.clone(), HashSet::from([source.clone()]));

                self.links.insert(link);

                vec![
                    Action::Listen(namespace.clone()),
                    Action::Subscribe(namespace.clone(), source),
                    Action::Link(namespace, topic),
                ]
            }
            (true, false, false) => {
                if let Some(sources) = self.sources_by_namespace.get_mut(&namespace) {
                    sources.insert(source.clone());
                };

                self.links.insert(link);

                vec![Action::Subscribe(namespace.clone(), source), Action::Link(namespace, topic)]
            }
            (true, true, false) => {
                self.links.insert(link);

                vec![Action::Link(namespace, topic)]
            }
            _ => vec![],
        }
    }
}

type Ess = EventSubSystem<String, String, Event, Event>;

pub struct SubscriptionRegistry {
    ess_by_namespace: HashMap<Namespace, Ess>,
    // channel ID be namspace
    channel_id_by_namespace: HashMap<Namespace, String>,
}

impl SubscriptionRegistry {
    pub fn new() -> Self {
        Self { ess_by_namespace: HashMap::new(), channel_id_by_namespace: HashMap::new() }
    }

    pub fn set_channel_id(&mut self, namespace: Namespace, channel_id: String) {
        self.channel_id_by_namespace.insert(namespace, channel_id);
    }

    pub fn channel_id(&mut self, namespace: &Namespace) -> Option<&String> {
        self.channel_id_by_namespace.get(namespace)
    }

    pub fn ess(&self, namespace: &str) -> &Ess {
        self.ess_by_namespace.get(namespace).unwrap()
    }

    pub fn subscribe(
        &mut self,
        namespace: Namespace,
        source: String,
        client_id: String,
    ) -> Result<EssSubscription<String, String, Event, Event>, NotReadingEvents> {
        let ess = self.ess_by_namespace.entry(namespace).or_insert_with(|| EventSubSystem::new());
        let subscriptions = ess.register_subscriptions(client_id, vec![source])?;
        Ok(subscriptions.into_iter().next().unwrap())
    }

    pub fn publish(&self, namespace: &str, source: &str, payload: Event) -> Result<(), Error> {
        let ess = self.ess_by_namespace.get(namespace).ok_or_else(|| {
            Error::new(format!("Could not find an ESS for namespace '{namespace}'."))
        })?;
        ess.publish(source, payload);
        Ok(())
    }
}
