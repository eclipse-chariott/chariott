// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use bollard::{container::StatsOptions, Docker};
use chariott_common::{
    config::env,
    error::{Error, ResultExt},
};
use examples_common::chariott::{
    self,
    api::{Chariott, GrpcChariott},
    value::Value,
};
use futures_util::stream::StreamExt;
use metrics_util::Summary;
use serde::Serialize;
use std::{
    fs::File,
    io::Write,
    ops::Mul,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tokio::time::{sleep, sleep_until};
use tracing::{debug, error, info, warn};

const CHARIOTT_CONTAINER_NAME: &str = "chariott";
const LT_PROVIDER_NAMESPACE: &str = "lt.provider";

// How many invoke fulfillments to schedule.
const TARGET_INVOKE_COUNT_ENV: &str = "TARGET_INVOKE_COUNT";

// At which rate to schedule the invoke fulfillments in [1/s].
const TARGET_RATE_ENV: &str = "TARGET_RATE";

const COLLECT_DOCKER_STATS_ENV: &str = "COLLECT_DOCKER_STATS";

// How many fulfillments to schedule at the same time at "max rate".
static CHUNK_SIZE: u64 = 100;

const LATENCY_METRIC: &str = "Latency";
const THROUGHPUT_METRIC: &str = "Invoke Throughput";
const MILLISECONDS_UNIT: &str = "Millisecond";
const RATE_UNIT: &str = "1/Second";
const RESULT_FILE: &str = "lt-output/app.out";
const RESULT_FILE_DOCKER: &str = "lt-output/docker.out";
const SAMPLE_RATE: u64 = 50;

chariott::provider::main!(wain);

async fn wain() -> Result<(), Error> {
    let invoke_count: u64 = env(TARGET_INVOKE_COUNT_ENV).unwrap();
    let target_rate: u32 = env(TARGET_RATE_ENV).unwrap();
    let collect_docker_stats = env(COLLECT_DOCKER_STATS_ENV).unwrap_or(false);
    let chunk_execution_duration =
        Duration::from_millis(1_000 * CHUNK_SIZE as u64 / target_rate as u64);

    let chariott = GrpcChariott::connect().await?;
    let latency_metric = Arc::new(Mutex::new(Summary::with_defaults()));
    let invoke_fulfillments = Arc::new(AtomicUsize::new(0));

    assert!(invoke_count > 0, "Must schedule at least one invocation.");

    info!(
        "Scheduling {} invoke fulfillments at a rate of {}/s. A new chunk schedules each {}ms.",
        invoke_count,
        target_rate,
        chunk_execution_duration.as_millis()
    );

    let cpu_usage_metric = Arc::new(Mutex::new(Summary::with_defaults()));
    let memory_usage_metric = Arc::new(Mutex::new(Summary::with_defaults()));
    if collect_docker_stats {
        tokio::task::spawn(evaluate_docker_stats(
            cpu_usage_metric.clone(),
            memory_usage_metric.clone(),
        ));
    }

    let now = tokio::time::Instant::now();
    let start_times =
        (0..=invoke_count / CHUNK_SIZE).map(|i| now + chunk_execution_duration.mul(i as u32));

    // Generate load

    let first_invoke_instant = Instant::now();

    for (chunk_index, start_time) in start_times.enumerate() {
        let chunk_offset = CHUNK_SIZE * chunk_index as u64;
        sleep_until(start_time).await;

        let now = Instant::now();

        for c in chunk_offset..(chunk_offset + CHUNK_SIZE) {
            if c == invoke_count {
                break;
            }

            let mut chariott = chariott.clone();
            let latency_metric = Arc::clone(&latency_metric);
            let invoke_fulfillments = Arc::clone(&invoke_fulfillments);

            _ = tokio::task::spawn(async move {
                // Measure the latency based on the sample rate.
                let now = if c % SAMPLE_RATE == 0 { Some(Instant::now()) } else { None };

                // Only the namespace matters when invoking. The load testing
                // provider will not take action based on payload or command.
                let sent_value = chariott.invoke(LT_PROVIDER_NAMESPACE, "foo", [Value::NULL]).await;

                if let Some(request_instant) = now {
                    let latency = request_instant.elapsed().as_millis() as _;
                    latency_metric.lock().unwrap().add(latency);
                }

                if let Err(e) = sent_value {
                    warn!("{}: {:?}", e, e);
                }

                invoke_fulfillments.fetch_add(1, Ordering::Relaxed);
            });
        }

        debug!("Scheduled chunk {} in {}ms.", chunk_offset, now.elapsed().as_millis());
    }

    // Wait for all invoke fulfillments.
    while invoke_fulfillments.load(Ordering::Relaxed) < invoke_count as _ {
        sleep(Duration::from_millis(50)).await;
    }

    info!("Calculating metrics.");

    let latency_metric = latency_metric.lock().unwrap();

    let metrics = vec![
        Metric {
            name: THROUGHPUT_METRIC,
            unit: RATE_UNIT,
            value: (invoke_count as f64 / (first_invoke_instant.elapsed()).as_secs_f64()).round(),
            range: None,
        },
        Metric {
            name: LATENCY_METRIC,
            unit: MILLISECONDS_UNIT,
            value: latency_metric.quantile(0.5).unwrap().round(),
            range: Some(format!("p95={}", latency_metric.quantile(0.95).unwrap().round())),
        },
    ];

    info!("Writing metrics to disk.");

    let json = serde_json::to_string(&metrics).unwrap();
    let mut file = File::create(RESULT_FILE).unwrap();
    file.write_all(json.as_bytes()).unwrap();

    if collect_docker_stats {
        info!("Writing docker stats to disk.");
        let cpu_usage_metric = cpu_usage_metric.lock().unwrap();
        let memory_usage_metric = memory_usage_metric.lock().unwrap();

        let metrics = vec![
            Metric {
                name: "CPU Usage",
                unit: "Percent",
                value: cpu_usage_metric.quantile(0.5).unwrap().round(),
                range: Some(format!("p95={}", cpu_usage_metric.quantile(0.95).unwrap().round())),
            },
            Metric {
                name: "Memory Usage",
                unit: "Bytes",
                value: memory_usage_metric.quantile(0.5).unwrap().round(),
                range: Some(format!("p95={}", memory_usage_metric.quantile(0.95).unwrap().round())),
            },
        ];

        let json = serde_json::to_string(&metrics).unwrap();
        let mut file = File::create(RESULT_FILE_DOCKER).unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }

    Ok(())
}

async fn evaluate_docker_stats(
    cpu_usage_metric: Arc<Mutex<Summary>>,
    memory_usage_metric: Arc<Mutex<Summary>>,
) {
    if let Err(e) = execute(cpu_usage_metric, memory_usage_metric).await {
        error!("{e:?}");
    }

    async fn execute(
        cpu_usage_metric: Arc<Mutex<Summary>>,
        memory_usage_metric: Arc<Mutex<Summary>>,
    ) -> Result<(), Error> {
        let docker = Docker::connect_with_local_defaults()
            .map_err_with("could not connect with defaults to docker")?;
        let mut stats = docker.stats(
            CHARIOTT_CONTAINER_NAME,
            Some(StatsOptions { stream: true, ..Default::default() }),
        );

        let version = docker.version().await.map_err_with("Could not retrieve docker version")?;
        info!("Default Docker API version: {version:?}");

        let mut number_cpus = None;
        while let Some(stats) = stats.next().await {
            let stats = stats.map_err_with("Could not retrieve stats")?;

            // Calculate container usage percentage according to docs
            // https://docs.docker.com/engine/api/v1.41/#tag/Container/operation/ContainerStats.
            if let Some(system_cpu_delta) =
                stats.cpu_stats.system_cpu_usage.and_then(|system_cpu_usage| {
                    stats
                        .precpu_stats
                        .system_cpu_usage
                        .map(|precpu_system_cpu_usage| system_cpu_usage - precpu_system_cpu_usage)
                })
            {
                let cpu_stats = stats.cpu_stats;
                let cpu_delta =
                    cpu_stats.cpu_usage.total_usage - stats.precpu_stats.cpu_usage.total_usage;

                if number_cpus.is_none() {
                    number_cpus = cpu_stats.cpu_usage.percpu_usage.map(|cpu| cpu.len());
                }

                cpu_usage_metric.lock().unwrap().add(
                    100.0 * number_cpus.unwrap() as f64 * cpu_delta as f64
                        / system_cpu_delta as f64,
                );
            }

            if let Some(memory_usage) = stats.memory_stats.usage {
                memory_usage_metric.lock().unwrap().add(memory_usage as f64 / 1_000.0);
            }
        }

        Ok(())
    }
}

#[derive(Serialize)]
struct Metric {
    name: &'static str,
    unit: &'static str,
    value: f64,
    range: Option<String>,
}
