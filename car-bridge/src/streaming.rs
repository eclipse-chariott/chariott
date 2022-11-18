use std::collections::{HashMap, HashSet};

type Target = String;

#[derive(PartialEq, Eq, Hash)]
pub struct Subscription {
    key: String,
    subscriber: String,
}

impl Subscription {
    pub fn new(key: String, subscriber: String) -> Self {
        Self { key, subscriber }
    }
}

pub enum Action {
    Listen,
    Subscribe(String),
}

/// Tracks the active subscriptions for each target.
pub struct Streaming {
    subscriptions_by_target: HashMap<Target, HashSet<Subscription>>,
}

impl Streaming {
    pub fn new() -> Self {
        Self { subscriptions_by_target: HashMap::new() }
    }

    pub fn subscribe(&mut self, target: Target, subscription: Subscription) -> Vec<Action> {
        let key = subscription.key.clone();

        if let Some(subscriptions) = self.subscriptions_by_target.get_mut(&target) {
            subscriptions.insert(subscription);
            vec![Action::Subscribe(key)]
        } else {
            self.subscriptions_by_target.insert(target, HashSet::from([subscription]));
            vec![Action::Listen, Action::Subscribe(key)]
        }
    }
}
