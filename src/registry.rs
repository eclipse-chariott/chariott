// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use core::fmt;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use chariott_common::error::Error;
use url::Url;

const SYSTEM_NAMESPACE: &str = "system";
const SYSTEM_NAMESPACE_PREFIX: &str = "system.";

/// Represents a type which can observe changes to the registry.
pub trait RegistryObserver {
    /// Handles observation on changed registry state.
    fn on_intent_config_change<'a>(
        &self,
        intent_configuration: IntentConfiguration,
        service_configurations: impl IntoIterator<Item = &'a ServiceConfiguration>,
    );
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
pub struct Registry<T: RegistryObserver> {
    external_services_by_intent: HashMap<IntentConfiguration, HashSet<ServiceConfiguration>>,
    known_services: HashMap<ServiceConfiguration, Instant>,
    observer: T,
    config: Config,
}

impl<T: RegistryObserver> Registry<T> {
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

    fn prune_by(&mut self, predicate: impl Fn(&ServiceConfiguration, Instant) -> bool) -> bool {
        let initial_known_services_len = self.known_services.len();

        self.known_services.retain(|services, ts| !predicate(services, *ts));

        if self.known_services.len() == initial_known_services_len {
            return false;
        }

        // Prune the old service registrations and bindings.

        self.external_services_by_intent.retain(|intent_configuration, services| {
            let service_count = services.len();

            services.retain(|service| self.known_services.contains_key(service));

            if service_count != services.len() {
                // If we changed the services, we need to refresh the broker.
                self.observer
                    .on_intent_config_change(intent_configuration.clone(), services.iter());
            }

            !services.is_empty()
        });

        true
    }

    pub fn prune(&mut self, timestamp: Instant) -> (Specificity, Instant) {
        use Specificity::*;
        let ttl = self.config.entry_ttl;
        _ = self.prune_by(|_, ts| timestamp.duration_since(ts) > ttl);
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

        _ = self.prune_by(|service, _| service.id == service_configuration.id);

        // Add the new service registrations and resolve the new Bindings to be
        // used for each intent.

        for intent_configuration in intent_configurations {
            // Update the service registry for a given intent.

            let service_configurations = self
                .external_services_by_intent
                .entry(intent_configuration.clone())
                .or_insert_with(HashSet::new);

            service_configurations.insert(service_configuration.clone());

            // Refresh the broker with the updated service configurations.

            self.observer
                .on_intent_config_change(intent_configuration, service_configurations.iter());
        }

        // Add the service to the lookup for known services.

        self.known_services.insert(service_configuration, timestamp);
        Ok(())
    }

    #[cfg(test)]
    pub fn count_external_intents(&self) -> usize {
        self.external_services_by_intent.len()
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

    pub fn namespace_as_str(&self) -> &str {
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
    use std::sync::Mutex;
    use std::time::Instant;
    use test_case::test_case;

    use crate::registry::{ExecutionLocality, IntentKind, ServiceId};

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
        registry.observer.assert_number_of_refreshes(1);
        registry.observer.assert_refreshed_with(&setup.intents[0], |actual_services| {
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
        registry.upsert(updated_service.clone(), setup.intents, now()).unwrap();

        // assert
        assert!(registry.has_service(&updated_service));
        assert!(!registry.has_service(&service));

        // broker was refreshed twice, the first time to prune the old services,
        // then to update with the new services.
        let refresh_calls = registry.observer.refresh_calls.lock().unwrap();
        assert_eq!(2, refresh_calls.len());
        assert!(refresh_calls[0].1.is_empty());
        assert_eq!([updated_service], refresh_calls[1].1.as_slice());
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
        registry.observer.assert_refreshed_with(&setup.intents[0], |actual_services| {
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
        registry.observer.assert_refreshed_with(&intent_1, |actual_services| {
            assert_eq!([service_b.build()], actual_services.as_slice());
        });

        registry.observer.assert_refreshed_with(&intent_2, |actual_services| {
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
        registry.observer.assert_refreshed_with(&setup.intents[0], |services| {
            assert!(services.is_empty());
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

        let refreshes = registry.observer.refresh_calls.lock().unwrap();
        // remove the refreshes due to upserts
        let expected_refreshes_len =
            !expect_first_registered as usize + !expect_second_registered as usize;

        assert_eq!(expected_refreshes_len, refreshes.len());

        for (ic, scs) in refreshes.iter() {
            assert!(scs.is_empty());
            if *ic == first_intent {
                assert!(!expect_first_registered);
            } else if *ic == second_intent {
                assert!(!expect_second_registered);
            } else {
                panic!("unhandled case?");
            }
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

    struct MockBroker {
        refresh_calls: Mutex<Vec<(IntentConfiguration, Vec<ServiceConfiguration>)>>,
    }

    impl MockBroker {
        pub fn new() -> Self {
            Self { refresh_calls: Mutex::new(Vec::new()) }
        }

        pub fn clear(&mut self) {
            self.refresh_calls = Mutex::new(Vec::new());
        }

        pub fn assert_refreshed_with(
            &self,
            intent_configuration: &IntentConfiguration,
            assert: impl FnOnce(&Vec<ServiceConfiguration>),
        ) {
            if let Some((_, actual_services)) = self.refresh_calls.lock().unwrap().iter().find(
                |(actual_intent_configuration, _)| {
                    actual_intent_configuration == intent_configuration
                },
            ) {
                assert(actual_services);
            } else {
                panic!("Expected one invocation with {intent_configuration:?}.");
            }
        }

        pub fn is_empty(&self) -> bool {
            self.refresh_calls.lock().unwrap().is_empty()
        }

        pub fn assert_number_of_refreshes(&self, expected: usize) {
            assert_eq!(expected, self.refresh_calls.lock().unwrap().len());
        }
    }

    impl RegistryObserver for MockBroker {
        fn on_intent_config_change<'a>(
            &self,
            intent_configuration: IntentConfiguration,
            service_configurations: impl IntoIterator<Item = &'a ServiceConfiguration>,
        ) {
            self.refresh_calls
                .lock()
                .unwrap()
                .push((intent_configuration, service_configurations.into_iter().cloned().collect()))
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
