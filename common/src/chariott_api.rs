use std::env;

use async_trait::async_trait;
use chariott_proto::{
    common::{IntentEnum, IntentMessage},
    runtime::{chariott_service_client::ChariottServiceClient, FulfillRequest, FulfillResponse},
};
use tonic::{transport::Channel, Request, Response};

use crate::error::{Error, ResultExt as _};

/// Chariott abstracts the Communication layer, but is based on the Protobuf
/// definitions of the Chariott API.
#[async_trait]
pub trait ChariottCommunication: Send {
    async fn fulfill(
        &mut self,
        namespace: impl Into<Box<str>> + Send,
        intent: IntentEnum,
    ) -> Result<Response<FulfillResponse>, Error>;
}

#[derive(Clone)]
pub struct GrpcChariott {
    client: ChariottServiceClient<Channel>,
}

impl GrpcChariott {
    pub async fn connect() -> Result<Self, Error> {
        const CHARIOTT_URL_KEY: &str = "CHARIOTT_URL";
        const DEFAULT_CHARIOTT_URL: &str = env!("DEFAULT_CHARIOTT_URL");

        let chariott_url =
            env::var(CHARIOTT_URL_KEY).unwrap_or_else(|_| DEFAULT_CHARIOTT_URL.to_string());

        let client = ChariottServiceClient::connect(chariott_url)
            .await
            .map_err_with("Connecting to Chariott failed.")?;

        Ok(Self { client })
    }
}

#[async_trait]
impl ChariottCommunication for GrpcChariott {
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
