// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::time::Duration;

use chariott_common::proto::runtime::{
    intent_registration::Intent, intent_service_registration::ExecutionLocality,
};
use common::get_uuid;
use examples_common::chariott::{
    api::{Chariott, GrpcChariott},
    registration::Builder as RegistrationBuilder,
};
use tokio::time::*;

mod common;

#[tokio::test]
async fn expired_registrations_are_pruned_after_ttl() -> Result<(), anyhow::Error> {
    // arrange
    let namespace = format!("e2e.registration.{}", get_uuid());

    let builder = RegistrationBuilder::new(
        "e2e",
        "1.0.0",
        "http://localhost/".parse().unwrap(),
        &namespace,
        [Intent::Inspect],
        ExecutionLocality::Local,
    );

    let mut chariott = setup().await;

    // act
    builder.register_once(&mut None, true).await?;

    let initial_entries = chariott.inspect("system.registry", namespace.clone()).await?;
    let ttl = Duration::from_secs(env!("CHARIOTT_REGISTRY_TTL_SECS").parse::<u64>().unwrap() + 1);
    sleep(ttl).await;
    let entries = chariott.inspect("system.registry", namespace).await?;

    // assert
    assert_eq!(1, initial_entries.len());
    assert_eq!(0, entries.len());

    Ok(())
}

async fn setup() -> impl Chariott {
    GrpcChariott::connect().await.unwrap()
}
