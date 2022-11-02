// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{env, net::SocketAddr, time::Duration};

use tokio::time::sleep;
use tonic::transport::Channel;
use tracing::warn;
use url::Url;

use chariott_common::{
    config,
    error::{Error, ResultExt},
};

use crate::chariott::proto::runtime_api::{
    chariott_service_client::ChariottServiceClient, intent_service_registration::ExecutionLocality,
    *,
};
use crate::url::UrlExt as _;

const CHARIOTT_URL_KEY: &str = "CHARIOTT_URL";
const DEFAULT_CHARIOTT_URL: &str = env!("DEFAULT_CHARIOTT_URL");
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
    intents: Vec<intent_registration::Intent>,
    chariott_url: Url,
    registration_interval: Duration,
    locality: ExecutionLocality,
}

impl Builder {
    pub fn new(
        name: &str,
        version: &str,
        url: Url,
        namespace: &str,
        intents: impl IntoIterator<Item = intent_registration::Intent>,
        locality: ExecutionLocality,
    ) -> Self {
        let chariott_url = env::var(CHARIOTT_URL_KEY)
            .unwrap_or_else(|_| DEFAULT_CHARIOTT_URL.to_string())
            .parse()
            .unwrap();

        let announce_url: Url =
            chariott_common::config::env(ANNOUNCE_URL_KEY).unwrap_or_else(|| url.clone());

        Self {
            name: name.into(),
            version: version.into(),
            announce_url,
            provider_url: url,
            namespace: namespace.into(),
            intents: intents.into_iter().collect(),
            chariott_url,
            registration_interval: Duration::from_secs(5),
            locality,
        }
    }

    pub fn set_registration_interval(mut self, value: ConfigSource<Duration>) -> Self {
        match value {
            ConfigSource::Value(value) => self.registration_interval = value,
            ConfigSource::Environment(name) => {
                let name = name.unwrap_or("CHARIOTT_REGISTRATION_INTERVAL");
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

    pub fn set_chariott_url(mut self, value: ConfigSource<Url>) -> Self {
        match value {
            ConfigSource::Value(value) => self.chariott_url = value,
            ConfigSource::Environment(name) => {
                let name = name.unwrap_or("CHARIOTT_URL");
                if let Some(url) = config::env::<Url>(name) {
                    return self.set_chariott_url(ConfigSource::Value(url));
                }
            }
        }
        self
    }

    pub fn from_env(self) -> Self {
        self.set_chariott_url(ConfigSource::Environment(None))
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
        client: &mut Option<ChariottServiceClient<Channel>>,
        first_iteration: bool,
    ) -> Result<(), Error> {
        if client.is_none() {
            *client = Some(
                ChariottServiceClient::connect(self.chariott_url.to_string()).await.map_err_with(
                    format!("Could not connect to Chariott ({})", self.chariott_url),
                )?,
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
                .map_err_with("Error when announcing to Chariott.")?
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

                tracing::info!("Registered with Chariott runtime: {:?}", register_request);
                _ = client
                    .register(register_request.clone())
                    .await
                    .map_err_with("Error when registering with Chariott.")?;
            }
        }

        Ok(())
    }
}
