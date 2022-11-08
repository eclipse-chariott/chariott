use std::collections::HashMap;

use async_trait::async_trait;
//use examples_common::chariott;
use examples_common::chariott::proto::provider::{
    provider_service_server::ProviderService, FulfillRequest, FulfillResponse,
};
//use examples_common::chariott::streaming::ProtoExt as _;
//use examples_common::chariott::value::Value;
use examples_common::chariott::proto::*;
//use examples_common::chariott::value::Value;
use tonic::{Request, Response, Status};
use url::Url;

pub struct Provider {
    url: Url,
}

impl Provider {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

#[async_trait]
impl ProviderService for Provider {
    async fn fulfill(
        &self,
        request: Request<FulfillRequest>,
    ) -> Result<Response<FulfillResponse>, Status> {
        let response = match request
            .into_inner()
            .intent
            .and_then(|i| i.intent)
            .ok_or_else(|| Status::invalid_argument("Intent must be specified"))?
        {
            common::intent::Intent::Discover(_) => {
                common::fulfillment::Fulfillment::Discover(common::DiscoverFulfillment {
                    services: vec![common::discover_fulfillment::Service {
                        url: self.url.to_string(),
                        schema_kind: "grpc+proto".to_owned(),
                        schema_reference: "kuksa.val.v1".to_owned(),
                        metadata: HashMap::new(),
                    }],
                })
            }
            /*
            common::intent::Intent::Inspect(inspect) => fulfill(inspect.query, &*VDT_SCHEMA),
            common::intent::Intent::Subscribe(subscribe) => {
                self.streaming_store.subscribe(subscribe)?
            }
            */
            common::intent::Intent::Read(_read) => unimplemented!(),
            _ => return Err(Status::unknown("Unknown or unsupported intent!")),
        };

        Ok(Response::new(provider::FulfillResponse {
            fulfillment: Some(common::Fulfillment { fulfillment: Some(response) }),
        }))
    }
}
