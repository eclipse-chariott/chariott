// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{env, net::SocketAddr, time::Duration};

use intent_brokering_common::{
    config,
    error::{Error, ResultExt},
};
use intent_brokering_proto::runtime::{
    intent_brokering_service_client::IntentBrokeringServiceClient, intent_registration::Intent,
    intent_service_registration::ExecutionLocality, AnnounceRequest, IntentRegistration,
    IntentServiceRegistration, RegisterRequest, RegistrationState,
};
use tokio::time::sleep;
use tonic::transport::Channel;
use tracing::warn;
use url::Url;

use crate::url::UrlExt as _;

const INTENT_BROKER_URL_KEY: &str = "INTENT_BROKER_URL";
const DEFAULT_INTENT_BROKER_URL: &str = env!("DEFAULT_INTENT_BROKER_URL");
const ANNOUNCE_URL_KEY: &str = "ANNOUNCE_URL";

pub enum ConfigSource<'a, T> {
    Value(T),
    Environment(Option<&'a str>),
}

pub struct Builder {
    name: Box<str>,
    version: Box<str>,
    announce_url: Url,
    provider_url: Url,
    namespace: Box<str>,
    intents: Vec<Intent>,
    intent_broker_url: Url,
    registration_interval: Duration,
    locality: ExecutionLocality,
}

impl Builder {
    pub fn new(
        name: &str,
        version: &str,
        url: Url,
        namespace: &str,
        intents: impl IntoIterator<Item = Intent>,
        locality: ExecutionLocality,
    ) -> Self {
        let intent_broker_url = env::var(INTENT_BROKER_URL_KEY)
            .unwrap_or_else(|_| DEFAULT_INTENT_BROKER_URL.to_string())
            .parse()
            .unwrap();

        let announce_url: Url =
            intent_brokering_common::config::env(ANNOUNCE_URL_KEY).unwrap_or_else(|| url.clone());

        Self {
            name: name.into(),
            version: version.into(),
            announce_url,
            provider_url: url,
            namespace: namespace.into(),
            intents: intents.into_iter().collect(),
            intent_broker_url,
            registration_interval: Duration::from_secs(5),
            locality,
        }
    }

    pub fn set_registration_interval(mut self, value: ConfigSource<Duration>) -> Self {
        match value {
            ConfigSource::Value(value) => self.registration_interval = value,
            ConfigSource::Environment(name) => {
                let name = name.unwrap_or("INTENT_BROKER_REGISTRATION_INTERVAL");
                let registration_interval = self.registration_interval;
                return self.set_registration_interval(ConfigSource::Value(
                    config::env::<u64>(name)
                        .map(Duration::from_secs)
                        .unwrap_or(registration_interval),
                ));
            }
        }
        self
    }

    pub fn set_intent_broker_url(mut self, value: ConfigSource<Url>) -> Self {
        match value {
            ConfigSource::Value(value) => self.intent_broker_url = value,
            ConfigSource::Environment(name) => {
                let name = name.unwrap_or("INTENT_BROKER_URL");
                if let Some(url) = config::env::<Url>(name) {
                    return self.set_intent_broker_url(ConfigSource::Value(url));
                }
            }
        }
        self
    }

    pub fn from_env(self) -> Self {
        self.set_intent_broker_url(ConfigSource::Environment(None))
            .set_registration_interval(ConfigSource::Environment(None))
    }

    pub fn announce_url(&self) -> &Url {
        &self.announce_url
    }

    pub fn provider_url(&self) -> &Url {
        &self.provider_url
    }

    pub fn parse_provider_socket_address(&self) -> Result<SocketAddr, Error> {
        self.provider_url()
            .parse_socket_address()
            .map_err_with("Error parsing provider socket address.")
    }

    pub async fn register(self) {
        let mut client = None;
        let mut first_iteration = true;

        loop {
            match self.register_once(&mut client, first_iteration).await {
                Ok(_) => {
                    first_iteration = false;
                }
                Err(e) => {
                    warn!(
                        "Registration failed with '{:?}'. Retrying after {:?}.",
                        e, self.registration_interval
                    );
                    client = None;
                }
            }

            sleep(self.registration_interval).await;
        }
    }

    pub async fn register_once(
        &self,
        client: &mut Option<IntentBrokeringServiceClient<Channel>>,
        first_iteration: bool,
    ) -> Result<(), Error> {
        if client.is_none() {
            *client = Some(
                IntentBrokeringServiceClient::connect(self.intent_broker_url.to_string())
                    .await
                    .map_err_with(format!(
                        "Could not connect to IntentBrokering ({})",
                        self.intent_broker_url
                    ))?,
            );
        }

        if let Some(client) = client {
            let announce_request = AnnounceRequest {
                service: Some(IntentServiceRegistration {
                    name: self.name.to_string(),
                    url: self.announce_url.to_string(),
                    version: self.version.to_string(),
                    locality: self.locality as i32,
                }),
            };

            let registration_state = client
                .announce(announce_request.clone())
                .await
                .map_err_with("Error when announcing to IntentBrokering.")?
                .into_inner()
                .registration_state;

            if first_iteration || registration_state == RegistrationState::Announced as i32 {
                let register_request = RegisterRequest {
                    service: announce_request.service.clone(),
                    intents: self
                        .intents
                        .iter()
                        .map(|i| IntentRegistration {
                            intent: *i as i32,
                            namespace: self.namespace.to_string(),
                        })
                        .collect(),
                };

                tracing::info!("Registered with IntentBrokering runtime: {:?}", register_request);
                _ = client
                    .register(register_request.clone())
                    .await
                    .map_err_with("Error when registering with IntentBrokering.")?;
            }
        }

        Ok(())
    }
}
