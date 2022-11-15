// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use chariott_common::{
    config::env,
    error::{Error, ResultExt as _},
};
use chariott_proto::{
    common::{FulfillmentEnum, FulfillmentMessage, IntentEnum, InvokeFulfillment},
    provider::{
        provider_service_server::{ProviderService, ProviderServiceServer},
        FulfillRequest, FulfillResponse,
    },
    runtime::{intent_registration::Intent, intent_service_registration::ExecutionLocality},
};
use examples_common::chariott;
use examples_common::chariott::value::Value;
use rand::{rngs::SmallRng, SeedableRng};
use rand_distr::{DistIter, Distribution, Normal};
use tokio::time::sleep;
use tonic::transport::Server;
use tonic::{async_trait, Request, Response, Status};
use tracing::info;

// Mean latency in milliseconds.
const LATENCY_MEAN_ENV: &str = "LATENCY_MEAN";

// Standard deviation of latency distribution in milliseconds.
const LATENCY_STD_DEV_ENV: &str = "LATENCY_STD_DEV";

chariott::provider::main!(wain);

async fn wain() -> Result<(), Error> {
    let (url, socket_address) = chariott::provider::register(
        "lt.provider",
        "0.0.1",
        "lt.provider",
        [Intent::Invoke],
        "LT_PROVIDER_URL",
        "http://0.0.0.0:50051",
        ExecutionLocality::Local,
    )
    .await?;

    info!("LT provider listening: {url}");

    let provider = match (env(LATENCY_MEAN_ENV), env(LATENCY_STD_DEV_ENV)) {
        (Some(mean), Some(std_dev)) => ChariottProvider::normal(mean, std_dev),
        (_, _) => ChariottProvider::new(),
    };

    Server::builder()
        .add_service(ProviderServiceServer::new(provider))
        .serve(socket_address)
        .await
        .map_err_with("Error when serving gRPC server.")
}

type Rand = Arc<Mutex<DistIter<Normal<f32>, SmallRng, f32>>>;

struct ChariottProvider {
    latency_distribution: Option<Rand>,
}

impl ChariottProvider {
    pub fn new() -> Self {
        info!("No simulation of response latencies, responding immediately.");
        Self { latency_distribution: None }
    }

    pub fn normal(mean: f32, std_dev: f32) -> Self {
        info!("Using a normal distribution with mean {mean} and standard deviation {std_dev} for sampling latencies.");

        let distribution = Normal::new(mean as _, std_dev as _).unwrap();
        // Use a RNG seeded from entropy, as the non-deterministic timings on
        // the consumer application side make reproducing impossible in any
        // case.
        Self {
            latency_distribution: Some(Arc::new(Mutex::new(
                distribution.sample_iter(SmallRng::from_entropy()),
            ))),
        }
    }
}

#[async_trait]
impl ProviderService for ChariottProvider {
    async fn fulfill(
        &self,
        request: Request<FulfillRequest>,
    ) -> Result<Response<FulfillResponse>, Status> {
        let response = match request
            .into_inner()
            .intent
            .and_then(|i| i.intent)
            .ok_or_else(|| Status::invalid_argument("Intent must be specified"))?
        {
            IntentEnum::Invoke(_) => {
                if let Some(latency_distribution) = &self.latency_distribution {
                    let sample = latency_distribution.lock().unwrap().next().unwrap();
                    sleep(Duration::from_millis(sample as _)).await;
                }

                FulfillmentEnum::Invoke(InvokeFulfillment { r#return: Some(Value::NULL.into()) })
            }
            _ => Err(Status::not_found(""))?,
        };

        Ok(Response::new(FulfillResponse {
            fulfillment: Some(FulfillmentMessage { fulfillment: Some(response) }),
        }))
    }
}
