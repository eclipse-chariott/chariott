// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

mod chariott_provider;

use std::{sync::Arc, net::SocketAddr, time::Duration};

use url::Url;

use chariott_common::error::Error;
use chariott_common::shutdown::RouterExt as _;
use chariott_proto::{
    provider::provider_service_server::ProviderServiceServer,
    runtime::{
        intent_registration::Intent, intent_service_registration::ExecutionLocality,
        IntentServiceRegistration, AnnounceRequest, IntentRegistration, RegisterRequest,
        RegistrationState, chariott_service_client::ChariottServiceClient
    },
};
use examples_common::chariott;
use tokio::time::sleep;
use tonic::transport::{Channel, Server};
use tracing::warn;

use crate::chariott_provider::ChariottProvider;

async fn connect_chariott_client(
    client: &mut Option<ChariottServiceClient<Channel>>,
    chariott_url: String
) -> Result<(), Error> {
    *client = Some(ChariottServiceClient::connect(chariott_url).await
        .map_err(|e| {
            *client = None; // Set client back to None on error.
            Error::from_error("Could not connect to client", Box::new(e))
        })?
    );

    Ok(())
}

async fn register_announce_once(
    client: &mut Option<ChariottServiceClient<Channel>>,
    name: String,
    version: String,
    namespace: String,
    intents: Vec<Intent>,
    url: String,
    chariott_url: String,
    locality: ExecutionLocality
) -> Result<(), Error> {

    // If there is no client, need to attempt connection.
    if client.is_none() {
        connect_chariott_client(client, chariott_url).await?;
    }

    let service = Some(IntentServiceRegistration {
        name: name,
        url: url,
        version: version,
        locality: locality as i32,
    });

    let announce_req = AnnounceRequest {
        service: service.clone(),
    };

    // Allways announce to Chariott.
    let registration_state = client.as_mut()
        .expect("No client found")
        .announce(announce_req.clone())
        .await
        .map_err(|e| Error::from_error("Error announcing to Chariott.", Box::new(e)))?
        .into_inner()
        .registration_state;
    
    // Only attempt registration with Chariott if the announced state is ANNOUNCED.
    // This also handles re-registration if Chariott crashes and comes back online.
    if registration_state == RegistrationState::Announced as i32 {
        let register_req = RegisterRequest {
            service: service.clone(),
            intents: intents.iter().map(
                |i| IntentRegistration {
                    intent: *i as i32,
                    namespace: namespace.to_string(),
                }
            ).collect(),
        };

        tracing::info!("Registered with Chariott runtime: {:?}", register_req);

        _ = client.as_mut()
            .expect("No client found")
            .register(register_req.clone())
            .await
            .map_err(|e| Error::from_error("Error registering with Chariott.", Box::new(e)))?;
    }

    Ok(())
}


async fn register_announce_provider(
    name: String,
    version: String,
    namespace: String,
    intents: Vec<Intent>,
    url: String,
    chariott_url: String,
    locality: ExecutionLocality,
    ttl_seconds: u64
) -> Result<(), Error> {

    // Initiate registration and announce thread.
    _ = tokio::task::spawn(async move {
        let mut client = None;

        // Loop that handles provider registration and announce heartbeat pattern.
        loop {
            match register_announce_once(
                &mut client,
                name.clone(),
                version.clone(),
                namespace.clone(),
                intents.clone(),
                url.clone(),
                chariott_url.clone(),
                locality.clone()
            ).await {
                Ok(_) => {},
                Err(e) => {
                    warn!(
                        "Registration failed with '{:?}'. Retrying after {:?}.",
                        e, ttl_seconds
                    );
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
    let chariott_url = "http://0.0.0.0:4243".to_string();
    let base_provider_address = "0.0.0.0:50064";
    let provider_url_str = format!("http://{}", base_provider_address.clone());
    let socket_address: SocketAddr = base_provider_address.clone().parse().map_err(|e| Error::from_error("error getting SocketAddr", Box::new(e)))?;
    let provider_url: Url = Url::parse(&provider_url_str).map_err(|e| Error::from_error("error getting Url", Box::new(e)))?; 

    // Intitate provider registration and announce heartbeat.
    register_announce_provider(
        "sdv.simple.provider".to_string(),
        "0.0.1".to_string(),
        "sdv.simple.provider".to_string(),
        [Intent::Discover].to_vec(),
        provider_url_str.clone(),
        chariott_url,
        ExecutionLocality::Local,
        5
    )
    .await?;

    tracing::info!("Application listening on: {provider_url_str}");

    let provider = Arc::new(ChariottProvider::new(provider_url.clone()));

    Server::builder()
        .add_service(ProviderServiceServer::from_arc(Arc::clone(&provider)))
        .serve_with_ctrl_c_shutdown(socket_address)
        .await
}