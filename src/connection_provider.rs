// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::sync::Arc;

use async_trait::async_trait;
use chariott_common::error::{Error, ResultExt as _};
use chariott_proto::provider::{
    provider_service_client::ProviderServiceClient, FulfillRequest, FulfillResponse,
};
use tokio::sync::Mutex;
use tonic::{transport::Channel, Request};
use url::Url;

/// Contains abstractions and implementations related to communication with
/// remote providers. The `ConnectionProvider` trait represents a remote
/// provider to which we can connect to, via its `connect` method we can ensure
/// that we get into a connected state (`ConnectedProvider`). We can create
/// custom `ConnectionProvider`s, such as the `ReusableProvider`, that allow us
/// to add functionality while wrapping an inner `ConnectionProvider`. Other
/// use-cases that may use this in the future is for adding logic for
/// observability, resiliency or retries or load-balancing between multiple
/// channels.

/// Abstracts the communication layer for a provider. Represents a provider to
/// which we can connect to.
#[async_trait]
pub trait ConnectionProvider {
    type ConnectedProvider: ConnectedProvider;

    /// Instantiates a new instance of the Provider implementation.
    fn new(url: Url) -> Self;

    /// Ensures that the `ConnectionProvider` is connected and returns a
    /// `Self::ConnectedProvider`.
    async fn connect(&mut self) -> Result<Self::ConnectedProvider, Error>;
}

/// Abstracts the communication layer for a provider. This is based on the
/// Protobuf definitions of the provider API. It represents a connected
/// provider.
#[async_trait]
pub trait ConnectedProvider {
    /// Fulfills a request for a given provider.
    async fn fulfill(&mut self, fulfill_request: FulfillRequest) -> Result<FulfillResponse, Error>;
}

/// Represents an unconnected, gRPC-based provider.
#[derive(Clone, Debug)]
pub struct GrpcProvider(pub(super) Url);

#[async_trait]
impl ConnectionProvider for GrpcProvider {
    type ConnectedProvider = ProviderServiceClient<Channel>;

    fn new(url: Url) -> Self {
        Self(url)
    }

    async fn connect(&mut self) -> Result<Self::ConnectedProvider, Error> {
        ProviderServiceClient::connect(self.0.to_string())
            .await
            .map_err_with("Error when connecting to provider.")
    }
}

#[async_trait]
impl ConnectedProvider for ProviderServiceClient<Channel> {
    async fn fulfill(&mut self, fulfill_request: FulfillRequest) -> Result<FulfillResponse, Error> {
        self.fulfill(Request::new(fulfill_request))
            .await
            .map_err_with("Error when invoking provider.")
            .map(|r| r.into_inner())
    }
}

/// Allows us to reuse a connected provider based on an unconnected provider,
/// given that they support an efficient `Clone` implementation.
#[derive(Clone, Debug)]
pub struct ReusableProvider<T: ConnectionProvider + Clone> {
    pub(super) inner: T,
    connected_inner: Arc<Mutex<Option<T::ConnectedProvider>>>,
}

/// Reuses a cached connected instance to be optimize the reconnection. When
/// calling connect, we do not always reconnect, but reuse the `Clone`
/// implementation instead.
#[async_trait]
impl<T> ConnectionProvider for ReusableProvider<T>
where
    T::ConnectedProvider: Clone + Send + Sync,
    T: ConnectionProvider + Clone + Send,
{
    type ConnectedProvider = T::ConnectedProvider;

    fn new(url: Url) -> Self {
        Self { inner: T::new(url), connected_inner: Arc::new(Mutex::new(None)) }
    }

    /// Establishes a connection to the provider if none exists, or clones the
    /// cached connection if already present.
    async fn connect(&mut self) -> Result<Self::ConnectedProvider, Error> {
        // Even though this operation is expected to be write-heavy, we choose
        // Mutex over RwLock as otherwise we might drop a few connections when
        // initializing. When load-testing, we could not make out a difference
        // in performance between the two, as the bottleneck is in a different
        // component.

        let mut connected_inner = self.connected_inner.lock().await;

        if let Some(connected_inner) = connected_inner.clone() {
            Ok(connected_inner)
        } else {
            let client = self.inner.connect().await?;
            *connected_inner = Some(client.clone());
            Ok(client)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use async_trait::async_trait;
    use chariott_common::error::Error;
    use chariott_proto::provider::{FulfillRequest, FulfillResponse};
    use url::Url;

    use super::{ConnectedProvider, ConnectionProvider, ReusableProvider};

    #[tokio::test]
    async fn reusable_provider_when_already_connected_reuses_provider() {
        #[derive(Clone)]
        struct MockProvider;

        #[async_trait]
        impl ConnectionProvider for MockProvider {
            type ConnectedProvider = MockConnectedProvider;

            fn new(_: Url) -> Self {
                Self
            }

            async fn connect(&mut self) -> Result<Self::ConnectedProvider, Error> {
                Ok(MockConnectedProvider { fulfill_count: Arc::new(AtomicUsize::new(0)) })
            }
        }

        #[derive(Clone)]
        struct MockConnectedProvider {
            fulfill_count: Arc<AtomicUsize>,
        }

        #[async_trait]
        impl ConnectedProvider for MockConnectedProvider {
            async fn fulfill(&mut self, _: FulfillRequest) -> Result<FulfillResponse, Error> {
                self.fulfill_count.fetch_add(1, Ordering::Relaxed);
                Err(Error::new("Not implemented"))
            }
        }

        // arrange
        let mut subject =
            ReusableProvider::<MockProvider>::new("https://contoso.com".parse().unwrap());

        // act
        let mut first = subject.connect().await.unwrap();
        let mut second = subject.connect().await.unwrap();
        let mut third = subject.connect().await.unwrap();

        // assert
        async fn fulfill_any(provider: &mut MockConnectedProvider) {
            provider.fulfill(FulfillRequest { intent: None }).await.unwrap_err();
        }

        fulfill_any(&mut first).await;
        fulfill_any(&mut second).await;
        fulfill_any(&mut third).await;

        assert_eq!(3, first.fulfill_count.load(Ordering::Relaxed));
    }
}
