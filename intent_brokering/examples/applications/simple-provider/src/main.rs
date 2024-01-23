// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod intent_provider;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use url::Url;

use intent_brokering_common::error::Error;
use intent_brokering_common::shutdown::RouterExt as _;
use intent_brokering_proto::{
    provider::provider_service_server::ProviderServiceServer,
    runtime::{
        intent_brokering_service_client::IntentBrokeringServiceClient, intent_registration::Intent,
        intent_service_registration::ExecutionLocality, AnnounceRequest, IntentRegistration,
        IntentServiceRegistration, RegisterRequest, RegistrationState,
    },
};
use examples_common::chariott;
use tokio::time::sleep;
use tonic::transport::{Channel, Server};
use tracing::warn;

use crate::intent_provider::IntentProvider;

#[derive(Clone)]
struct RegisterParams {
    name: String,
    namespace: String,
    version: String,
    intents: Vec<Intent>,
    url: String,
    chariott_url: String,
    locality: ExecutionLocality,
}

async fn connect_chariott_client(
    client: &mut Option<IntentBrokeringServiceClient<Channel>>,
    chariott_url: String,
) -> Result<(), Error> {
    *client = Some(IntentBrokeringServiceClient::connect(chariott_url).await.map_err(|e| {
        *client = None; // Set client back to None on error.
        Error::from_error("Could not connect to client", Box::new(e))
    })?);

    Ok(())
}

async fn register_and_announce_once(
    client: &mut Option<IntentBrokeringServiceClient<Channel>>,
    reg_params: RegisterParams,
) -> Result<(), Error> {
    // If there is no client, need to attempt connection.
    if client.is_none() {
        connect_chariott_client(client, reg_params.chariott_url).await?;
    }

    let service = Some(IntentServiceRegistration {
        name: reg_params.name,
        url: reg_params.url,
        version: reg_params.version,
        locality: reg_params.locality as i32,
    });

    let announce_req = AnnounceRequest { service: service.clone() };

    // Always announce to Chariott.
    let registration_state = client
        .as_mut()
        .expect("No client found")
        .announce(announce_req.clone())
        .await
        .map_err(|e| Error::from_error("Error announcing to Chariott.", Box::new(e)))?
        .into_inner()
        .registration_state;

    // Only attempt registration with Chariott if the announced state is 'ANNOUNCED'.
    // The 'ANNOUNCED' state means that this service is not currently registered in Chariott.
    // This also handles re-registration if Chariott crashes and comes back online.
    if registration_state == RegistrationState::Announced as i32 {
        let register_req = RegisterRequest {
            service: service.clone(),
            intents: reg_params
                .intents
                .iter()
                .map(|i| IntentRegistration {
                    intent: *i as i32,
                    namespace: reg_params.namespace.clone(),
                })
                .collect(),
        };

        tracing::info!("Registered with Chariott runtime: {:?}", register_req);

        _ = client
            .as_mut()
            .expect("No client found")
            .register(register_req.clone())
            .await
            .map_err(|e| Error::from_error("Error registering with Chariott.", Box::new(e)))?;
    }

    Ok(())
}

async fn register_and_announce_provider(
    reg_params: RegisterParams,
    ttl_seconds: u64,
) -> Result<(), Error> {
    // Initiate registration and announce thread.
    tokio::task::spawn(async move {
        let mut client = None;

        // Loop that handles provider registration and announce heartbeat pattern.
        loop {
            match register_and_announce_once(&mut client, reg_params.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Registration failed with '{:?}'. Retrying after {:?}.", e, ttl_seconds);
                }
            }

            // Interval between announce heartbeats or connection retries.
            sleep(Duration::from_secs(ttl_seconds)).await;
        }
    });

    Ok(())
}

// This macro sets up tracing and exit code handling.
chariott::provider::main!(wain);

async fn wain() -> Result<(), Error> {
    // Intitialize addresses for provider and chariott communication.
    let chariott_url = "http://0.0.0.0:4243".to_string(); // DevSkim: ignore DS137138
    let base_provider_address = "0.0.0.0:50064".to_string();
    let provider_url_str = format!("http://{}", base_provider_address.clone()); // DevSkim: ignore DS137138
    let socket_address: SocketAddr = base_provider_address
        .clone()
        .parse()
        .map_err(|e| Error::from_error("error getting SocketAddr", Box::new(e)))?;
    let provider_url: Url = Url::parse(&provider_url_str)
        .map_err(|e| Error::from_error("error getting Url", Box::new(e)))?;

    let register_params: RegisterParams = RegisterParams {
        name: "sdv.simple.provider".to_string(),
        namespace: "sdv.simple.provider".to_string(),
        version: "0.0.1".to_string(),
        intents: [Intent::Discover].to_vec(),
        url: provider_url_str.clone(),
        chariott_url,
        locality: ExecutionLocality::Local,
    };

    // Intitate provider registration and announce heartbeat.
    register_and_announce_provider(register_params, 5).await?;

    tracing::info!("Application listening on: {provider_url_str}");

    let provider = Arc::new(IntentProvider::new(provider_url.clone()));

    Server::builder()
        .add_service(ProviderServiceServer::from_arc(Arc::clone(&provider)))
        .serve_with_ctrl_c_shutdown(socket_address)
        .await
}
