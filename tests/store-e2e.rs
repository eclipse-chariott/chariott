// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{collections::HashSet, error::Error as _, time::Duration};

use chariott_common::error::Error;
use common::get_uuid;
use examples_common::chariott::{
    api::{Chariott, ChariottExt, Event, GrpcChariott},
    value::Value,
};
use tokio::time::{sleep_until, Instant};
use tokio_stream::StreamExt as _;

mod common;

const KV_NAMESPACE: &str = "sdv.kvs";

#[tokio::test]
async fn when_key_does_not_exist_returns_none() -> Result<(), anyhow::Error> {
    // arrange
    let mut chariott = setup().await;

    // act
    let response = chariott.read(KV_NAMESPACE, get_uuid()).await?;

    // assert
    assert_eq!(None, response);

    Ok(())
}

#[tokio::test]
async fn when_writing_value_returns_value_on_read() -> Result<(), anyhow::Error> {
    // arrange
    let mut chariott = setup().await;
    let value: Value = "some_value".into();
    let key = get_uuid();

    // act
    chariott.write(KV_NAMESPACE, key.clone(), value.clone()).await?;
    let response = chariott.read(KV_NAMESPACE, key).await?;

    // assert
    assert_eq!(Some(value), response);

    Ok(())
}

#[tokio::test]
async fn when_provider_does_not_exist_returns_error() {
    // arrange
    let mut chariott = setup().await;

    // act
    let response = chariott.read("sdv.does_not_exist", "key").await;

    // assert
    assert!(response.unwrap_err().source().unwrap().to_string().contains("No provider found."));
}

#[tokio::test]
async fn when_writing_while_streaming_publishes_value() -> Result<(), anyhow::Error> {
    // arrange
    let mut chariott = setup().await;
    let key = get_uuid();
    let value: Value = 10.into();

    // act
    let response_stream = chariott.listen(KV_NAMESPACE, [key.clone()]).await?;
    chariott.write(KV_NAMESPACE, key.clone(), value.clone()).await?;

    // assert
    let event = response_stream.take(1).collect::<Vec<Result<Event, Error>>>().await;
    let event = event[0].as_ref().unwrap();
    assert_eq!(value, event.data);
    assert_eq!(key, event.id);
    assert_eq!(1, event.seq);

    Ok(())
}

#[tokio::test]
async fn when_streaming_increases_sequence_number() -> Result<(), anyhow::Error> {
    // arrange
    const NUMBER_OF_EVENTS: u64 = 5;
    let mut chariott = setup().await;
    let key = get_uuid();

    // act
    let response_stream = chariott.listen(KV_NAMESPACE, [key.clone()]).await?;
    for _ in 0..NUMBER_OF_EVENTS {
        chariott.write(KV_NAMESPACE, key.clone(), 10.into()).await?;
    }

    // assert
    let actual_sequence_numbers = response_stream
        .map(|e| e.map(|e| e.seq))
        .take(NUMBER_OF_EVENTS as _)
        .collect::<Vec<Result<u64, Error>>>()
        .await
        .into_iter()
        .collect::<Result<HashSet<u64>, Error>>()?;

    // Events may be delivered out of order.
    for i in 1..=NUMBER_OF_EVENTS {
        assert!(actual_sequence_numbers.contains(&i));
    }

    Ok(())
}

#[tokio::test]
async fn when_writing_to_a_different_key_does_not_publish_value() -> Result<(), anyhow::Error> {
    // arrange
    let mut chariott = setup().await;

    // act
    let mut response_stream = chariott.listen(KV_NAMESPACE, [get_uuid()]).await?;
    chariott.write(KV_NAMESPACE, get_uuid(), 10.into()).await?;

    // assert

    // succeed if the stream does not receive an event for five seconds.
    let timeout = Instant::now() + Duration::from_secs(5);
    tokio::select! {
        _ = response_stream.next() => {
            panic!("Did not expect to receive an event.")
        }
        _ = sleep_until(timeout) => {
            // No event received. Continue.
        }
    }

    Ok(())
}

async fn setup() -> impl Chariott {
    GrpcChariott::connect().await.unwrap()
}
