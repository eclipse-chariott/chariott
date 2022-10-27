// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

#[macro_export]
macro_rules! chariott_provider_main {
    ($main:ident) => {
        #[cfg(not(tarpaulin_include))]
        #[::tokio::main]
        async fn main() -> ::std::process::ExitCode {
            use ::examples_common::chariott::provider::internal::trace_result;
            use ::std::process::ExitCode;
            use ::tracing_subscriber::{util::SubscriberInitExt, EnvFilter};

            let main = async {
                let collector = tracing_subscriber::fmt()
                    .with_env_filter(
                        EnvFilter::builder()
                            .with_default_directive(tracing::Level::INFO.into())
                            .from_env_lossy(),
                    )
                    .finish();

                collector.init();

                $main().await
            };

            let result = main.await;
            trace_result("Error when executing main", &result);
            result.map(|_| ExitCode::from(0)).unwrap_or(ExitCode::from(1))
        }
    };
}

pub use chariott_provider_main as main;

use std::net::SocketAddr;

use url::Url;

use chariott_common::config::env;
use chariott_common::error::{Error, ResultExt};

use super::proto::runtime_api::intent_service_registration::ExecutionLocality;

pub async fn register(
    name: impl Into<&str>,
    version: impl Into<&str>,
    namespace: impl Into<&str>,
    intents: impl IntoIterator<Item = super::proto::runtime_api::intent_registration::Intent>,
    url_env_name: impl Into<&str>,
    url: impl Into<&str>,
    locality: ExecutionLocality,
) -> Result<(Url, SocketAddr), Error> {
    let url: Url = env(url_env_name.into())
        .unwrap_or_else(|| url.into().to_owned())
        .parse()
        .map_err_with("Failed to parse URL.")?;

    let registration = super::registration::Builder::new(
        name.into(),
        version.into(),
        url,
        namespace.into(),
        intents,
        locality,
    )
    .from_env();

    let socket_address = registration.parse_provider_socket_address()?;
    let announce_url = registration.announce_url().to_owned();

    // Potential race condition if we register before the server is up.
    // Since this is only an example, we do not ensure that the race does not
    // happen.
    _ = tokio::task::spawn(registration.register());

    Ok((announce_url, socket_address))
}

pub mod internal {
    use super::Error;
    use tracing::error;

    /// Ensures that a result is tracked in case of an error.
    pub fn trace_result<T>(action: &str, result: &Result<T, Error>) {
        if let Err(e) = result {
            trace_error(action, &e);
        }
        /// Recursively traces the source error
        fn trace_error(action: &str, error: &(dyn std::error::Error)) {
            error!("{action}\nDescription: '{error}'\nDebug: '{error:?}'");
            if let Some(source) = error.source() {
                trace_error("--- Inner Error", source);
            }
        }
    }
}
