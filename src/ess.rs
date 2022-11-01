use std::{collections::HashSet, sync::Arc, time::SystemTime};

use async_trait::async_trait;
use chariott_common::proto::{
    common::value::Value as ValueEnum,
    common::Value as ValueMessage,
    streaming::{channel_service_server::ChannelService, Event, OpenRequest},
};
use ess::EventSubSystem;
use tokio::spawn;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use uuid::Uuid;

use crate::registry::{Change, Observer};

type InnerEss = EventSubSystem<Box<str>, Box<str>, (), Result<Event, Status>>;

#[derive(Clone)]
pub struct Ess(Arc<InnerEss>);

impl Ess {
    pub fn new() -> Self {
        Self(Arc::new(EventSubSystem::new()))
    }

    pub fn serve_subscriptions(
        &self,
        client_id: impl Into<Box<str>>,
        requested_subscriptions: impl IntoIterator<Item = Box<str>>,
    ) -> Result<(), Status> {
        let subscriptions = self
            .0
            .register_subscriptions(client_id.into(), requested_subscriptions)
            .map_err(|_| Status::failed_precondition("The specified client does not exist."))?;

        for subscription in subscriptions {
            let source = subscription.event_id().to_string();

            spawn(subscription.serve(move |_, seq| {
                Ok(Event {
                    source: source.clone(),
                    value: Some(ValueMessage { value: Some(ValueEnum::Null(0)) }),
                    seq,
                    timestamp: Some(SystemTime::now().into()),
                })
            }));
        }

        Ok(())
    }
}

impl Default for Ess {
    fn default() -> Self {
        Self::new()
    }
}

impl Observer for Ess {
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
            self.0.publish(format!("namespaces[{}]", namespace).as_str(), ());
        }
    }
}

#[async_trait]
impl ChannelService for Ess {
    type OpenStream = ReceiverStream<Result<Event, Status>>;

    async fn open(
        &self,
        _: tonic::Request<OpenRequest>,
    ) -> Result<Response<Self::OpenStream>, Status> {
        const METADATA_KEY: &str = "x-chariott-channel-id";

        let id = Uuid::new_v4().to_string();
        let (_, receiver_stream) = self.0.read_events(id.clone().into());
        let mut response = Response::new(receiver_stream);
        response.metadata_mut().insert(METADATA_KEY, id.try_into().unwrap());
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, time::Duration};

    use chariott_common::proto::{
        common::value::Value as ValueEnum,
        common::Value as ValueMessage,
        streaming::{channel_service_server::ChannelService, OpenRequest},
    };
    use futures::Stream;
    use tokio_stream::StreamExt as _;
    use tonic::{Code, Request};

    use crate::registry::{
        tests::IntentConfigurationBuilder, Change, IntentConfiguration, Observer,
    };

    use super::Ess;

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
                format!("namespaces[{}]", namespace)
            }

            // arrange
            const CLIENT_ID: &str = "CLIENT";

            let subject = setup();
            let (_, stream) = subject.0.read_events(CLIENT_ID.into());

            // always subscribe to all possible namespace changes.
            for nonce in [INTENT_A, INTENT_B, INTENT_C] {
                let intent = IntentConfigurationBuilder::with_nonce(nonce).build();
                subject
                    .serve_subscriptions(CLIENT_ID, [namespace_event(intent.namespace()).into()])
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
            let mut result = exhaust(stream).map(|e| e.unwrap().source).collect::<Vec<_>>().await;

            // namespace change events can be delivered out of order. Sort
            // before comparing.
            result.sort();
            expected_events.sort();

            assert_eq!(result, expected_events);
        }
    }

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
        let client_id = response.metadata().get("x-chariott-channel-id").unwrap().to_str().unwrap();

        // act
        subject.serve_subscriptions(client_id, [EVENT_A.into(), EVENT_B.into()]).unwrap();

        // assert
        subject.0.publish(EVENT_A, ());
        subject.0.publish(EVENT_B, ());

        let result = exhaust(response.into_inner())
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|e| e.unwrap())
            .collect::<Vec<_>>();

        // assert sources
        assert_eq!(
            vec![EVENT_A, EVENT_B],
            result.iter().map(|e| e.source.clone()).collect::<Vec<_>>()
        );

        // assert sequence numbers
        assert_eq!(1, result[0].seq);
        assert_eq!(1, result[1].seq);

        // assert payload
        assert_eq!(Some(ValueMessage { value: Some(ValueEnum::Null(0)) }), result[0].value);
    }

    #[tokio::test]
    async fn serve_subscriptions_should_error_when_no_client_active() {
        // arrange
        let subject = setup();

        // act
        let result = subject.serve_subscriptions("client", ["test-event".into()]);

        // assert
        let result = result.unwrap_err();
        assert_eq!(Code::FailedPrecondition, result.code());
        assert_eq!("The specified client does not exist.", result.message());
    }

    fn setup() -> Ess {
        Ess::new()
    }

    // Takes values from a stream as long as the stream is still producing
    // values. If the stream did not produce a value for 100ms, it ends the
    // stream.
    fn exhaust<T>(stream: impl Stream<Item = T>) -> impl Stream<Item = T> {
        stream.timeout(Duration::from_millis(100)).take_while(|e| e.is_ok()).map(|e| e.unwrap())
    }
}
