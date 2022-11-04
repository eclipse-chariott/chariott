// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{ops::Deref, sync::Arc, time::SystemTime};

use crate::proto::{
    common::Value as ValueMessage,
    common::{value::Value as ValueEnum, SubscribeFulfillment, SubscribeIntent},
    streaming::{channel_service_server::ChannelService, Event, OpenRequest},
};
use async_trait::async_trait;
use tokio::spawn;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use uuid::Uuid;

type EventSubSystem<T> = ess::EventSubSystem<Box<str>, Box<str>, T, Result<Event, Status>>;

/// [`StreamingEss`](StreamingEss) integrates the reusable
/// [`EventSubSystem`](ess::EventSubSystem) component with the Chariott gRPC
/// streaming contract. Cloning [`StreamingEss`](StreamingEss) is cheap, it will
/// not create a new instance but refer to the same underlying instance instead.
#[derive(Clone)]
pub struct StreamingEss<T>(Arc<EventSubSystem<T>>);

impl<T: Clone> StreamingEss<T> {
    pub fn new() -> Self {
        Self(Arc::new(EventSubSystem::new()))
    }
}

impl<T: Clone> Default for StreamingEss<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + 'static> StreamingEss<T> {
    pub fn serve_subscriptions(
        &self,
        subscribe_intent: SubscribeIntent,
        into_value: fn(T) -> ValueEnum,
    ) -> Result<SubscribeFulfillment, Status> {
        let subscriptions = self
            .register_subscriptions(
                subscribe_intent.channel_id.into(),
                subscribe_intent.sources.into_iter().map(|s| s.into()),
            )
            .map_err(|_| Status::failed_precondition("The specified client does not exist."))?;

        for subscription in subscriptions {
            let source = subscription.event_id().to_string();

            spawn(subscription.serve(move |data, seq| {
                Ok(Event {
                    source: source.clone(),
                    value: Some(ValueMessage { value: Some(into_value(data)) }),
                    seq,
                    timestamp: Some(SystemTime::now().into()),
                })
            }));
        }

        Ok(SubscribeFulfillment {})
    }
}

#[async_trait]
impl<T> ChannelService for StreamingEss<T>
where
    T: Clone + Send + Sync + 'static,
{
    type OpenStream = ReceiverStream<Result<Event, Status>>;

    async fn open(
        &self,
        _: tonic::Request<OpenRequest>,
    ) -> Result<Response<Self::OpenStream>, Status> {
        const METADATA_KEY: &str = "x-chariott-channel-id";

        let id = Uuid::new_v4().to_string();
        let (_, receiver_stream) = self.read_events(id.clone().into());
        let mut response = Response::new(receiver_stream);
        response.metadata_mut().insert(METADATA_KEY, id.try_into().unwrap());
        Ok(response)
    }
}

impl<T> Deref for StreamingEss<T> {
    type Target = EventSubSystem<T>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::proto::{
        common::Value as ValueMessage,
        common::{value::Value as ValueEnum, SubscribeIntent},
        streaming::{channel_service_server::ChannelService, OpenRequest},
    };
    use tokio_stream::StreamExt as _;
    use tonic::{Code, Request};

    use super::StreamingEss;

    #[tokio::test]
    async fn open_should_set_channel_id() {
        // arrange
        let subject = setup();

        // act
        let response = subject.open(Request::new(OpenRequest {})).await.unwrap();

        // assert
        assert!(!response.metadata().get("x-chariott-channel-id").unwrap().is_empty());
    }

    #[tokio::test]
    async fn serve_subscriptions_should_serve_subscription_for_event() {
        // arrange
        const EVENT_A: &str = "test-event-a";
        const EVENT_B: &str = "test-event-b";

        let subject = setup();
        let response = subject.open(Request::new(OpenRequest {})).await.unwrap();
        let channel_id =
            response.metadata().get("x-chariott-channel-id").unwrap().to_str().unwrap().into();

        // act
        subject
            .serve_subscriptions(
                SubscribeIntent { channel_id, sources: vec![EVENT_A.into(), EVENT_B.into()] },
                |_| ValueEnum::Null(0),
            )
            .unwrap();

        // assert
        subject.publish(EVENT_A, ());
        subject.publish(EVENT_B, ());

        let result = response
            .into_inner()
            .timeout(Duration::from_millis(100))
            .take_while(|e| e.is_ok())
            .map(|e| e.unwrap().unwrap())
            .collect::<Vec<_>>()
            .await;

        let expected_sources = [EVENT_A, EVENT_B];

        assert_eq!(expected_sources.len(), result.len());

        for (expected_source, actual) in expected_sources.into_iter().zip(result) {
            assert_eq!(expected_source, actual.source.as_str());
            assert_eq!(1, actual.seq);
            assert_eq!(Some(ValueMessage { value: Some(ValueEnum::Null(0)) }), actual.value);
        }
    }

    #[tokio::test]
    async fn serve_subscriptions_should_error_when_no_client_active() {
        // arrange
        let subject = setup();

        // act
        let result = subject.serve_subscriptions(
            SubscribeIntent { channel_id: "client".into(), sources: vec!["test-event".into()] },
            |_| ValueEnum::Null(0),
        );

        // assert
        let result = result.unwrap_err();
        assert_eq!(Code::FailedPrecondition, result.code());
        assert_eq!("The specified client does not exist.", result.message());
    }

    fn setup() -> StreamingEss<()> {
        Default::default()
    }
}
