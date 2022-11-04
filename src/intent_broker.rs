// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use url::Url;

use crate::{
    connection_provider::{ConnectionProvider, GrpcProvider, ReusableProvider},
    execution::RuntimeBinding,
    registry::{Change, ExecutionLocality, IntentConfiguration, IntentKind, Observer},
    streaming::StreamingEss,
};

type Provider = ReusableProvider<GrpcProvider>;

#[derive(Clone)]
enum Binding {
    Remote(Provider),
    Fallback(Box<Binding>, Box<Binding>),
    SystemInspect,
    SystemDiscover(Url),
    SystemSubscribe(StreamingEss),
}

#[derive(Default)]
struct IntentBinder {
    bindings_by_intent: HashMap<IntentConfiguration, Binding>,
}

impl IntentBinder {
    pub fn new(streaming_url: Url, streaming_ess: StreamingEss) -> Self {
        const SYSTEM_REGISTRY_NAMESPACE: &str = "system.registry";

        Self {
            bindings_by_intent: HashMap::from([
                (
                    IntentConfiguration::new(SYSTEM_REGISTRY_NAMESPACE, IntentKind::Inspect),
                    Binding::SystemInspect,
                ),
                (
                    IntentConfiguration::new(SYSTEM_REGISTRY_NAMESPACE, IntentKind::Discover),
                    Binding::SystemDiscover(streaming_url),
                ),
                (
                    IntentConfiguration::new(SYSTEM_REGISTRY_NAMESPACE, IntentKind::Subscribe),
                    Binding::SystemSubscribe(streaming_ess),
                ),
            ]),
        }
    }

    pub fn resolve(&self, intent: &IntentConfiguration) -> Option<RuntimeBinding<Provider>> {
        fn binding_into_runtime_binding(
            broker: &IntentBinder,
            binding: &Binding,
        ) -> RuntimeBinding<Provider> {
            match binding {
                Binding::SystemInspect => RuntimeBinding::SystemInspect(
                    broker.bindings_by_intent.keys().cloned().collect(),
                ),
                Binding::Remote(provider) => RuntimeBinding::Remote(provider.clone()),
                Binding::Fallback(primary, secondary) => RuntimeBinding::Fallback(
                    Box::new(binding_into_runtime_binding(broker, primary)),
                    Box::new(binding_into_runtime_binding(broker, secondary)),
                ),
                Binding::SystemDiscover(url) => RuntimeBinding::SystemDiscover(url.clone()),
                Binding::SystemSubscribe(ess) => RuntimeBinding::SystemSubscribe(ess.clone()),
            }
        }

        self.bindings_by_intent
            .get(intent)
            .map(|binding| binding_into_runtime_binding(self, binding))
    }

    fn refresh<'a>(&mut self, changes: impl IntoIterator<Item = Change<'a>>) {
        for change in changes {
            let (intent_configuration, service_configurations) = match change {
                Change::Add(intent, services) => (intent, Some(services)),
                Change::Modify(intent, services) => (intent, Some(services)),
                Change::Remove(intent) => (intent, None),
            };

            let mut cloud_service = None;
            let mut local_service = None;

            if let Some(service_configurations) = service_configurations {
                for candidate in service_configurations {
                    match (candidate.locality(), &local_service, &cloud_service) {
                        // Stop on the first cloud/local provider that is
                        // found. This could be evolved in the future by
                        // always comparing all candidates using a priority
                        // as a tie-breaker (which does not yet exist).
                        (_, Some(_), Some(_)) => {
                            break;
                        }
                        (ExecutionLocality::Local, None, _) => {
                            local_service = Some(candidate);
                        }
                        (ExecutionLocality::Cloud, _, None) => {
                            cloud_service = Some(candidate);
                        }
                        (ExecutionLocality::Local, Some(_), None) => {}
                        (ExecutionLocality::Cloud, None, Some(_)) => {}
                    }
                }
            }

            let binding = match (local_service, cloud_service) {
                (Some(local_service), Some(cloud_service)) => Some(Binding::Fallback(
                    Box::new(Binding::Remote(Provider::new(cloud_service.url().to_owned()))),
                    Box::new(Binding::Remote(Provider::new(local_service.url().to_owned()))),
                )),
                (Some(service), None) => {
                    Some(Binding::Remote(Provider::new(service.url().to_owned())))
                }
                (None, Some(service)) => {
                    Some(Binding::Remote(Provider::new(service.url().to_owned())))
                }
                (None, None) => None,
            };

            if let Some(binding) = binding {
                self.bindings_by_intent.insert(intent_configuration.clone(), binding);
            } else {
                self.bindings_by_intent.remove(intent_configuration);
            }
        }
    }
}

/// Brokers intents based on internal state. Cloning is cheap and only increases
/// a reference count to shared mutable state.
#[derive(Clone, Default)]
pub struct IntentBroker(Arc<RwLock<IntentBinder>>);

impl IntentBroker {
    pub fn new(streaming_url: Url, streaming_ess: StreamingEss) -> Self {
        Self(Arc::new(RwLock::new(IntentBinder::new(streaming_url, streaming_ess))))
    }

    pub fn resolve(&self, intent: &IntentConfiguration) -> Option<RuntimeBinding<Provider>> {
        self.0.read().unwrap().resolve(intent)
    }
}

impl Observer for IntentBroker {
    fn on_change<'a>(&self, changes: impl IntoIterator<Item = Change<'a>>) {
        self.0.write().unwrap().refresh(changes)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{HashMap, HashSet},
        sync::Arc,
    };

    use chariott_common::streaming_ess::StreamingEss;
    use url::Url;

    use crate::{
        connection_provider::{GrpcProvider, ReusableProvider},
        execution::RuntimeBinding,
        intent_broker::{IntentBroker, Observer as _},
        registry::{
            tests::{IntentConfigurationBuilder, ServiceConfigurationBuilder},
            Change, ExecutionLocality, IntentConfiguration, IntentKind,
        },
    };

    #[test]
    fn when_empty_does_not_resolve() {
        // arrange
        let subject =
            IntentBroker::new("https://localhost:4243".parse().unwrap(), StreamingEss::new());

        // act + assert
        assert!(subject.resolve(&IntentConfigurationBuilder::new().build()).is_none());
    }

    #[test]
    fn when_broker_contains_different_intent_does_not_resolve() {
        // arrange
        let subject = Setup::new().build();

        // act + assert
        assert!(subject.resolve(&IntentConfigurationBuilder::with_nonce("2").build()).is_none());
    }

    #[test]
    fn when_modifying_with_empty_services_does_no_longer_resolve_intent() {
        // arrange
        let setup = Setup::new();
        let subject = setup.clone().build();

        // act
        subject.on_change([Change::Modify(&setup.intent, &HashSet::new())].into_iter());

        // assert
        assert!(subject.resolve(&setup.intent).is_none());
    }

    #[test]
    fn when_removing_does_no_longer_resolve_intent() {
        // arrange
        let setup = Setup::new();
        let subject = setup.clone().build();

        // act
        subject.on_change([Change::Remove(&setup.intent)].into_iter());

        // assert
        assert!(subject.resolve(&setup.intent).is_none());
    }

    #[test]
    fn when_resolve_if_services_are_cloud_and_local_returns_fallback() {
        // arrange
        let build = |execution_locality, name| {
            Setup::new().execution_locality(execution_locality).service_name(name)
        };

        let local = build(ExecutionLocality::Local, "A");
        let cloud = build(ExecutionLocality::Cloud, "B");
        let subject = Setup::combine([local.clone(), cloud.clone()]);

        // act
        let binding = subject.resolve(&local.intent).unwrap();

        // assert
        assert_remote_fallback_binding(
            &binding,
            |actual_service| assert_eq!(cloud.service.build().url(), actual_service),
            |actual_service| assert_eq!(local.service.build().url(), actual_service),
        );
    }

    #[test]
    fn when_resolve_binding_if_multi_cloud_and_multi_local_returns_cloud_and_local_fallback() {
        // arrange
        let intent = IntentConfigurationBuilder::new().build();
        let subject = Setup::combine(
            [
                (ExecutionLocality::Local, "local1"),
                (ExecutionLocality::Local, "local2"),
                (ExecutionLocality::Cloud, "cloud1"),
                (ExecutionLocality::Cloud, "cloud2"),
            ]
            .map(|(locality, name)| Setup {
                intent: intent.clone(),
                service: ServiceConfigurationBuilder::new()
                    .name(name)
                    .url(&format!("http://{}", name))
                    .execution_locality(locality),
            }),
        );

        // act
        let result = subject.resolve(&intent).unwrap();

        // assert
        assert_remote_fallback_binding(
            &result,
            |primary| assert!(primary.to_string().contains("cloud")),
            |secondary| assert!(secondary.to_string().contains("local")),
        );
    }

    #[test]
    fn when_resolve_with_single_locality_is_remote() {
        test([ExecutionLocality::Cloud]);
        test([ExecutionLocality::Cloud, ExecutionLocality::Cloud]);
        test([ExecutionLocality::Local]);
        test([ExecutionLocality::Local, ExecutionLocality::Local]);

        fn test(locality: impl IntoIterator<Item = ExecutionLocality>) {
            // arrange
            let intent = IntentConfigurationBuilder::new().build();
            let setup = Setup::combine(locality.into_iter().map(|locality| Setup {
                intent: intent.clone(),
                ..Setup::new().execution_locality(locality)
            }));

            // act
            let result = setup.resolve(&intent).unwrap();

            // assert
            assert_grpc_binding(
                &result,
                |_| { /* succeed if it is of the correct inner type `GrpcProvider`. */ },
            );
        }
    }

    #[test]
    fn resolve_system_registry_succeeds() {
        // arrange
        let intent = IntentConfiguration::new("system.registry".to_owned(), IntentKind::Inspect);
        let setup = Setup::new();
        let subject = setup.clone().build();

        // act
        let result = subject.resolve(&intent).unwrap();

        // assert
        if let RuntimeBinding::SystemInspect(context) = result {
            assert!(context.contains(&Arc::new(intent)));
            assert!(context.contains(&Arc::new(setup.intent)));
        } else {
            panic!()
        }
    }

    #[test]
    fn resolve_succeeds_for_system_discover() {
        // arrange
        let intent = IntentConfiguration::new("system.registry".to_owned(), IntentKind::Discover);

        // act
        let result = Setup::new().build().resolve(&intent).unwrap();

        // assert
        if let RuntimeBinding::SystemDiscover(url) = result {
            assert_eq!(Setup::STREAMING_URL.parse::<Url>().unwrap(), url);
        } else {
            panic!()
        }
    }

    #[test]
    fn resolve_succeeds_for_system_subscribe() {
        // arrange
        let intent = IntentConfiguration::new("system.registry".to_owned(), IntentKind::Subscribe);

        // act
        let result = Setup::new().build().resolve(&intent).unwrap();

        // assert
        if let RuntimeBinding::SystemSubscribe(_) = result {
            // assertions on the ESS itself are covered by integration tests.
        } else {
            panic!()
        }
    }

    #[test]
    fn when_refreshing_does_not_depend_on_previous_state() {
        // arrange
        const SERVICE_URL: &str = "http://service_b";
        let setup = Setup::new();
        let service_b = setup.service.clone().url(SERVICE_URL).build();
        let subject = setup.clone().build();

        // act
        subject.on_change([Change::Modify(&setup.intent, &HashSet::from([service_b]))].into_iter());

        // assert
        let result = subject.resolve(&setup.intent).unwrap();
        assert_grpc_binding(&result, |url| assert_eq!(&SERVICE_URL.parse::<Url>().unwrap(), url));
    }

    fn assert_grpc_binding(
        actual: &RuntimeBinding<ReusableProvider<GrpcProvider>>,
        assert: impl FnOnce(&Url),
    ) {
        if let RuntimeBinding::Remote(ReusableProvider { inner: GrpcProvider(url), .. }) = actual {
            assert(url);
        } else {
            panic!()
        }
    }

    fn assert_remote_fallback_binding(
        actual: &RuntimeBinding<ReusableProvider<GrpcProvider>>,
        assert_primary: impl FnOnce(&Url),
        assert_secondary: impl FnOnce(&Url),
    ) {
        if let RuntimeBinding::Fallback(primary, secondary) = actual {
            match (primary.as_ref(), secondary.as_ref()) {
                (
                    RuntimeBinding::Remote(ReusableProvider {
                        inner: GrpcProvider(primary), ..
                    }),
                    RuntimeBinding::Remote(ReusableProvider {
                        inner: GrpcProvider(secondary), ..
                    }),
                ) => {
                    assert_primary(primary);
                    assert_secondary(secondary);
                }
                _ => panic!(),
            }
        } else {
            panic!()
        }
    }

    #[derive(Clone)]
    struct Setup {
        intent: IntentConfiguration,
        service: ServiceConfigurationBuilder,
    }

    impl Setup {
        const STREAMING_URL: &str = "https://localhost:4243";

        fn new() -> Self {
            let intent = IntentConfigurationBuilder::new().build();
            let service = ServiceConfigurationBuilder::new();
            Setup { intent, service }
        }

        fn build(self) -> IntentBroker {
            let broker =
                IntentBroker::new(Self::STREAMING_URL.parse().unwrap(), StreamingEss::new());

            broker.on_change(
                [Change::Add(&self.intent, &HashSet::from([self.service.build()]))].into_iter(),
            );

            broker
        }

        fn execution_locality(mut self, execution_locality: ExecutionLocality) -> Self {
            self.service = self.service.execution_locality(execution_locality);
            self
        }

        fn service_name(mut self, name: &str) -> Self {
            self.service = self.service.name(name);
            self
        }

        fn combine(setups: impl IntoIterator<Item = Setup>) -> IntentBroker {
            let broker =
                IntentBroker::new("https://localhost:4243".parse().unwrap(), StreamingEss::new());

            let services_by_intent = setups.into_iter().fold(HashMap::new(), |mut acc, s| {
                acc.entry(s.intent.clone()).or_insert_with(Vec::new).push(s.service);
                acc
            });

            for (intent, services) in services_by_intent {
                broker.on_change(
                    [Change::Add(
                        &intent,
                        &services.clone().into_iter().map(|s| s.build()).collect(),
                    )]
                    .into_iter(),
                );
            }

            broker
        }
    }
}
