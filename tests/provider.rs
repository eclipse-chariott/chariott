// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use async_trait::async_trait;
use examples_common::chariott::{
    proto::{
        common::{self as common_proto, InvokeFulfillment},
        provider as provider_proto,
    },
    value::Value,
};
use tokio::{net::TcpSocket, spawn};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Request, Response, Status};
use url::Url;

#[derive(Default)]
pub struct Provider {
    on_invoke: Option<fn(common_proto::InvokeIntent) -> Option<Value>>,
    // Expand this type with other intents that are used for integration tests.
}

impl Provider {
    pub fn new() -> Self {
        Self { on_invoke: None }
    }

    pub fn with_on_invoke(
        self,
        on_invoke: fn(common_proto::InvokeIntent) -> Option<Value>,
    ) -> Self {
        Self { on_invoke: Some(on_invoke) }
    }

    pub async fn serve(self, port: u16) -> Url {
        let socket = TcpSocket::new_v4().unwrap();
        socket.bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)).unwrap();
        let listener = TcpListenerStream::new(socket.listen(2).unwrap());

        _ = spawn(
            Server::builder()
                .add_service(provider_proto::provider_service_server::ProviderServiceServer::new(
                    self,
                ))
                .serve_with_incoming(listener),
        );

        format!("http://localhost:{port}").parse().unwrap()
    }
}

#[async_trait]
impl provider_proto::provider_service_server::ProviderService for Provider {
    async fn fulfill(
        &self,
        request: Request<provider_proto::FulfillRequest>,
    ) -> Result<Response<provider_proto::FulfillResponse>, Status> {
        let response = match request
            .into_inner()
            .intent
            .and_then(|i| i.intent)
            .ok_or_else(|| Status::invalid_argument("Intent must be specified"))?
        {
            common_proto::intent::Intent::Invoke(intent) => {
                if let Some(on_invoke) = self.on_invoke {
                    let result = on_invoke(intent);
                    common_proto::fulfillment::Fulfillment::Invoke(InvokeFulfillment {
                        r#return: result.map(|v| v.into()),
                    })
                } else {
                    unimplemented!()
                }
            }
            _ => Err(Status::not_found(""))?,
        };

        Ok(Response::new(provider_proto::FulfillResponse {
            fulfillment: Some(common_proto::Fulfillment { fulfillment: Some(response) }),
        }))
    }
}
