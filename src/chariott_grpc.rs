// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::sync::{Arc, RwLock};
use std::time::Instant;

use tonic::{async_trait, Request, Response, Status};
use url::Url;

use crate::intent_broker::IntentBroker;
use crate::registry::{
    ExecutionLocality, IntentConfiguration, IntentKind, Observer, Registry, ServiceConfiguration,
    ServiceId,
};
use chariott_common::proto::*;
use chariott_common::proto::{runtime as runtime_api, runtime::IntentRegistration};

// Enums are mapped to i32 in proto, we map
// the values here to the actual values in the proto.
// When new intents are added, they need to be
// added here. Tests have been put in place
// to ensure the lists are kept in sync.
const INTENT_MAPPING_DISCOVER: i32 = 0;
const INTENT_MAPPING_INSPECT: i32 = 1;
const INTENT_MAPPING_READ: i32 = 2;
const INTENT_MAPPING_WRITE: i32 = 3;
const INTENT_MAPPING_INVOKE: i32 = 4;
const INTENT_MAPPING_SUBSCRIBE: i32 = 5;

pub struct ChariottServer<T: Observer> {
    broker: IntentBroker,
    registry: Arc<RwLock<Registry<T>>>,
}

impl<T: Observer> ChariottServer<T> {
    pub fn new(registry: Registry<T>, broker: IntentBroker) -> Self {
        Self { registry: Arc::new(RwLock::new(registry)), broker }
    }

    pub fn registry_do<U>(&self, f: impl FnOnce(&mut Registry<T>) -> U) -> U {
        let mut registry = self.registry.write().unwrap();
        f(&mut registry)
    }

    fn create_configruation_from_registration(
        intent: IntentRegistration,
    ) -> Result<IntentConfiguration, Status> {
        ChariottServer::<T>::map_intent_value(intent.intent)
            .map(|kind| IntentConfiguration::new(intent.namespace, kind))
    }

    fn map_intent_value(intent_value: i32) -> Result<IntentKind, Status> {
        match intent_value {
            INTENT_MAPPING_DISCOVER => Ok(IntentKind::Discover),
            INTENT_MAPPING_INSPECT => Ok(IntentKind::Inspect),
            INTENT_MAPPING_READ => Ok(IntentKind::Read),
            INTENT_MAPPING_WRITE => Ok(IntentKind::Write),
            INTENT_MAPPING_INVOKE => Ok(IntentKind::Invoke),
            INTENT_MAPPING_SUBSCRIBE => Ok(IntentKind::Subscribe),
            _ => Err(Status::invalid_argument("No such intent known.")),
        }
    }

    fn map_intent_variant(intent: &common::intent::Intent) -> IntentKind {
        match intent {
            common::intent::Intent::Discover(_) => IntentKind::Discover,
            common::intent::Intent::Inspect(_) => IntentKind::Inspect,
            common::intent::Intent::Read(_) => IntentKind::Read,
            common::intent::Intent::Write(_) => IntentKind::Write,
            common::intent::Intent::Invoke(_) => IntentKind::Invoke,
            common::intent::Intent::Subscribe(_) => IntentKind::Subscribe,
        }
    }
}

#[async_trait]
impl<T: Observer + Send + Sync + 'static> runtime_api::chariott_service_server::ChariottService
    for ChariottServer<T>
{
    async fn announce(
        &self,
        request: Request<runtime_api::AnnounceRequest>,
    ) -> Result<Response<runtime_api::AnnounceResponse>, Status> {
        let service = request
            .into_inner()
            .service
            .ok_or_else(|| Status::new(tonic::Code::InvalidArgument, "service is required"))?;
        let svc_cfg = resolve_service_configuration(service)?;
        let registration_state = if self.registry.write().unwrap().touch(&svc_cfg, Instant::now()) {
            tracing::debug!("Service {:#?} already announced", svc_cfg);
            runtime_api::RegistrationState::NotChanged
        } else {
            tracing::debug!("Service {:#?} not yet announced", svc_cfg);
            runtime_api::RegistrationState::Announced
        };

        Ok(Response::new(runtime_api::AnnounceResponse {
            registration_state: registration_state as i32,
        }))
    }

    async fn register(
        &self,
        request: Request<runtime_api::RegisterRequest>,
    ) -> Result<Response<runtime_api::RegisterResponse>, Status> {
        let request = request.into_inner();
        let service =
            request.service.ok_or_else(|| Status::invalid_argument("service is required"))?;
        let svc_cfg = resolve_service_configuration(service)?;
        let intents: Result<Vec<_>, _> = request
            .intents
            .into_iter()
            .map(ChariottServer::<T>::create_configruation_from_registration)
            .collect();
        self.registry
            .write()
            .unwrap()
            .upsert(svc_cfg, intents?, Instant::now())
            .map_err(|e| Status::unknown(e.message()))?;
        Ok(Response::new(runtime_api::RegisterResponse {}))
    }

    async fn fulfill(
        &self,
        request: Request<runtime_api::FulfillRequest>,
    ) -> Result<Response<runtime_api::FulfillResponse>, Status> {
        let request = request.into_inner();
        let intent =
            request.intent.ok_or_else(|| Status::invalid_argument("intent is required"))?;

        let config = IntentConfiguration::new(
            request.namespace,
            match intent.intent {
                Some(ref intent) => Ok(ChariottServer::<T>::map_intent_variant(intent)),
                None => Err(Status::invalid_argument("Intent is not known.")),
            }?,
        );

        #[cfg(not(test))]
        let broker = &self.broker;
        #[cfg(test)]
        _ = self.broker; // Suppress dead code warning when test feature is active.
        #[cfg(test)]
        let broker = tests::MockBroker;

        let binding =
            broker.resolve(&config).ok_or_else(|| Status::not_found("No provider found."))?;

        let response = binding.execute(intent).await?;

        Ok(tonic::Response::new(runtime_api::FulfillResponse { fulfillment: response.fulfillment }))
    }
}

fn resolve_service_configuration(
    service: runtime_api::IntentServiceRegistration,
) -> Result<ServiceConfiguration, Status> {
    map_locality_value(service.locality)
        .and_then(|locality| {
            Url::parse(&service.url)
                .map_err(|_| Status::invalid_argument("Service URL is not valid."))
                .map(|url| (locality, url))
        })
        .map(|(locality, url)| {
            ServiceConfiguration::new(
                ServiceId::new(service.name.into_boxed_str(), service.version.into_boxed_str()),
                url,
                locality,
            )
        })
}

fn map_locality_value(locality: i32) -> Result<ExecutionLocality, Status> {
    match locality {
        0 => Ok(ExecutionLocality::Local),
        1 => Ok(ExecutionLocality::Cloud),
        _ => Err(Status::invalid_argument("No such intent known.")),
    }
}

#[cfg(test)]
mod tests {
    use crate::execution::RuntimeBinding;
    use crate::registry::{Change, Observer, Registry};
    use crate::streaming::StreamingEss;
    use crate::{connection_provider::GrpcProvider, execution::tests::TestBinding};
    use chariott_common::proto::{
        common, runtime as runtime_api,
        runtime::{
            chariott_service_server::ChariottService, intent_registration, AnnounceRequest,
            IntentRegistration, IntentServiceRegistration, RegisterRequest, RegistrationState,
        },
    };
    use tonic::Code;

    use super::*;

    #[tokio::test]
    async fn test_service_announcement() {
        let server = setup();
        let request = create_announce_request();
        let response = server.announce(Request::new(request)).await.unwrap();
        let response = response.into_inner();
        assert_eq!(response.registration_state, RegistrationState::Announced as i32);
    }

    #[tokio::test]
    async fn test_register_service_with_intents() {
        let server = setup();
        let request = create_register_request();
        let _response = server.register(Request::new(request)).await.unwrap();
        let request = create_announce_request();
        let response = server.announce(Request::new(request)).await.unwrap();
        let response = response.into_inner();
        assert_eq!(response.registration_state, RegistrationState::NotChanged as i32);
    }

    #[tokio::test]
    async fn test_register_service_twice_doesnt_throw_error() {
        let server = setup();
        let request = create_register_request();
        let _response = server.register(Request::new(request.clone())).await.unwrap();
        let _response = server.register(Request::new(request)).await.unwrap();
    }

    #[tokio::test]
    async fn test_register_service_twice_with_different_intents() {
        let server = setup();

        let request = create_register_request();

        _ = server.register(Request::new(request.clone())).await.unwrap();
        assert_eq!(server.registry.read().unwrap().count_external_intents(), 2);

        let request = create_register_request_with_different_namespace();
        _ = server.register(Request::new(request)).await.unwrap();
        assert_eq!(server.registry.read().unwrap().count_external_intents(), 3);
    }

    #[tokio::test]
    async fn when_registering_unknown_intent_should_return_invalid_argument_error() {
        // arrange
        let subject = setup();
        let request = RegisterRequest {
            intents: vec![IntentRegistration { namespace: "test".to_owned(), intent: -1 }],
            ..create_register_request()
        };

        // act
        let result = subject.register(Request::new(request)).await;

        // assert
        assert_eq!(Code::InvalidArgument, result.unwrap_err().code())
    }

    #[test]
    fn intent_match_failure_are_caught() {
        assert!(ChariottServer::<IntentBroker>::map_intent_value(-1).is_err());
    }

    #[test]
    fn test_intent_mappings() {
        // The match is only here to catch adding of new intents.
        // Devs adding new intents are required to update the
        // match arm as well as the mapping validations below
        // ensuring the values map with the values from the proto.
        let intent = IntentKind::Discover;
        match intent {
            IntentKind::Discover => {}
            IntentKind::Inspect => {}
            IntentKind::Read => {}
            IntentKind::Write => {}
            IntentKind::Invoke => {}
            IntentKind::Subscribe => {}
        }

        fn test(intent_value: i32, kind: IntentKind) {
            assert_eq!(
                ChariottServer::<IntentBroker>::map_intent_value(intent_value).unwrap(),
                kind
            );
        }

        test(INTENT_MAPPING_DISCOVER, IntentKind::Discover);
        test(INTENT_MAPPING_INSPECT, IntentKind::Inspect);
        test(INTENT_MAPPING_READ, IntentKind::Read);
        test(INTENT_MAPPING_WRITE, IntentKind::Write);
        test(INTENT_MAPPING_INVOKE, IntentKind::Invoke);
        test(INTENT_MAPPING_SUBSCRIBE, IntentKind::Subscribe);
    }

    #[test]
    fn test_intent_proto_mappings() {
        // The match is only here to catch adding of new intents.
        // Devs adding new intents are required to update the
        // match arm as well as the mapping validations below
        // ensuring the values map with the values from the proto.
        let intent = IntentKind::Discover;
        match intent {
            IntentKind::Discover => {}
            IntentKind::Inspect => {}
            IntentKind::Read => {}
            IntentKind::Write => {}
            IntentKind::Invoke => {}
            IntentKind::Subscribe => {}
        }

        // mapping validations
        assert_eq!(intent_registration::Intent::Discover as i32, INTENT_MAPPING_DISCOVER);
        assert_eq!(intent_registration::Intent::Inspect as i32, INTENT_MAPPING_INSPECT);
        assert_eq!(intent_registration::Intent::Read as i32, INTENT_MAPPING_READ);
        assert_eq!(intent_registration::Intent::Write as i32, INTENT_MAPPING_WRITE);
        assert_eq!(intent_registration::Intent::Invoke as i32, INTENT_MAPPING_INVOKE);
        assert_eq!(intent_registration::Intent::Subscribe as i32, INTENT_MAPPING_SUBSCRIBE);
    }

    #[test]
    fn test_map_locality_value() {
        assert_eq!(
            map_locality_value(ExecutionLocality::Local as i32).unwrap(),
            crate::registry::ExecutionLocality::Local
        );
        assert_eq!(
            map_locality_value(ExecutionLocality::Cloud as i32).unwrap(),
            crate::registry::ExecutionLocality::Cloud
        );
        assert_eq!(map_locality_value(-1).unwrap_err().code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn fulfill_ensures_binding_is_executed() {
        // arrange
        let subject = setup();

        // act
        let result = subject
            .fulfill(Request::new(runtime_api::FulfillRequest {
                namespace: "system".to_owned(),
                intent: Some(create_fulfill()),
            }))
            .await;

        // assert
        assert_eq!(
            MockBroker::RETURN_VALUE,
            TestBinding::parse_result(result.map(|r| r.into_inner().fulfillment.unwrap())).unwrap()
        );
    }

    #[tokio::test]
    async fn fulfill_returns_error_if_intent_not_set() {
        // arrange
        let subject = setup();

        // act
        let result = subject
            .fulfill(Request::new(runtime_api::FulfillRequest {
                namespace: "system".to_owned(),
                intent: None,
            }))
            .await;

        // assert
        assert_eq!(Code::InvalidArgument, result.unwrap_err().code());
    }

    #[test]
    fn test_map_intent_variant() {
        use common::intent::Intent;
        use common::*;

        // The match is only here to catch adding of new intents. Devs adding
        // new intents are required to update the match arm as well as the
        // mapping validations below ensuring the intent kinds map with the
        // values from the proto.
        let intent = IntentKind::Discover;
        match intent {
            IntentKind::Discover => {}
            IntentKind::Inspect => {}
            IntentKind::Read => {}
            IntentKind::Write => {}
            IntentKind::Invoke => {}
            IntentKind::Subscribe => {}
        }

        // assert
        for (intent, expected) in [
            (Intent::Discover(DiscoverIntent {}), IntentKind::Discover),
            (Intent::Inspect(InspectIntent { query: "".to_owned() }), IntentKind::Inspect),
            (Intent::Read(ReadIntent { key: "".to_owned() }), IntentKind::Read),
            (Intent::Write(WriteIntent { key: "".to_owned(), value: None }), IntentKind::Write),
            (
                Intent::Invoke(InvokeIntent { command: "".to_owned(), args: vec![] }),
                IntentKind::Invoke,
            ),
            (
                Intent::Subscribe(SubscribeIntent { channel_id: "".to_owned(), sources: vec![] }),
                IntentKind::Subscribe,
            ),
        ] {
            assert_eq!(expected, ChariottServer::<IntentBroker>::map_intent_variant(&intent));
        }
    }

    pub struct MockBroker;

    impl MockBroker {
        const RETURN_VALUE: i32 = 10;

        pub fn resolve(&self, _: &IntentConfiguration) -> Option<RuntimeBinding<GrpcProvider>> {
            Some(RuntimeBinding::Test(TestBinding::new(
                Ok(Self::RETURN_VALUE),
                Some(create_fulfill().intent.unwrap()),
            )))
        }
    }

    impl Observer for MockBroker {
        fn on_change<'a>(&self, _: impl IntoIterator<Item = Change<'a>>) {
            todo!()
        }
    }

    fn create_fulfill() -> common::Intent {
        common::Intent {
            intent: Some(common::intent::Intent::Invoke(common::InvokeIntent {
                command: "test".to_owned(),
                args: vec![common::Value { value: Some(common::value::Value::Int32(1)) }],
            })),
        }
    }

    fn setup() -> ChariottServer<IntentBroker> {
        let broker =
            IntentBroker::new("https://localhost:4243".parse().unwrap(), StreamingEss::new());
        ChariottServer::new(Registry::new(broker.clone(), Default::default()), broker)
    }

    fn create_announce_request() -> AnnounceRequest {
        AnnounceRequest {
            service: Some(IntentServiceRegistration {
                name: "test".to_string(),
                version: "1.0".to_string(),
                url: "http://test.com".to_string(),
                locality: ExecutionLocality::Local as i32,
            }),
        }
    }

    fn create_register_request() -> RegisterRequest {
        RegisterRequest {
            service: Some(IntentServiceRegistration {
                name: "test".to_string(),
                version: "1.0".to_string(),
                url: "http://test.com".to_string(),
                locality: ExecutionLocality::Local as i32,
            }),
            intents: vec![
                IntentRegistration {
                    namespace: "foo".to_string(),
                    intent: runtime_api::intent_registration::Intent::Discover as i32,
                },
                IntentRegistration {
                    namespace: "bar".to_string(),
                    intent: runtime_api::intent_registration::Intent::Discover as i32,
                },
            ],
        }
    }

    fn create_register_request_with_different_namespace() -> RegisterRequest {
        RegisterRequest {
            service: Some(IntentServiceRegistration {
                name: "test".to_string(),
                version: "1.0".to_string(),
                url: "http://test.com".to_string(),
                locality: ExecutionLocality::Local as i32,
            }),
            intents: vec![
                IntentRegistration {
                    namespace: "foo".to_string(),
                    intent: runtime_api::intent_registration::Intent::Discover as i32,
                },
                IntentRegistration {
                    namespace: "bar".to_string(),
                    intent: runtime_api::intent_registration::Intent::Discover as i32,
                },
                IntentRegistration {
                    namespace: "baz".to_string(),
                    intent: runtime_api::intent_registration::Intent::Discover as i32,
                },
            ],
        }
    }
}
