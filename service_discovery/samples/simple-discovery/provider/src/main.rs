// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! A simple provider for a sample of Chariott Service Discovery.
//!
//! This provider has one service, the hello_world service, which has one
//! method that returns a message containing "Hello, " followed by the string
//! provided in the request. The provider registers itself in the registry.

// Tells cargo to warn if a doc comment is missing and should be provided.
#![warn(missing_docs)]

use chariott_common::error::Error;
use hello_world_impl::HelloWorldImpl;
use samples_proto::hello_world::v1::hello_world_server::HelloWorldServer;
use std::net::SocketAddr;
use url::Url;

use service_discovery_proto::service_registry::v1::service_registry_client::ServiceRegistryClient;
use service_discovery_proto::service_registry::v1::{RegisterRequest, ServiceMetadata};
use tonic::transport::Server;
use tonic::Request;
use tracing::info;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod hello_world_impl;

/// URL for the service registry
const SERVICE_REGISTRY_URL: &str = "http://0.0.0.0:50000";
/// Endpoint for the hello world service, which is also a provider
const HELLO_WORLD_ENDPOINT: &str = "0.0.0.0:50064";
/// communication kind for this service
const COMMUNICATION_KIND: &str = "grpc+proto";
/// communication reference for this service
const COMMUNICATION_REFERENCE: &str = "hello_world_service.v1.proto";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing
    let collector = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .finish();

    collector.init();

    // Intitialize addresses for provider communication.
    let provider_url_str = format!("http://{HELLO_WORLD_ENDPOINT}");
    let socket_address: SocketAddr = HELLO_WORLD_ENDPOINT
        .parse()
        .map_err(|e| Error::from_error("error getting SocketAddr", Box::new(e)))?;
    let _provider_url: Url = Url::parse(&provider_url_str)
        .map_err(|e| Error::from_error("error getting Url", Box::new(e)))?;

    let service_metadata: ServiceMetadata = ServiceMetadata {
        namespace: "sdv.samples".to_string(),
        name: "hello-world".to_string(),
        version: "1.0.0.0".to_string(),
        uri: provider_url_str.clone(),
        communication_kind: COMMUNICATION_KIND.to_owned(),
        communication_reference: COMMUNICATION_REFERENCE.to_owned(),
    };

    let mut service_registry_client = ServiceRegistryClient::connect(SERVICE_REGISTRY_URL).await?;

    let register_request = Request::new(RegisterRequest { service: Some(service_metadata) });
    service_registry_client.register(register_request).await?.into_inner();
    info!("Hello World Service registered as a provider");

    let hello_world_impl = HelloWorldImpl::default();
    // Grpc server for handling calls from clients
    Server::builder()
        .add_service(HelloWorldServer::new(hello_world_impl))
        .serve(socket_address)
        .await?;
    Ok(())
}
