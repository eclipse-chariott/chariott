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
            .map_err(|_| Status::failed_precondition("Specified client does not exist."))?;

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
                Change::Modify(intent, services) => {
                    if services.is_empty() {
                        Some(intent.namespace())
                    } else {
                        None
                    }
                }
            })
            .collect::<HashSet<_>>()
        {
            self.0.publish(namespace, ());
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

        let id: Box<str> = uuid::Uuid::new_v4().to_string().into();
        let (_, receiver_stream) = self.0.read_events(id.clone());
        let mut response = Response::new(receiver_stream);
        response.metadata_mut().insert(METADATA_KEY, id.to_string().try_into().unwrap());
        Ok(response)
    }
}
