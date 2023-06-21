// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Project Eclipse Chariott Service Discovery
//!
//! This is the Service Discovery system for Chariott. It includes a service registry,
//! which is a database of services that are currently registered. Other applications
//! can find the metadata for registered services.

// Tells cargo to warn if a doc comment is missing and should be provided.
#![warn(missing_docs)]

use parking_lot::RwLock;
use proto_servicediscovery::chariott_registry::v1::registry_server::RegistryServer;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::{debug, info};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod registry_impl;

/// Endpoint for the chariott service registry
const CHARIOTT_SERVICE_REGISTRY_ADDR: &str = "0.0.0.0:50000";

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

    // Start up registry service
    let addr: SocketAddr = CHARIOTT_SERVICE_REGISTRY_ADDR.parse()?;
    let registry_impl =
        registry_impl::RegistryImpl { registry_map: Arc::new(RwLock::new(HashMap::new())) };
    info!("Chariott Registry listening on {addr}");

    Server::builder().add_service(RegistryServer::new(registry_impl)).serve(addr).await?;

    debug!("The Chariott Registry has completed.");
    Ok(())
}
