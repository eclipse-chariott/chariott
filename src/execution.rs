// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::collections::HashMap;

use crate::connection_provider::{ConnectedProvider, ConnectionProvider};
use crate::ess::Ess;
use crate::registry::IntentConfiguration;
use async_recursion::async_recursion;
use chariott_common::proto::common::discover_fulfillment::Service;
use chariott_common::proto::common::SubscribeFulfillment;
use chariott_common::proto::{
    common::{
        fulfillment::Fulfillment as FulfillmentEnum, inspect_fulfillment::Entry,
        intent::Intent as IntentEnum, value::Value as ValueEnum, DiscoverFulfillment, Fulfillment,
        InspectFulfillment, Intent as IntentMessage, List, Value as ValueMessage,
    },
    provider::{FulfillRequest, FulfillResponse},
};
use chariott_common::query::regex_from_query;
use tonic::Status;
use url::Url;

const REGISTERED_INTENTS_KEY: &str = "registered_intents";

trait IterGroupingExt<K, V>: IntoIterator<Item = (K, V)> {
    fn group(self) -> HashMap<K, Vec<V>>;
}

impl<K, V, T> IterGroupingExt<K, V> for T
where
    T: IntoIterator<Item = (K, V)>,
    K: Eq + std::hash::Hash,
{
    fn group(self) -> HashMap<K, Vec<V>> {
        let mut groupings = HashMap::new();
        for (key, value) in self {
            groupings.entry(key).or_insert_with(Vec::new).push(value);
        }
        groupings
    }
}

#[derive(Clone)]
pub enum RuntimeBinding<T: ConnectionProvider> {
    Remote(T),
    Fallback(Box<RuntimeBinding<T>>, Box<RuntimeBinding<T>>),
    SystemInspect(Vec<IntentConfiguration>),
    SystemDiscover(Url),
    SystemSubscribe(Ess),
    #[cfg(test)]
    Test(tests::TestBinding),
}

impl<T> RuntimeBinding<T>
where
    T::ConnectedProvider: Send,
    T: ConnectionProvider + Send + 'static,
{
    #[async_recursion]
    pub async fn execute(self, arg: IntentMessage) -> Result<FulfillResponse, Status> {
        match self {
            RuntimeBinding::Remote(mut provider) => provider
                .connect()
                .await
                .map_err(|e| Status::unknown(format!("Failed to connect to provider: {}.", e)))?
                .fulfill(FulfillRequest { intent: Some(arg) })
                .await
                .map_err(|e| Status::unknown(format!("Error when invoking provider: '{}'.", e))),
            RuntimeBinding::Fallback(primary, secondary) => {
                match primary.execute(arg.clone()).await {
                    ok @ Ok(_) => ok,
                    Err(_) => secondary.execute(arg).await,
                }
            }
            RuntimeBinding::SystemInspect(intents) => {
                let fulfillment = match arg.intent {
                    Some(IntentEnum::Inspect(inspect_intent)) => {
                        let regex = regex_from_query(&inspect_intent.query);

                        let intents = intents
                            .into_iter()
                            .filter(|e| regex.is_match(e.namespace()))
                            .map(|ic| ic.into_namespaced_intent())
                            .group();

                        Ok(FulfillmentEnum::Inspect(InspectFulfillment {
                            entries: intents
                                .into_iter()
                                .map(|(path, intent_kinds)| Entry {
                                    path,
                                    items: HashMap::from([(
                                        REGISTERED_INTENTS_KEY.to_owned(),
                                        ValueMessage {
                                            value: Some(ValueEnum::List(List {
                                                value: intent_kinds
                                                    .into_iter()
                                                    .map(|intent_kind| ValueMessage {
                                                        value: Some(ValueEnum::String(
                                                            intent_kind.to_string(),
                                                        )),
                                                    })
                                                    .collect(),
                                            })),
                                        },
                                    )]),
                                })
                                .collect(),
                        }))
                    }
                    _ => Err(Status::invalid_argument(
                        "System does not support the specified intent.",
                    )),
                }?;

                Ok(FulfillResponse {
                    fulfillment: Some(Fulfillment { fulfillment: Some(fulfillment) }),
                })
            }
            RuntimeBinding::SystemDiscover(url) => {
                const SCHEMA_VERSION_STREAMING: &str = "chariott.streaming.v1";
                const SCHEMA_REFERENCE: &str = "grpc+proto";

                let fulfillment = FulfillmentEnum::Discover(DiscoverFulfillment {
                    services: vec![Service {
                        url: url.to_string(),
                        schema_kind: SCHEMA_REFERENCE.to_owned(),
                        schema_reference: SCHEMA_VERSION_STREAMING.to_owned(),
                        metadata: HashMap::new(),
                    }],
                });

                Ok(FulfillResponse {
                    fulfillment: Some(Fulfillment { fulfillment: Some(fulfillment) }),
                })
            }
            RuntimeBinding::SystemSubscribe(ess) => {
                match arg.intent {
                    Some(IntentEnum::Subscribe(subscribe_intent)) => ess.serve_subscriptions(
                        subscribe_intent.channel_id,
                        subscribe_intent.sources.into_iter().map(|s| s.into()),
                    ),
                    _ => Err(Status::invalid_argument(
                        "System does not support the specified intent.",
                    )),
                }?;

                Ok(FulfillResponse {
                    fulfillment: Some(Fulfillment {
                        fulfillment: Some(FulfillmentEnum::Subscribe(SubscribeFulfillment {})),
                    }),
                })
            }
            #[cfg(test)]
            RuntimeBinding::Test(item) => item.execute(arg),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::{
        connection_provider::GrpcProvider,
        registry::{IntentConfiguration, IntentKind},
    };
    use chariott_common::proto::common::{
        fulfillment::Fulfillment as FulfillmentEnum, DiscoverFulfillment,
        Fulfillment as FulfillmentMessage, InspectIntent, InvokeFulfillment,
    };
    use tonic::Code;

    use super::*;

    // Implementation for an Binding that returns an integer. Can be used for
    // test assertions. Assertions can be made either on the Ok(i32), or
    // Err(Code).
    #[derive(Clone, Debug, PartialEq)]
    pub struct TestBinding {
        result: Result<i32, Code>,
        expected_arg: Option<IntentEnum>,
    }

    impl TestBinding {
        pub fn new(result: Result<i32, Code>, expected_arg: Option<IntentEnum>) -> Self {
            Self { result, expected_arg }
        }

        pub fn from_result(result: Result<i32, Code>) -> Self {
            Self::new(result, None)
        }

        pub fn execute(&self, arg: IntentMessage) -> Result<FulfillResponse, Status> {
            if let Some(expected_arg) = self.expected_arg.clone() {
                assert_eq!(expected_arg, arg.intent.unwrap());
            }

            self.result
                .map(|value| FulfillResponse {
                    fulfillment: Some(Fulfillment {
                        fulfillment: Some(FulfillmentEnum::Invoke(InvokeFulfillment {
                            r#return: Some(ValueMessage { value: Some(ValueEnum::Int32(value)) }),
                        })),
                    }),
                })
                .map_err(|code| Status::new(code, "Some error"))
        }

        pub fn parse_result(fulfillment: Result<Fulfillment, Status>) -> Result<i32, Code> {
            match fulfillment {
                Ok(Fulfillment {
                    fulfillment:
                        Some(FulfillmentEnum::Invoke(InvokeFulfillment {
                            r#return: Some(ValueMessage { value: Some(ValueEnum::Int32(value)) }),
                        })),
                }) => Ok(value),
                Err(err) => Err(err.code()),
                _ => panic!(),
            }
        }
    }

    async fn execute_with_empty_intent(binding: RuntimeBinding<GrpcProvider>) -> Result<i32, Code> {
        TestBinding::parse_result(
            binding.execute(IntentMessage { intent: None }).await.map(|r| r.fulfillment.unwrap()),
        )
    }

    #[tokio::test]
    async fn fallback_binding_when_first_succeeds_should_return_response() {
        // arrange
        let primary = RuntimeBinding::Test(TestBinding::from_result(Ok(1)));
        let secondary = RuntimeBinding::Test(TestBinding::from_result(Ok(2)));
        let subject = RuntimeBinding::Fallback(Box::new(primary), Box::new(secondary));

        // act
        let result = execute_with_empty_intent(subject).await;

        // assert
        assert_eq!(1, result.unwrap())
    }

    #[tokio::test]
    async fn fallback_binding_when_first_fails_returns_second_response() {
        // arrange

        let primary = RuntimeBinding::Test(TestBinding::from_result(Err(Code::InvalidArgument)));
        let secondary = RuntimeBinding::Test(TestBinding::from_result(Ok(2)));
        let subject = RuntimeBinding::Fallback(Box::new(primary), Box::new(secondary));

        // act
        let result = execute_with_empty_intent(subject).await;

        // assert
        assert_eq!(2, result.unwrap())
    }

    #[tokio::test]
    async fn fallback_binding_when_both_fail_returns_second_error() {
        // arrange
        let primary = RuntimeBinding::Test(TestBinding::from_result(Err(Code::InvalidArgument)));
        let secondary = RuntimeBinding::Test(TestBinding::from_result(Err(Code::Internal)));
        let subject = RuntimeBinding::Fallback(Box::new(primary), Box::new(secondary));

        // act
        let result = execute_with_empty_intent(subject).await;

        // assert
        assert_eq!(Code::Internal, result.unwrap_err())
    }

    #[tokio::test]
    async fn system_inspect_binding_fails_with_non_supported_intent() {
        // arrange
        let binding = RuntimeBinding::SystemInspect(vec![]);

        // act
        let result = execute_with_empty_intent(binding).await;

        // assert
        assert!(result.is_err());
        assert_eq!(Code::InvalidArgument, result.unwrap_err());
    }

    #[tokio::test]
    async fn system_inspect_binding_succeeds() {
        const NAMESPACE_1: &str = "foo";
        const NAMESPACE_2: &str = "bar";
        const NAMESPACE_3: &str = "baz";

        test("*", &[NAMESPACE_1, NAMESPACE_2, NAMESPACE_3]).await;
        test("**", &[NAMESPACE_1, NAMESPACE_2, NAMESPACE_3]).await;
        test("bar", &[NAMESPACE_2]).await;
        test("ba*", &[NAMESPACE_2, NAMESPACE_3]).await;

        async fn test(query: &str, expected_namespaces: &[&str]) {
            // arrange
            let intent_configurations = [
                IntentConfiguration::new(NAMESPACE_1.to_owned(), IntentKind::Inspect),
                IntentConfiguration::new(NAMESPACE_1.to_owned(), IntentKind::Discover),
                IntentConfiguration::new(NAMESPACE_2.to_owned(), IntentKind::Discover),
                IntentConfiguration::new(NAMESPACE_3.to_owned(), IntentKind::Invoke),
            ];

            // act
            let inspection_items =
                execute_system_inspect(query, intent_configurations.into_iter().collect()).await;

            // assert
            let assert_group = |group_name: &str, expected_intents: &[&str]| {
                let index = inspection_items.iter().position(|el| el.path == group_name).unwrap();
                let actual_intents: Vec<String> = match inspection_items[index].items
                    [REGISTERED_INTENTS_KEY]
                    .value
                    .as_ref()
                    .unwrap()
                {
                    ValueEnum::List(l) => l,
                    _ => panic!("Not correct fulfillment"),
                }
                .value
                .iter()
                .map(|intent| match intent.value.as_ref().unwrap() {
                    ValueEnum::String(s) => s.clone(),
                    _ => panic!("Not correct fulfillment"),
                })
                .collect();

                for expected_intent in expected_intents {
                    assert!(actual_intents.iter().any(|actual_intent| {
                        expected_intent.eq_ignore_ascii_case(actual_intent)
                    }));
                }
            };

            let assert_group_is_none = |group_name| {
                assert!(!inspection_items
                    .iter()
                    .any(|item| item.path.eq_ignore_ascii_case(group_name)));
            };

            if expected_namespaces.contains(&NAMESPACE_1) {
                assert_group(NAMESPACE_1, &["Inspect", "Discover"]);
            } else {
                assert_group_is_none(NAMESPACE_1);
            }

            if expected_namespaces.contains(&NAMESPACE_2) {
                assert_group(NAMESPACE_2, &["Discover"]);
            } else {
                assert_group_is_none(NAMESPACE_2);
            }

            if expected_namespaces.contains(&NAMESPACE_3) {
                assert_group(NAMESPACE_3, &["Invoke"]);
            } else {
                assert_group_is_none(NAMESPACE_3);
            }
        }
    }

    #[tokio::test]
    async fn system_discover_binding_succeeds() {
        // arrange
        let url: Url = "http://localhost:4243".parse().unwrap();

        // act
        let result = RuntimeBinding::<GrpcProvider>::SystemDiscover(url.clone())
            .execute(IntentMessage { intent: None })
            .await
            .unwrap();

        // assert
        assert_eq!(
            FulfillResponse {
                fulfillment: Some(FulfillmentMessage {
                    fulfillment: Some(FulfillmentEnum::Discover(DiscoverFulfillment {
                        services: vec![Service {
                            url: url.to_string(),
                            schema_kind: "grpc+proto".to_owned(),
                            schema_reference: "chariott.streaming.v1".to_owned(),
                            metadata: HashMap::new(),
                        }],
                    })),
                }),
            },
            result
        );
    }

    async fn execute_system_inspect(query: &str, intents: Vec<IntentConfiguration>) -> Vec<Entry> {
        let response = RuntimeBinding::<GrpcProvider>::SystemInspect(intents)
            .execute(IntentMessage {
                intent: Some(IntentEnum::Inspect(InspectIntent { query: query.to_owned() })),
            })
            .await;

        match response.unwrap().fulfillment.unwrap().fulfillment {
            Some(FulfillmentEnum::Inspect(InspectFulfillment { entries })) => entries,
            _ => panic!("Wrong fulfillment"),
        }
    }
}
