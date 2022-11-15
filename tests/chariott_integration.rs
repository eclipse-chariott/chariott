// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Instant;

use async_trait::async_trait;
use chariott::registry::{
    ExecutionLocality, IntentConfiguration, IntentKind, ServiceConfiguration, ServiceId,
};
use chariott::streaming::StreamingEss;
use chariott::{chariott_grpc::ChariottServer, registry::Registry, IntentBroker};
use chariott_common::error::{Error, ResultExt as _};
use chariott_common::shutdown::RouterExt as _;
use chariott_proto::{
    common::{IntentEnum, IntentMessage},
    runtime::{chariott_service_server::ChariottService, FulfillRequest, FulfillResponse},
    streaming::channel_service_server::ChannelServiceServer,
};
use common::get_uuid;
use examples_common::chariott::{
    api::{Chariott, ChariottCommunication},
    value::Value,
};
use provider::Provider;
use tokio::task::spawn;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tonic::{Request, Response};

mod common;
mod provider;

#[tokio::test]
async fn when_fulfill_invoke_intent_returns_response() -> anyhow::Result<()> {
    // arrange
    const VALUE: &str = "some_value";

    let mut subject = setup(Provider::new().with_on_invoke(|_| Some(VALUE.into()))).await;

    // act
    let response = subject.invoke(subject.namespace.clone(), "foo", vec![]).await?;

    // assert
    assert_eq!(VALUE, response.as_str().unwrap());

    Ok(())
}

#[tokio::test]
async fn when_two_providers_invoke_intent_prioritizes_cloud_response() -> anyhow::Result<()> {
    // arrange
    const VALUE: &str = "some_value";

    let local = Provider::new().with_on_invoke(|_| Some(10.into()));
    let cloud = Provider::new().with_on_invoke(|_| Some(VALUE.into()));

    let mut subject =
        setup_multiple([ProviderSetup::local(local), ProviderSetup::cloud(cloud)]).await;

    // act
    let response = subject.invoke(subject.namespace.clone(), "foo", vec![]).await?;

    // assert
    assert_eq!(VALUE, response.as_str().unwrap());

    Ok(())
}

#[tokio::test]
async fn when_two_providers_invoke_intent_falls_back_to_local_response() -> anyhow::Result<()> {
    // arrange
    const VALUE: i32 = 10;

    let local = Provider::new().with_on_invoke(|_| Some(VALUE.into()));
    let cloud = Provider::new().with_on_invoke(|_| panic!("Cloud invocation failed"));

    let mut subject =
        setup_multiple([ProviderSetup::local(local), ProviderSetup::cloud(cloud)]).await;

    // act
    let response = subject.invoke(subject.namespace.clone(), "foo", vec![]).await?;

    // assert
    assert_eq!(VALUE, response.to_i32().unwrap());

    Ok(())
}

#[tokio::test]
async fn when_invoke_intent_arguments_are_passed_correctly() -> anyhow::Result<()> {
    // arrange
    const VALUE_1: i32 = 10;
    const VALUE_2: bool = true;
    const COMMAND: &str = "foo";

    let mut subject = setup(Provider::new().with_on_invoke(|intent| {
        assert_eq!(COMMAND, intent.command);
        assert_eq!(VALUE_1, Value::try_from(intent.args[0].clone()).unwrap().to_i32().unwrap());
        assert_eq!(VALUE_2, Value::try_from(intent.args[1].clone()).unwrap().to_bool().unwrap());
        Some(Value::NULL)
    }))
    .await;

    // act
    let response = subject
        .invoke(subject.namespace.clone(), COMMAND, vec![VALUE_1.into(), VALUE_2.into()])
        .await?;

    // assert
    assert_eq!(Value::NULL, response);

    Ok(())
}

#[tokio::test]
async fn when_cancelled_shuts_down_provider() -> anyhow::Result<()> {
    // arrange
    let cancellation_token = CancellationToken::new();
    let handle = spawn(
        Server::builder()
            .add_service(ChannelServiceServer::new(StreamingEss::new()))
            .serve_with_cancellation(
                format!("0.0.0.0:{}", get_port()).parse().unwrap(),
                cancellation_token.child_token(),
            ),
    );

    // act
    cancellation_token.cancel();

    // assert
    assert!(handle.await.is_ok());

    Ok(())
}

struct Subject {
    namespace: String,
    subject: ChariottServer<IntentBroker>,
}

struct ProviderSetup {
    provider: Provider,
    name: Box<str>,
    port: u16,
    locality: ExecutionLocality,
}

impl ProviderSetup {
    pub fn local(provider: Provider) -> Self {
        Self { provider, name: get_uuid(), port: get_port(), locality: ExecutionLocality::Local }
    }

    pub fn cloud(provider: Provider) -> Self {
        Self { locality: ExecutionLocality::Cloud, ..Self::local(provider) }
    }
}

async fn setup(provider: Provider) -> Subject {
    setup_multiple([ProviderSetup::local(provider)]).await
}

async fn setup_multiple(providers: impl IntoIterator<Item = ProviderSetup>) -> Subject {
    let namespace = "sdv.integration".to_owned();
    let broker = IntentBroker::new("https://localhost:4243".parse().unwrap(), StreamingEss::new());
    let mut registry = Registry::new(broker.clone(), Default::default());

    for ProviderSetup { provider, port, name, locality } in providers {
        let url = provider.serve(port).await;

        registry
            .upsert(
                ServiceConfiguration::new(ServiceId::new(name, "1.0.0"), url, locality),
                vec![IntentConfiguration::new(namespace.clone(), IntentKind::Invoke)],
                Instant::now(),
            )
            .unwrap();
    }

    Subject { namespace, subject: ChariottServer::new(registry, broker) }
}

#[async_trait]
impl ChariottCommunication for Subject {
    async fn fulfill(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        intent: IntentEnum,
    ) -> Result<Response<FulfillResponse>, Error> {
        self.subject
            .fulfill(Request::new(FulfillRequest {
                namespace: namespace.into().into(),
                intent: Some(IntentMessage { intent: Some(intent) }),
            }))
            .await
            .map_err_with("Intent fulfillment failed.")
    }
}

pub fn get_port() -> u16 {
    static PORT: AtomicU16 = AtomicU16::new(40040);
    PORT.fetch_add(1, Ordering::Relaxed)
}
