// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::net::SocketAddr;
use tokio::{signal::ctrl_c, spawn};
use tokio_util::sync::CancellationToken;
use tonic::{async_trait, transport::server::Router};
use tracing::error;

use crate::error::{Error, ResultExt as _};

pub fn ctrl_c_cancellation() -> CancellationToken {
    let cancellation_token = CancellationToken::new();
    let result = cancellation_token.child_token();

    spawn(async move {
        if let Err(e) = ctrl_c().await {
            error!("Could not listen to Ctrl+C: {}", e);
        }

        cancellation_token.cancel();
    });

    result
}

#[async_trait]
pub trait RouterExt {
    async fn serve_with_cancellation(
        self,
        socket_addr: SocketAddr,
        cancellation_token: CancellationToken,
    ) -> Result<(), Error>;

    async fn serve_with_ctrl_c_shutdown(self, socket_addr: SocketAddr) -> Result<(), Error>;
}

#[async_trait]
impl RouterExt for Router {
    async fn serve_with_cancellation(
        self,
        socket_addr: SocketAddr,
        cancellation_token: CancellationToken,
    ) -> Result<(), Error> {
        self.serve_with_shutdown(socket_addr, cancellation_token.cancelled())
            .await
            .map_err_with("Error when serving gRPC server.")
    }

    async fn serve_with_ctrl_c_shutdown(self, socket_addr: SocketAddr) -> Result<(), Error> {
        self.serve_with_cancellation(socket_addr, ctrl_c_cancellation()).await
    }
}
