// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use core::fmt;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use chariott_common::error::Error;
use url::Url;

use crate::streaming::StreamingEss;

const SYSTEM_NAMESPACE: &str = "system";
const SYSTEM_NAMESPACE_PREFIX: &str = "system.";

#[derive(Clone)]
pub enum Change<'a> {
    Add(&'a IntentConfiguration, &'a HashSet<ServiceConfiguration>),
    Modify(&'a IntentConfiguration, &'a HashSet<ServiceConfiguration>),
    Remove(&'a IntentConfiguration),
}

/// Represents a type which can observe changes to the registry.
pub trait Observer {
    /// Handles observation on changed registry state.
    fn on_change<'a>(&self, changes: impl Iterator<Item = Change<'a>> + Clone);
}

impl Observer for StreamingEss {
    fn on_change<'a>(&self, changes: impl IntoIterator<Item = Change<'a>>) {
        for namespace in changes
            .into_iter()
            .filter_map(|change| match change {
                Change::Add(intent, _) => Some(intent.namespace()),
                Change::Modify(_, _) => None,
                Change::Remove(intent) => Some(intent.namespace()),
            })
            .collect::<HashSet<_>>()
        {
            self.publish(format!("namespaces/{}", namespace).as_str(), ());
        }
    }
}

pub struct Composite<T, U> {
    left: T,
    right: U,
}

impl<T, U> Composite<T, U> {
    pub fn new(left: T, right: U) -> Self {
        Self { left, right }
    }
}

impl<T: Observer, U: Observer> Observer for Composite<T, U> {
    fn on_change<'a>(&self, changes: impl Iterator<Item = Change<'a>> + Clone) {
        self.left.on_change(changes.clone());
        self.right.on_change(changes);
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    entry_ttl: Duration,
}

impl Config {
    pub const ENTRY_TTL_MIN: Duration = Duration::from_secs(2);

    pub fn entry_ttl(&self) -> Duration {
        self.entry_ttl
    }

    pub fn set_entry_ttl_bounded(self, value: Duration) -> Self {
        Self { entry_ttl: std::cmp::max(value, Self::ENTRY_TTL_MIN) }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { entry_ttl: Duration::from_secs(15) }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Specificity {
    Default,
    Specific,
}

#[derive(Clone, Debug)]
pub struct Registry<T: Observer> {
    external_services_by_intent: HashMap<IntentConfiguration, HashSet<ServiceConfiguration>>,
    known_services: HashMap<ServiceConfiguration, Instant>,
    observer: T,
    config: Config,
}

impl<T: Observer> Registry<T> {
    pub fn new(observer: T, config: Config) -> Self {
        Self {
            external_services_by_intent: HashMap::new(),
            known_services: HashMap::new(),
            observer,
            config,
        }
    }

    /// Returns whether the specified service configuration is already known to
    /// the registry. As system services cannot be updated, invocations with a
    /// system service configuration results in undefined behavior.
    #[cfg(test)]
    fn has_service(&self, key: &ServiceConfiguration) -> bool {
        self.known_services.contains_key(key)
    }

    pub fn touch(&mut self, key: &ServiceConfiguration, timestamp: Instant) -> bool {
        if let Some(ts) = self.known_services.get_mut(key) {
            *ts = timestamp;
            true
        } else {
            false
        }
    }

    fn prune_by(
        &mut self,
        predicate: impl Fn(&ServiceConfiguration, Instant) -> bool,
    ) -> ChangeSeries {
        let mut change_series = ChangeSeries::new();

        let initial_known_services_len = self.known_services.len();

        self.known_services.retain(|services, ts| !predicate(services, *ts));

        if self.known_services.len() == initial_known_services_len {
            return change_series;
        }

        // Prune the old service registrations and bindings.

        self.external_services_by_intent.retain(|intent_configuration, services| {
            let service_count = services.len();

            services.retain(|service| self.known_services.contains_key(service));

            if service_count != services.len() {
                match services.len() {
                    0 => change_series.change(intent_configuration.clone(), ChangeKind::Remove),
                    _ => change_series.change(intent_configuration.clone(), ChangeKind::Modify),
                }
            }

            !services.is_empty()
        });

        change_series
    }

    pub fn prune(&mut self, timestamp: Instant) -> (Specificity, Instant) {
        use Specificity::*;
        let ttl = self.config.entry_ttl;
        let change_series = self.prune_by(|_, ts| timestamp.duration_since(ts) > ttl);
        change_series.observe(&self.observer, self);

        self.known_services
            .iter()
            .map(|(_, ts)| *ts + ttl)
            .min()
            .map(|t| (Specific, t))
            .unwrap_or((Default, timestamp + ttl))
    }

    pub fn upsert(
        &mut self,
        service_configuration: ServiceConfiguration,
        intent_configurations: Vec<IntentConfiguration>,
        timestamp: Instant,
    ) -> Result<(), Error> {
        fn starts_with_ignore_ascii_case(string: &str, prefix: &str) -> bool {
            string.len() >= prefix.len()
                && string.as_bytes()[0..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())
        }

        if intent_configurations.iter().any(|ic| {
            ic.namespace.eq_ignore_ascii_case(SYSTEM_NAMESPACE)
                || starts_with_ignore_ascii_case(ic.namespace.as_str(), SYSTEM_NAMESPACE_PREFIX)
        }) {
            return Err(Error::new(
                "It is not possible to overwrite an existing system registration",
            ));
        }

        // Upserting a registration should not happen frequently and has worse
        // performance than service resolution.

        let mut change_series = self.prune_by(|service, _| service.id == service_configuration.id);

        // Add the new service registrations and resolve the new Bindings to be
        // used for each intent.

        for intent_configuration in intent_configurations {
            // Update the list of registry changes.

            match self.external_services_by_intent.contains_key(&intent_configuration) {
                true => change_series.change(intent_configuration.clone(), ChangeKind::Modify),
                false => change_series.change(intent_configuration.clone(), ChangeKind::Add),
            };

            // Update the service registry for a given intent.

            self.external_services_by_intent
                .entry(intent_configuration)
                .or_insert_with(HashSet::new)
                .insert(service_configuration.clone());
        }

        // Add the service to the lookup for known services.

        self.known_services.insert(service_configuration, timestamp);

        // Notify the observer

        change_series.observe(&self.observer, self);

        Ok(())
    }

    #[cfg(test)]
    pub fn count_external_intents(&self) -> usize {
        self.external_services_by_intent.len()
    }
}

#[derive(Copy, Clone, Debug)]
enum ChangeKind {
    Add,
    Remove,
    Modify,
}

/// Tracks the effective change to a registry based on a _series_ of isolated
/// changes for a given intent.
struct ChangeSeries {
    changes: HashMap<IntentConfiguration, ChangeKind>,
}

impl ChangeSeries {
    pub fn new() -> Self {
        Self { changes: HashMap::new() }
    }

    fn change(&mut self, intent: IntentConfiguration, to: ChangeKind) {
        let from = self.changes.get(&intent);
        let value = match (from, to) {
            (None, _) => to,
            (Some(ChangeKind::Remove), ChangeKind::Add) => ChangeKind::Modify,
            (Some(ChangeKind::Modify), ChangeKind::Modify) => ChangeKind::Modify,
            (Some(ChangeKind::Add), ChangeKind::Modify) => ChangeKind::Add,
            (from, to) => {
                panic!(
                    "{}",
                    format!("Bug: Transition from {from:?} to {to:?} must not be possible.")
                );
            }
        };

        self.changes.insert(intent, value);
    }

    fn observe<O: Observer>(self, observer: &O, registry: &Registry<O>) {
        let changes = self.changes.iter().map(|(intent, kind)| match kind {
            ChangeKind::Add => Change::Add(intent, &registry.external_services_by_intent[intent]),
            ChangeKind::Modify => {
                Change::Modify(intent, &registry.external_services_by_intent[intent])
            }
            ChangeKind::Remove => Change::Remove(intent),
        });

        if changes.len() > 0 {
            observer.on_change(changes);
        };
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct ServiceId(Box<str>, Box<str>);

impl ServiceId {
    pub fn new(name: impl Into<Box<str>>, version: impl Into<Box<str>>) -> Self {
        Self(name.into(), version.into())
    }

    pub fn name(&self) -> Box<str> {
        self.0.clone()
    }

    pub fn version(&self) -> Box<str> {
        self.1.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ServiceConfiguration {
    id: ServiceId,
    url: Url,
    locality: ExecutionLocality,
}

impl ServiceConfiguration {
    pub fn new(id: ServiceId, url: Url, locality: ExecutionLocality) -> Self {
        Self { id, url, locality }
    }

    pub fn locality(&self) -> &ExecutionLocality {
        &self.locality
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn id(&self) -> &ServiceId {
        &self.id
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionLocality {
    Local,
    Cloud,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct IntentConfiguration {
    namespace: String,
    intent: IntentKind,
}

impl IntentConfiguration {
    pub fn new(namespace: impl Into<String>, intent: IntentKind) -> Self {
        Self { namespace: namespace.into(), intent }
    }

    pub fn into_namespaced_intent(self) -> (String, IntentKind) {
        (self.namespace, self.intent)
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IntentKind {
    Discover,
    Inspect,
    Read,
    Write,
    Invoke,
    Subscribe,
}

impl fmt::Display for IntentKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            IntentKind::Discover => "discover",
            IntentKind::Inspect => "inspect",
            IntentKind::Read => "read",
            IntentKind::Write => "write",
            IntentKind::Invoke => "invoke",
            IntentKind::Subscribe => "subscribe",
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::{
        collections::HashSet,
        sync::{
            atomic::{AtomicBool, Ordering},
            Mutex,
        },
        time::Instant,
    };

    use chariott_common::{
        proto::common::{value::Value, SubscribeIntent},
        streaming_ess::StreamingEss,
    };
    use test_case::test_case;

    use crate::{
        execution::tests::StreamExt as _,
        registry::{Composite, ExecutionLocality, IntentKind, ServiceId},
    };

    use super::*;

    fn now() -> Instant {
        Instant::now()
    }

    #[test]
    fn default_config() {
        let config: Config = Default::default();

        assert_eq!(Duration::from_secs(15), config.entry_ttl());
    }

    #[test]
    fn config_set_entry_ttl_sets_new_value() {
        let config: Config = Default::default();
        let new_ttl = config.entry_ttl() + Duration::from_secs(60);

        let ttl = config.set_entry_ttl_bounded(new_ttl).entry_ttl();

        assert_eq!(new_ttl, ttl);
    }

    #[test]
    fn config_set_entry_ttl_sets_min_allowed_if_new_value_is_too_small() {
        let config: Config = Default::default();
        let new_ttl = Config::ENTRY_TTL_MIN - Duration::from_nanos(1);

        let ttl = config.set_entry_ttl_bounded(new_ttl).entry_ttl();

        assert_eq!(Config::ENTRY_TTL_MIN, ttl);
    }

    #[test]
    fn when_upserting_contains_service() {
        // arrange
        let mut registry = create_registry();
        let service = ServiceConfigurationBuilder::new().build();
        let intents = vec![IntentConfigurationBuilder::new().build()];

        // act
        registry.upsert(service.clone(), intents, now()).unwrap();

        // assert
        assert!(registry.has_service(&service));
    }

    #[test]
    fn when_upserting_empty_caches_service() {
        // arrange
        let mut registry = create_registry();
        let service = ServiceConfigurationBuilder::new().build();

        // act
        registry.upsert(service.clone(), vec![], now()).unwrap();

        // assert
        assert!(registry.has_service(&service));
        assert!(registry.observer.is_empty());
    }

    #[test]
    fn when_upserting_refreshes_broker_with_updated_service_configurations() {
        // arrange
        let setup = Setup::new();
        let mut registry = setup.clone().build();
        let service = ServiceConfigurationBuilder::with_nonce("2").build();

        // act
        registry.upsert(service.clone(), setup.intents.clone(), now()).unwrap();

        // assert
        registry.observer.assert_number_of_changes(&[1]);
        registry.observer.assert_modified(&setup.intents[0], |actual_services| {
            assert!(actual_services.contains(&setup.service.build()));
            assert!(actual_services.contains(&service));
        });
    }

    #[test]
    fn when_upserting_with_different_url_prunes_old_instance_and_refreshes_broker() {
        // arrange
        let setup = Setup::new();
        let mut registry = setup.clone().build();
        let service = setup.service.clone().build();
        let updated_service = setup.service.url("http://updated_url").build();

        // act
        registry.upsert(updated_service.clone(), setup.intents.clone(), now()).unwrap();

        // assert
        assert!(registry.has_service(&updated_service));
        assert!(!registry.has_service(&service));

        // broker was refreshed only once, as changes are observed
        // "transactionally".
        registry.observer.assert_number_of_changes(&[1]);
        registry.observer.assert_modified(&setup.intents[0], |services| {
            assert_eq!([updated_service], services.as_slice());
        });
    }

    #[test]
    fn when_upserting_with_different_versions_should_be_treated_as_different_services() {
        // arrange
        let setup = Setup::new();
        let mut registry = setup.clone().build();
        let service = setup.service.clone().build();
        let updated_service = setup.service.version("10.30.40").build();

        // act
        registry.upsert(updated_service.clone(), setup.intents.clone(), now()).unwrap();

        // assert
        assert!(registry.has_service(&service));
        assert!(registry.has_service(&updated_service));
        registry.observer.assert_modified(&setup.intents[0], |actual_services| {
            assert!(actual_services.contains(&service));
            assert!(actual_services.contains(&updated_service));
        });
    }

    #[test]
    fn when_service_reregisters_refreshes_all_affected_registrations_in_broker() {
        // Test setup is as follows:
        // initial:
        // intent_1: [service_a, service_b],
        //
        // after act:
        // intent_1: [service_b]
        // intent_2: [service_a(with updated URL)]

        // arrange
        let service_a = ServiceConfigurationBuilder::with_nonce("A");
        let service_b = ServiceConfigurationBuilder::with_nonce("B");
        let service_a_reregistration = service_a.clone().url("http://service-a-new").build();

        let intent_1 = IntentConfigurationBuilder::with_nonce("1").build();
        let intent_2 = IntentConfigurationBuilder::with_nonce("2").build();

        let mut registry = create_registry();
        registry.upsert(service_a.clone().build(), vec![intent_1.clone()], now()).unwrap();
        registry.upsert(service_b.clone().build(), vec![intent_1.clone()], now()).unwrap();
        registry.observer.clear();

        // act
        registry.upsert(service_a_reregistration.clone(), vec![intent_2.clone()], now()).unwrap();

        // assert
        registry.observer.assert_number_of_changes(&[2]);

        registry.observer.assert_modified(&intent_1, |actual_services| {
            assert_eq!([service_b.build()], actual_services.as_slice());
        });

        registry.observer.assert_added(&intent_2, |actual_services| {
            assert_eq!([service_a_reregistration.clone()], actual_services.as_slice());
        });

        assert!(registry.has_service(&service_a_reregistration));
        assert!(!registry.has_service(&service_a.build()));
    }

    #[test]
    fn when_upserting_same_service_with_new_intents_prunes_old_intent() {
        // arrange
        let setup = Setup::new();
        let mut registry = setup.clone().build();
        let reregistration_intent =
            IntentConfiguration::new("some_other_namespace", IntentKind::Read);

        // act
        registry.upsert(setup.service.build(), vec![reregistration_intent], now()).unwrap();

        // assert
        registry.observer.assert_removed(&setup.intents[0]);
    }

    #[test]
    fn when_upserting_same_intent_twice_is_idempotent() {
        // arrange
        let mut registry = create_registry();
        let intent = IntentConfigurationBuilder::new().build();
        let service = ServiceConfigurationBuilder::new().build();

        // act
        registry.upsert(service.clone(), vec![intent.clone(), intent.clone()], now()).unwrap();

        // assert
        assert!(registry.has_service(&service));
        registry.observer.assert_added(&intent, |services| {
            assert_eq!(1, services.len());
            assert_eq!(&vec![service], services);
        });
    }

    #[test]
    fn when_upserting_system_binding_returns_error() {
        test("system");
        test("system.registry");
        test("system.foo");
        test("system.");
        test("System");
        test("SYSTEM");
        test("SYSTEM.Registry");

        fn test(namespace: &str) {
            // arrange
            let service_configuration = ServiceConfigurationBuilder::new().build();
            let intent_configuration =
                IntentConfigurationBuilder::new().namespace(namespace).build();

            // act
            let result =
                create_registry().upsert(service_configuration, vec![intent_configuration], now());

            // assert
            assert!(result.is_err());
        }
    }

    #[test_case(Specificity::Default, 15, 0, [])]
    #[test_case(Specificity::Default, 15, 5, [])]
    #[test_case(Specificity::Default, 15, 15, [])]
    #[test_case(Specificity::Specific, 17, 0, [6, 2, 4])]
    #[test_case(Specificity::Specific, 7, 10, [6, 2, 4])]
    fn prune_schedule(
        expected_specificity: Specificity,
        expected_seconds_since_prune: u64,
        prune_seconds: u64,
        seconds: impl IntoIterator<Item = u64>,
    ) {
        let mut registry = create_registry();

        let epoch: Instant = now();
        let setup = ServiceConfigurationBuilder::dispense(1..)
            .into_iter()
            .map(|b| b.build())
            .zip(IntentConfigurationBuilder::dispense(1..).into_iter().map(|b| vec![b.build()]))
            .zip(seconds.into_iter().map(|s| epoch + Duration::from_secs(s)));

        for ((service, intents), timestamp) in setup {
            registry.upsert(service.clone(), intents.clone(), timestamp).unwrap();
        }

        let prune_time = epoch + Duration::from_secs(prune_seconds);
        let (specificity, t) = registry.prune(prune_time);

        assert_eq!(expected_specificity, specificity);
        assert_eq!(expected_seconds_since_prune, t.duration_since(prune_time).as_secs());
    }

    #[test_case(0, 0, 15, true, true)]
    #[test_case(0, 0, 16, false, false)]
    #[test_case(0, 20, 5, false, true)]
    #[test_case(0, 20, 10, false, true)]
    #[test_case(0, 20, 15, false, true)]
    #[test_case(0, 20, 16, false, false)]
    fn prune_removes_expired_services(
        first_registration_since_epoch: u64,
        second_since_first_registration: u64,
        prune_since_second_registration: u64,
        expect_first_registered: bool,
        expect_second_registered: bool,
    ) {
        let first_registration_since_epoch = Duration::from_secs(first_registration_since_epoch);
        let second_since_first_registration = Duration::from_secs(second_since_first_registration);
        let prune_since_second_registration = Duration::from_secs(prune_since_second_registration);

        // arrange

        let mut time = now();

        let mut registry = create_registry();

        let mut service_builder = ServiceConfigurationBuilder::dispense('a'..).into_iter();
        let mut intent_builder = IntentConfigurationBuilder::dispense('a'..).into_iter();

        let first_service = service_builder.next().unwrap().build();
        let first_intent = intent_builder.next().unwrap().build();
        time += first_registration_since_epoch;
        registry.upsert(first_service.clone(), vec![first_intent.clone()], time).unwrap();

        let second_service = service_builder.next().unwrap().build();
        let second_intent = intent_builder.next().unwrap().build();
        time += second_since_first_registration;
        registry.upsert(second_service.clone(), vec![second_intent.clone()], time).unwrap();

        registry.observer.clear();

        time += prune_since_second_registration;

        // act

        registry.prune(time);

        // assert

        assert_eq!(expect_first_registered, registry.has_service(&first_service));
        assert_eq!(expect_second_registered, registry.has_service(&second_service));

        registry.observer.assert_number_of_changes(
            match (expect_first_registered, expect_second_registered) {
                (true, true) => &[],
                (true, false) => &[1],
                (false, true) => &[1],
                (false, false) => &[2],
            },
        );

        if !expect_first_registered {
            registry.observer.assert_removed(&first_intent);
        }

        if !expect_second_registered {
            registry.observer.assert_removed(&second_intent);
        }
    }

    #[test]
    fn touch_returns_false_if_service_is_unregistered() {
        // arrange
        let mut registry = create_registry();
        let service = ServiceConfigurationBuilder::new().build();

        // act
        let found = registry.touch(&service, now());

        // assert
        assert!(!found);
    }

    #[test]
    fn touch_updates_timestamp() {
        // arrange
        let mut now = now();
        let mut registry = create_registry();
        let service = ServiceConfigurationBuilder::new().build();
        let intent = IntentConfigurationBuilder::new().build();
        registry.upsert(service.clone(), vec![intent], now).unwrap();

        // act
        now += Duration::from_secs(10);
        _ = registry.prune(now);
        let found1 = registry.touch(&service, now);

        now += Duration::from_secs(15);
        _ = registry.prune(now);
        let found2 = registry.touch(&service, now);

        // assert
        assert!(found1);
        assert!(found2);
    }

    #[test]
    fn test_create_new_service_configuration() {
        let service = ServiceConfiguration::new(
            ServiceId::new("name", "version"),
            "http://foo".parse().unwrap(),
            ExecutionLocality::Local,
        );
        assert_eq!(service.id.name(), "name".into());
        assert_eq!(service.id.version(), "version".into());
        assert_eq!(service.url, "http://foo".parse().unwrap());
        assert_eq!(service.locality, ExecutionLocality::Local);
    }

    #[test]
    fn test_create_new_intent_configuration() {
        let intent = IntentConfiguration::new("namespace".to_string(), IntentKind::Discover);
        assert_eq!(intent.namespace, "namespace");
        assert_eq!(intent.intent, IntentKind::Discover);
    }

    #[test]
    fn service_id_returns_name_and_version() {
        // arrange
        let name = "name".to_owned();
        let version = "1.0.0".to_owned();
        let service = ServiceId::new(name.as_str(), version.as_str());

        // act + assert
        assert_eq!(name.into_boxed_str(), service.name());
        assert_eq!(version.into_boxed_str(), service.version());
    }

    #[test]
    fn intent_kind_display_succeeds() {
        // The match is only here to catch adding of new intents. Devs adding
        // new intents are required to update the match arm as well as the
        // mapping validations below.
        let intent = IntentKind::Discover;
        match intent {
            IntentKind::Discover => {}
            IntentKind::Inspect => {}
            IntentKind::Read => {}
            IntentKind::Write => {}
            IntentKind::Invoke => {}
            IntentKind::Subscribe => {}
        }

        test("discover", IntentKind::Discover);
        test("inspect", IntentKind::Inspect);
        test("read", IntentKind::Read);
        test("write", IntentKind::Write);
        test("invoke", IntentKind::Invoke);
        test("subscribe", IntentKind::Subscribe);

        fn test(expected: &str, intent_kind: IntentKind) {
            assert_eq!(expected, format!("{}", intent_kind));
        }
    }

    #[test]
    fn composite_observes_both_inner_observers() {
        // arrange
        struct TestObserver {
            invoked: AtomicBool,
        }

        impl Observer for TestObserver {
            fn on_change<'a>(&self, _: impl Iterator<Item = Change<'a>> + Clone) {
                self.invoked.fetch_or(true, Ordering::Relaxed);
            }
        }

        let subject = Composite::new(
            TestObserver { invoked: Default::default() },
            TestObserver { invoked: Default::default() },
        );

        // act
        subject.on_change([].into_iter());

        // assert
        assert!(subject.left.invoked.load(Ordering::Relaxed));
        assert!(subject.right.invoked.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn on_change_notifies_when_namespace_change_detected() {
        const INTENT_A: &str = "A";
        const INTENT_B: &str = "B";
        const INTENT_C: &str = "C";

        fn intent(nonce: &str) -> IntentConfiguration {
            IntentConfigurationBuilder::with_nonce(nonce).build()
        }

        let services = HashSet::new(); // The observe logic does not care about which services serve a specific intent.
        let intent_a = intent(INTENT_A);
        let intent_b = intent(INTENT_B);
        let intent_c = intent(INTENT_C);

        test([Change::Add(&intent_a, &services)], [&intent_a]).await;
        test(
            [Change::Add(&intent_a, &services), Change::Modify(&intent_b, &services)],
            [&intent_a],
        )
        .await;
        test([Change::Modify(&intent_b, &services), Change::Remove(&intent_a)], [&intent_a]).await;
        test(
            [Change::Add(&intent_b, &services), Change::Remove(&intent_a)],
            [&intent_b, &intent_a],
        )
        .await;
        test(
            [
                Change::Add(&intent_b, &services),
                Change::Remove(&intent_a),
                Change::Modify(&intent_c, &services),
            ],
            [&intent_a, &intent_b],
        )
        .await;
        test([Change::Modify(&intent_a, &services)], []).await;

        async fn test<'a, 'b>(
            changes: impl IntoIterator<Item = Change<'a>>,
            expected_events: impl IntoIterator<Item = &'b IntentConfiguration>,
        ) {
            fn namespace_event(namespace: &str) -> String {
                format!("namespaces/{}", namespace)
            }

            // arrange
            const CLIENT_ID: &str = "CLIENT";

            let subject = StreamingEss::new();
            let (_, stream) = subject.read_events(CLIENT_ID.into());

            // always subscribe to all possible namespace changes.
            for nonce in [INTENT_A, INTENT_B, INTENT_C] {
                let intent = IntentConfigurationBuilder::with_nonce(nonce).build();
                subject
                    .serve_subscriptions(
                        SubscribeIntent {
                            channel_id: CLIENT_ID.into(),
                            sources: vec![namespace_event(intent.namespace())],
                        },
                        |_| Value::Null(0),
                    )
                    .unwrap();
            }

            // act
            subject.on_change(changes.into_iter().collect::<Vec<_>>().into_iter());

            // assert
            let mut expected_events = expected_events
                .into_iter()
                .map(|e| namespace_event(e.namespace()))
                .collect::<Vec<_>>();

            // collect the result while there are still events incoming.
            let mut result = stream
                .collect_when_stable()
                .await
                .into_iter()
                .map(|e| e.unwrap().source)
                .collect::<Vec<_>>();

            // namespace change events can be delivered out of order. Sort
            // before comparing.
            result.sort();
            expected_events.sort();

            assert_eq!(result, expected_events);
        }
    }

    struct MockBroker {
        refresh_calls: Mutex<Vec<Vec<ChangeSnapshot>>>,
    }

    enum ChangeSnapshot {
        Add(IntentConfiguration, Vec<ServiceConfiguration>),
        Modify(IntentConfiguration, Vec<ServiceConfiguration>),
        Remove(IntentConfiguration),
    }

    impl MockBroker {
        pub fn new() -> Self {
            Self { refresh_calls: Mutex::new(Vec::new()) }
        }

        pub fn clear(&mut self) {
            self.refresh_calls = Mutex::new(Vec::new());
        }

        pub fn assert_modified(
            &self,
            intent_configuration: &IntentConfiguration,
            assert: impl FnOnce(&Vec<ServiceConfiguration>),
        ) {
            let services = self
                .filter_map_change(intent_configuration, |change| match change {
                    ChangeSnapshot::Modify(_, s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| {
                    panic!("Expected one modification for {intent_configuration:?}.")
                });

            assert(&services);
        }

        pub fn assert_added(
            &self,
            intent_configuration: &IntentConfiguration,
            assert: impl FnOnce(&Vec<ServiceConfiguration>),
        ) {
            let services = self
                .filter_map_change(intent_configuration, |change| match change {
                    ChangeSnapshot::Add(_, s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("Expected one addition for {intent_configuration:?}."));

            assert(&services);
        }

        pub fn assert_removed(&self, intent_configuration: &IntentConfiguration) {
            self.filter_map_change(intent_configuration, |change| match change {
                ChangeSnapshot::Remove(_) => Some(()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("Expected one removal for {intent_configuration:?}."));
        }

        fn filter_map_change<T>(
            &self,
            expected_intent: &IntentConfiguration,
            filter_map: fn(&ChangeSnapshot) -> Option<T>,
        ) -> Option<T> {
            self.refresh_calls.lock().unwrap().iter().flatten().find_map(|change| {
                let actual_intent = match change {
                    ChangeSnapshot::Add(i, _) => i,
                    ChangeSnapshot::Modify(i, _) => i,
                    ChangeSnapshot::Remove(i) => i,
                };

                if actual_intent != expected_intent {
                    return None;
                }

                filter_map(change)
            })
        }

        pub fn is_empty(&self) -> bool {
            self.refresh_calls.lock().unwrap().is_empty()
        }

        pub fn assert_number_of_changes(&self, expected: &[usize]) {
            let refresh_calls = self.refresh_calls.lock().unwrap();
            assert_eq!(expected.len(), refresh_calls.len());

            for (expected, refresh_calls) in expected.iter().zip(refresh_calls.iter()) {
                assert_eq!(*expected, refresh_calls.len());
            }
        }
    }

    impl Observer for MockBroker {
        fn on_change<'a>(&self, changes: impl IntoIterator<Item = Change<'a>>) {
            let changes = changes
                .into_iter()
                .map(|change| match change {
                    Change::Add(i, s) => {
                        ChangeSnapshot::Add(i.clone(), s.iter().cloned().collect())
                    }
                    Change::Modify(i, s) => {
                        ChangeSnapshot::Modify(i.clone(), s.iter().cloned().collect())
                    }
                    Change::Remove(i) => ChangeSnapshot::Remove(i.clone()),
                })
                .collect();

            self.refresh_calls.lock().unwrap().push(changes);
        }
    }

    fn create_registry() -> Registry<MockBroker> {
        Registry::new(MockBroker::new(), Default::default())
    }

    #[derive(Clone)]
    struct Setup {
        intents: Vec<IntentConfiguration>,
        service: ServiceConfigurationBuilder,
    }

    impl Setup {
        /// Test setup containing a single service with a single intent.
        fn new() -> Self {
            let intents = vec![IntentConfigurationBuilder::new().build()];
            let service = ServiceConfigurationBuilder::new();
            Setup { intents, service }
        }

        fn build(self) -> Registry<MockBroker> {
            let mut registry = Registry::new(MockBroker::new(), Default::default());
            registry.upsert(self.service.clone().build(), self.intents, now()).unwrap();
            registry.observer.clear();
            registry
        }
    }

    use std::fmt::Display;

    #[derive(Clone)]
    pub struct ServiceConfigurationBuilder(ServiceConfiguration);

    impl ServiceConfigurationBuilder {
        pub fn new() -> Self {
            Self::with_nonce("0")
        }

        pub fn dispense(
            nonce: impl IntoIterator<Item = impl Display>,
        ) -> impl IntoIterator<Item = Self> {
            nonce.into_iter().map(Self::with_nonce)
        }

        pub fn with_nonce(nonce: impl std::fmt::Display) -> Self {
            Self(ServiceConfiguration::new(
                ServiceId::new(format!("mock-service-{nonce}"), "0.1.0"),
                format!("http://service-{nonce}").parse().unwrap(),
                ExecutionLocality::Cloud,
            ))
        }

        pub fn build(self) -> ServiceConfiguration {
            self.0
        }

        pub fn version(mut self, version: impl Into<Box<str>>) -> Self {
            self.0.id = ServiceId::new(self.0.id.name(), version);
            self
        }

        pub fn name(mut self, name: impl Into<Box<str>>) -> Self {
            self.0.id = ServiceId::new(name, self.0.id.version());
            self
        }

        pub fn url(mut self, url: &str) -> Self {
            self.0.url = url.parse().unwrap();
            self
        }

        pub fn execution_locality(mut self, execution_locality: ExecutionLocality) -> Self {
            self.0.locality = execution_locality;
            self
        }
    }

    #[derive(Clone)]
    pub struct IntentConfigurationBuilder(IntentConfiguration);

    impl IntentConfigurationBuilder {
        pub fn new() -> Self {
            Self::with_nonce("0")
        }

        pub fn dispense(
            nonce: impl IntoIterator<Item = impl Display>,
        ) -> impl IntoIterator<Item = Self> {
            nonce.into_iter().map(Self::with_nonce)
        }

        pub fn with_nonce(nonce: impl std::fmt::Display) -> Self {
            Self(IntentConfiguration::new(format!("namespace-{nonce}"), IntentKind::Discover))
        }

        pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
            self.0 = IntentConfiguration::new(namespace, self.0.intent);
            self
        }

        pub fn build(self) -> IntentConfiguration {
            self.0
        }
    }
}
