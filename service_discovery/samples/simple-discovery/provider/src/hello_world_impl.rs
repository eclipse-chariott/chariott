// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Module containing gRPC service implementation based on [`proto_servicediscovery::hello_world::v1`].
//!
//! Provides a gRPC endpoint for external services to call a hello world method.

use proto_servicediscovery::hello_world::v1::hello_world_server::HelloWorld;
use proto_servicediscovery::hello_world::v1::{HelloRequest, HelloResponse};
use tonic::{Request, Response, Status};
use tracing::info;

/// Base structure for the Hello World gRPC service.
#[derive(Default)]
pub struct HelloWorldImpl {}

#[tonic::async_trait]
impl HelloWorld for HelloWorldImpl {
    /// Says Hello, followed by the input string
    /// This function returns a message which says Hello, followed by the string
    /// provided in a [`HelloRequest`]. Returns a [`HelloResponse`]
    ///
    /// # Arguments
    ///
    /// * `request` - A [`HelloRequest`] wrapped by a [`tonic::Request`].
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let request_inner = request.into_inner();
        let name = request_inner.name;
        let message = format!("Hello, {name}");
        let hello_response = HelloResponse { message: message.clone() };
        info!("Sent message: {message}");

        Ok(Response::new(hello_response))
    }
}
