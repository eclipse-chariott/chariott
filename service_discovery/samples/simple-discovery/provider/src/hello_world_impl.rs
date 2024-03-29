// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Module containing gRPC service implementation based on [`samples_proto::hello_world::v1`].
//!
//! Provides a gRPC endpoint for external services to call a hello world method.

use samples_proto::hello_world::v1::hello_world_server::HelloWorld;
use samples_proto::hello_world::v1::{HelloRequest, HelloResponse};
use tonic::{Request, Response, Status};
use tracing::info;

/// Base structure for the Hello World gRPC service.
#[derive(Default)]
pub struct HelloWorldImpl {}

#[tonic::async_trait]
impl HelloWorld for HelloWorldImpl {
    /// This function returns a message which says Hello, followed by the string
    /// provided in the request.
    ///
    /// # Arguments
    ///
    /// * `request` - Contains the input string
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
