// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// api.rs contains code that can be considered "boilerplate" when
// interacting with the Intent Brokering runtime. It will most likely need to be
// repeated for all applications interacting with Intent Brokering.

use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    env,
};

use super::{inspection::Entry as InspectionEntry, value::Value};

use async_trait::async_trait;
use futures::{stream::BoxStream, StreamExt};
use intent_brokering_common::error::{Error, ResultExt as _};
use intent_brokering_proto::{
    common::{
        discover_fulfillment::Service as ServiceMessage, DiscoverFulfillment, DiscoverIntent,
        FulfillmentEnum, InspectFulfillment, InspectIntent, IntentEnum, IntentMessage,
        InvokeFulfillment, InvokeIntent, ReadFulfillment, ReadIntent, SubscribeFulfillment,
        SubscribeIntent, WriteFulfillment, WriteIntent,
    },
    runtime::{
        intent_brokering_service_client::IntentBrokeringServiceClient, FulfillRequest,
        FulfillResponse,
    },
    streaming::{channel_service_client::ChannelServiceClient, OpenRequest},
};
use tonic::{transport::Channel, Request, Response};
use tracing::debug;

const INTENT_BROKER_URL_KEY: &str = "INTENT_BROKER_URL";
const DEFAULT_INTENT_BROKER_URL: &str = env!("DEFAULT_INTENT_BROKER_URL");

struct Fulfillment(FulfillmentEnum);

trait FulfillResponseExt {
    fn fulfillment<F>(self) -> Result<F, Error>
    where
        F: TryFrom<Fulfillment>;
}

impl FulfillResponseExt for Response<FulfillResponse> {
    fn fulfillment<F>(self) -> Result<F, Error>
    where
        F: TryFrom<Fulfillment>,
    {
        self.into_inner()
            .fulfillment
            .and_then(|fulfillment| fulfillment.fulfillment)
            .ok_or_else(|| Error::new("Did not receive fulfillment"))
            .and_then(|f| {
                Fulfillment(f).try_into().map_err(|_| Error::new("Unpexpected fulfillment"))
            })
    }
}

macro_rules! impl_try_from_var {
    ($source:ty, $variant:path, $target:ty) => {
        impl TryFrom<$source> for $target {
            type Error = ();

            fn try_from(value: $source) -> Result<Self, Self::Error> {
                match value.0 {
                    $variant(f) => Ok(f),
                    _ => Err(()),
                }
            }
        }
    };
}

impl_try_from_var!(Fulfillment, FulfillmentEnum::Inspect, InspectFulfillment);
impl_try_from_var!(Fulfillment, FulfillmentEnum::Read, ReadFulfillment);
impl_try_from_var!(Fulfillment, FulfillmentEnum::Write, WriteFulfillment);
impl_try_from_var!(Fulfillment, FulfillmentEnum::Invoke, InvokeFulfillment);
impl_try_from_var!(Fulfillment, FulfillmentEnum::Subscribe, SubscribeFulfillment);
impl_try_from_var!(Fulfillment, FulfillmentEnum::Discover, DiscoverFulfillment);

#[derive(Clone)]
pub struct GrpcIntentBrokering {
    client: IntentBrokeringServiceClient<Channel>,
}

impl GrpcIntentBrokering {
    pub async fn connect() -> Result<Self, Error> {
        let intent_brokering_url = env::var(INTENT_BROKER_URL_KEY)
            .unwrap_or_else(|_| DEFAULT_INTENT_BROKER_URL.to_string());
        let client = IntentBrokeringServiceClient::connect(intent_brokering_url)
            .await
            .map_err_with("Connecting to IntentBrokering failed.")?;

        Ok(Self { client })
    }
}

#[async_trait]
impl IntentBrokeringCommunication for GrpcIntentBrokering {
    async fn fulfill(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        intent: IntentEnum,
    ) -> Result<Response<FulfillResponse>, Error> {
        self.client
            .fulfill(Request::new(FulfillRequest {
                intent: Some(IntentMessage { intent: Some(intent) }),
                namespace: namespace.into().into(),
            }))
            .await
            .map_err_with("Intent fulfillment failed.")
    }
}

/// IntentBrokering abstracts the Communication layer, but is based on the Protobuf
/// definitions of the IntentBrokering API.
#[async_trait]
pub trait IntentBrokeringCommunication: Send {
    async fn fulfill(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        intent: IntentEnum,
    ) -> Result<Response<FulfillResponse>, Error>;
}

/// IntentBrokering abstracts the Protobuf definitions that define IntentBrokering's API.
#[async_trait]
pub trait IntentBrokering: Send {
    async fn invoke<I: IntoIterator<Item = Value> + Send>(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        command: impl Into<Box<str>> + Send,
        args: I,
    ) -> Result<Value, Error>;

    async fn subscribe<I: IntoIterator<Item = Box<str>> + Send>(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        channel_id: impl Into<Box<str>> + Send,
        event_ids: I,
    ) -> Result<(), Error>;

    async fn discover(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
    ) -> Result<Vec<Service>, Error>;

    async fn inspect(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        query: impl Into<Box<str>> + Send,
    ) -> Result<Vec<InspectionEntry>, Error>;

    async fn write(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        key: impl Into<Box<str>> + Send,
        value: Value,
    ) -> Result<(), Error>;

    async fn read(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        key: impl Into<Box<str>> + Send,
    ) -> Result<Option<Value>, Error>;
}

#[async_trait]
impl<T: IntentBrokeringCommunication> IntentBrokering for T {
    async fn invoke<I: IntoIterator<Item = Value> + Send>(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        command: impl Into<Box<str>> + Send,
        args: I,
    ) -> Result<Value, Error> {
        let command = command.into();
        debug!("Invoking command '{:?}'.", command);

        let args = args.into_iter().map(|arg| arg.into()).collect();

        self.fulfill(namespace, IntentEnum::Invoke(InvokeIntent { args, command: command.into() }))
            .await?
            .fulfillment()
            .and_then(|invoke: InvokeFulfillment| {
                invoke
                    .r#return
                    .and_then(|v| v.try_into().ok())
                    .ok_or_else(|| Error::new("Return value could not be parsed."))
            })
    }

    async fn subscribe<I: IntoIterator<Item = Box<str>> + Send>(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        channel_id: impl Into<Box<str>> + Send,
        event_ids: I,
    ) -> Result<(), Error> {
        let channel_id = channel_id.into();
        debug!("Subscribing to events on channel '{:?}'.", channel_id);

        let sources = event_ids.into_iter().map(|e| e.into()).collect();

        self.fulfill(
            namespace,
            IntentEnum::Subscribe(SubscribeIntent { channel_id: channel_id.into(), sources }),
        )
        .await?
        .fulfillment()
        .map(|_: SubscribeFulfillment| ())
    }

    async fn discover(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
    ) -> Result<Vec<Service>, Error> {
        let namespace = namespace.into();
        debug!("Discovering services for namespace '{:?}'.", namespace);

        self.fulfill(namespace, IntentEnum::Discover(DiscoverIntent {})).await?.fulfillment().map(
            |discover: DiscoverFulfillment| {
                discover.services.into_iter().map(|s| s.into()).collect()
            },
        )
    }

    async fn inspect(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        query: impl Into<Box<str>> + Send,
    ) -> Result<Vec<InspectionEntry>, Error> {
        let namespace = namespace.into();
        let query = query.into();
        debug!("Inspecting namespace '{:?}' with query '{:?}'.", namespace, query);

        self.fulfill(namespace, IntentEnum::Inspect(InspectIntent { query: query.into() }))
            .await?
            .fulfillment()
            .and_then(|inspect: InspectFulfillment| {
                inspect
                    .entries
                    .into_iter()
                    .map(|e| {
                        let items_parse_result: Result<HashMap<Box<str>, Value>, ()> = e
                            .items
                            .into_iter()
                            .map(|(key, value)| value.try_into().map(|value| (key.into(), value)))
                            .collect();
                        match items_parse_result {
                            Ok(items) => Ok(InspectionEntry::new(e.path, items)),
                            Err(_) => Err(Error::new("Could not parse value.")),
                        }
                    })
                    .collect()
            })
    }

    async fn write(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        key: impl Into<Box<str>> + Send,
        value: Value,
    ) -> Result<(), Error> {
        let key = key.into();
        debug!("Writing key '{:?}' with value '{:?}'.", key, value);

        self.fulfill(
            namespace,
            IntentEnum::Write(WriteIntent { key: key.into(), value: Some(value.into()) }),
        )
        .await?
        .fulfillment()
        .map(|_: WriteFulfillment| ())
    }

    async fn read(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        key: impl Into<Box<str>> + Send,
    ) -> Result<Option<Value>, Error> {
        let key = key.into();
        let namespace = namespace.into();
        debug!("Reading key '{:?}' on namespace '{:?}'.", key, namespace);

        self.fulfill(namespace, IntentEnum::Read(ReadIntent { key: key.into() }))
            .await?
            .fulfillment()
            .and_then(|read: ReadFulfillment| match read.value {
                Some(value) => value
                    .value
                    .map(|v| {
                        Value::try_from(v).map_err(|_| Error::new("Could not parse read value."))
                    })
                    .map_or(Ok(None), |r| r.map(Some)),
                None => Ok(None),
            })
    }
}

#[async_trait::async_trait]
pub trait IntentBrokeringExt {
    async fn listen<'b>(
        self,
        namespace: impl Into<Box<str>> + Send,
        subscription_sources: impl IntoIterator<Item = Box<str>> + Send,
    ) -> Result<BoxStream<'b, Result<Event, Error>>, Error>;
}

#[async_trait::async_trait]
impl<T> IntentBrokeringExt for &mut T
where
    T: IntentBrokering + Send,
{
    async fn listen<'b>(
        self,
        namespace: impl Into<Box<str>> + Send,
        subscription_sources: impl IntoIterator<Item = Box<str>> + Send,
    ) -> Result<BoxStream<'b, Result<Event, Error>>, Error> {
        const CHANNEL_ID_HEADER_NAME: &str = "x-chariott-channel-id";
        const SDV_EVENT_STREAMING_SCHEMA_REFERENCE: &str = "intent_brokering.streaming.v1";
        const SDV_EVENT_STREAMING_SCHEMA_KIND: &str = "grpc+proto";

        let namespace = namespace.into();

        let streaming_endpoint = self
            .discover(namespace.clone())
            .await?
            .into_iter()
            .find(|service| {
                service.schema_reference.as_ref() == SDV_EVENT_STREAMING_SCHEMA_REFERENCE
                    && service.schema_kind.as_ref() == SDV_EVENT_STREAMING_SCHEMA_KIND
            })
            .ok_or_else(|| {
                Error::new("No compatible streaming endpoint found for '{namespace:?}'.")
            })?
            .url;

        debug!("Streaming endpoint for '{namespace:?}' is: {streaming_endpoint}");

        let mut provider_client = ChannelServiceClient::connect(streaming_endpoint.into_string())
            .await
            .map_err_with("Connecting to streaming endpoint failed.")?;

        let response = provider_client
            .open(Request::new(OpenRequest {}))
            .await
            .map_err_with("Opening stream failed.")?;

        debug!("Now listening for events in namespace '{namespace:?}'");

        let channel_id: Box<str> = response
            .metadata()
            .get(CHANNEL_ID_HEADER_NAME)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| Error::new("Channel ID header not found."))?
            .into();

        let result_stream = response.into_inner().map(|r| {
            r.map_err_with("Could not establish stream.").and_then(|event| {
                event
                    .value
                    .ok_or_else(|| Error::new("No value found in event payload."))
                    .and_then(|v| {
                        v.try_into().map_err(|_e: ()| Error::new("Could not parse protobuf value."))
                    })
                    .map(|data| Event { id: event.source.into_boxed_str(), data, seq: event.seq })
            })
        });

        self.subscribe(namespace, channel_id, subscription_sources).await?;

        Ok(result_stream.boxed())
    }
}

pub struct Event {
    pub id: Box<str>,
    pub data: Value,
    pub seq: u64,
}

pub struct Service {
    pub url: Box<str>,
    pub schema_kind: Box<str>,
    pub schema_reference: Box<str>,
}

impl From<ServiceMessage> for Service {
    fn from(value: ServiceMessage) -> Self {
        Service {
            url: value.url.into(),
            schema_kind: value.schema_kind.into(),
            schema_reference: value.schema_reference.into(),
        }
    }
}
