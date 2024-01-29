// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::sync::Arc;
use std::{env::args, net::SocketAddr};

use futures::future::join_all;
use intent_brokering_common::{
    error::{Error, ResultExt as _},
    shutdown::{ctrl_c_cancellation, RouterExt as _},
};
use intent_brokering_proto::{
    provider::provider_service_server::ProviderServiceServer,
    streaming::channel_service_server::ChannelServiceServer,
};
use tokio::spawn;
use tonic::transport::Server;
use url::Url;

use crate::{
    camera::CameraLogic,
    intent_provider::{IntentProvider, StreamingStore},
};

pub async fn serve(url: Url, address: SocketAddr) -> Result<(), Error> {
    let streaming_store = Arc::new(StreamingStore::new());
    let mut camera_logic = CameraLogic::new(Arc::clone(&streaming_store))?;

    let cancellation_token = ctrl_c_cancellation();
    let server_token = cancellation_token.child_token();

    let camera_handle = spawn(async move {
        let result = if args().any(|arg| arg == "-m") {
            camera_logic.execute(cancellation_token.child_token()).await
        } else {
            camera_logic.camera_loop(cancellation_token.child_token()).await
        };

        cancellation_token.cancel();

        result
    });

    let server_handle = spawn(
        Server::builder()
            .add_service(ProviderServiceServer::new(IntentProvider::new(
                url,
                Arc::clone(&streaming_store),
            )))
            .add_service(ChannelServiceServer::new(streaming_store.ess().clone()))
            .serve_with_cancellation(address, server_token),
    );

    for result in join_all([camera_handle, server_handle]).await {
        result.map_err_with("Joining the handle failed.")??;
    }

    Ok(())
}
