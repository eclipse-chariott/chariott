// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use proto_registry::chariott_registry::v1::registry_server::RegistryServer;
use parking_lot::RwLock;
use tonic::transport::Server;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;

use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod registry_impl;

const CHARIOTT_SERVICE_REGISTRY_ADDR: &str = "[::1]:50000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // start up registry server
    

    // Set up tracing
    let collector = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::Level::INFO.into())
                .from_env_lossy(),
        )
        .finish();

    collector.init();

    let addr: SocketAddr = CHARIOTT_SERVICE_REGISTRY_ADDR.parse()?;
    let registry_impl = registry_impl::RegistryImpl {
        registry_map: Arc::new(RwLock::new(HashMap::new())),
    };

    let server_future = Server::builder().add_service(RegistryServer::new(registry_impl)).serve(addr);

    server_future.await?;
    println!("Helloworld!");
    Ok(())
}
