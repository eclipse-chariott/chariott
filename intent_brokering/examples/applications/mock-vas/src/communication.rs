// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use futures::future::join_all;
use intent_brokering_common::shutdown::{ctrl_c_cancellation, RouterExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::spawn;
use url::Url;

use tonic::transport::Server;

use intent_brokering_common::error::{Error, ResultExt as _};
use intent_brokering_proto::{
    provider::provider_service_server::ProviderServiceServer,
    streaming::channel_service_server::ChannelServiceServer,
};

use crate::intent_provider::{IntentProvider, StreamingStore};
use crate::simulation::VehicleSimulation;

pub async fn serve(url: Url, address: SocketAddr) -> Result<(), Error> {
    let streaming_store = Arc::new(StreamingStore::new());
    let simulation = VehicleSimulation::new(Arc::clone(&streaming_store));
    let provider = IntentProvider::new(url, simulation.clone(), Arc::clone(&streaming_store));

    let cancellation_token = ctrl_c_cancellation();
    let server_token = cancellation_token.child_token();

    let simulation_handle = spawn(async move {
        let result = simulation
            .execute(cancellation_token.child_token())
            .await
            .map_err(|e| Error::from_error("Error when executing simulation.", e.into()));

        // If the simulation terminates, we shut down the entire program. In
        // case the simulation exited for a different reason, this will cause
        // the cancellation token to be canceled again, which will have no
        // effect.
        cancellation_token.cancel();

        result
    });

    let server_handle = spawn(
        Server::builder()
            .add_service(ProviderServiceServer::new(provider))
            .add_service(ChannelServiceServer::new(streaming_store.ess().clone()))
            .serve_with_cancellation(address, server_token),
    );

    for result in join_all([simulation_handle, server_handle]).await {
        result.map_err_with("Joining the handle failed.")??;
    }

    Ok(())
}
