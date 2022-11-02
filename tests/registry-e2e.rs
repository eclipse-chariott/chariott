// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::time::Duration;

use chariott_common::proto::runtime::{
    intent_registration::Intent, intent_service_registration::ExecutionLocality,
};
use examples_common::chariott::{
    api::{Chariott, ChariottExt as _, GrpcChariott},
    registration::Builder as RegistrationBuilder,
};
use tokio_stream::StreamExt as _;
use uuid::Uuid;

#[tokio::test]
async fn when_provider_registers_notifies_registry_observers() -> anyhow::Result<()> {
    fn namespace_event(namespace: &str) -> String {
        format!("namespaces[{}]", namespace)
    }

    // arrange
    let namespace = format!("e2e.registration.{}", Uuid::new_v4());

    let builder = RegistrationBuilder::new(
        "registration.provider.e2e",
        "1.0.0",
        "http://test-url:7090".parse().unwrap(), // arbitrary url, the provider will never be invoked
        &namespace,
        [Intent::Inspect],
        ExecutionLocality::Local,
    );

    let mut subject = setup().await;

    // act
    let stream =
        subject.listen("system.registry", vec![namespace_event(&namespace).into()]).await?;

    builder.register_once(&mut None, true).await?;

    // assert
    let stream = stream.timeout(Duration::from_secs(5)).take(1).collect::<Vec<_>>().await;
    let result = stream.into_iter().next().unwrap();
    assert_eq!(namespace_event(&namespace).as_str(), result.unwrap().unwrap().id.as_ref());

    Ok(())
}

async fn setup() -> impl Chariott {
    GrpcChariott::connect().await.unwrap()
}
