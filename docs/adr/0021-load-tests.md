# Load tests

Status: accepted

## Context and Problem Statement

We want to be able to track the performance of our solution and compare it to
the performance from previous versions of the codebase. If performance
regresses, we want to detect this as early as possible to be able to better
identify what introduced the regression.

## Decisions

We will reuse the load tests from a previous version of the code base, and adapt
them to use the invoke intent as main path for generating the load on Chariott.
A consumer application (can be extended to multiple consumer applications) will
generate load by requesting invoke fulfillments, the providers fulfilling the
invoke intent will allow for a configurable timeout until they response,
allowing us to test different distributions for response times.

The ESS will be benchmarked in isolation.

### Metrics to track

The following metrics will be tracked in a first version:

- Latency from requested fulfillment to received fulfillment by measuring the
  elapsed time from request to response.
- Throughput: fulfillments per unit of time, as measured by the consumer
  application.
- CPU usage
  - We collect this from Docker stats.
- Memory usage
  - We collect this from Docker stats.

This can be extended in the future:

- Provider registration activity
- Number of open channels
- Panics and recoverable errors
- Micro-benchmarks on components that are run using `cargo bench`

We discard the following metrics and profilers for use during load testing, as
they slow down the application considerably or are non-straightforward to
analyze/parse and are hence better suited for manual analysis:

- Valgrind/Massif, [slows down the application
  considerably](https://rust-analyzer.github.io/blog/2020/12/04/measuring-memory-usage-in-rust.html)
- [Heaptrack](https://github.com/KDE/heaptrack)
- [Perf](https://perf.wiki.kernel.org/index.php/Main_Page)
- [Valgrind/Memcheck](https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/6/html/performance_tuning_guide/s-memory-valgrind#idm140718732267040):
  Slows down app 10-30x.
- [Valgrind/Cachegrind](https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/6/html/performance_tuning_guide/ch05s03s02)

### Executing the load tests

We will execute the load tests as part of the CI. While a Raspberry Pi was
identified as our go-to hardware for load tests, for the purpose of detecting
performance regressions we care more about performance _changes_ instead of
absolute numbers. However, if the performance available on the Github agents
fluctuates, this might impact our benchmarks and/or generate false positives.

### Publishing metrics

#### Custom metrics

The metrics we consider for a first automation of the load tests can be tracked
without instrumenting Chariott. We will track and publish the metrics directly
from the load test provider/application, as they have access to all essential
performance metrics. We will use a metrics library, such as [OpenTelemetry
(Alpha)](https://opentelemetry.io/docs/instrumentation/rust/) or
[metrics-rs](https://github.com/metrics-rs/metrics) to be able to export the
metrics in different formats (e.g. Prometheus) without changing how we raise the
metrics. Measuring and preparing the metrics in the application/provider can be
an expensive operation, and hence in our case the measurement influences the
performance outcome. To alleviate this, we will sample expensive metrics (e.g.
latency calculation) and rely on trial-and-error to find a reasonable balance
between accuracy and impact on performance.

All mentioned metrics libraries are not to be considered production-ready and
used for load tests only.

#### Publishing memory/CPU consumption

We use the [docker](https://docs.rs/crate/docker/0.0.41) crate to collect Docker
stats from the load test application. While this is not the most insightful, it
also does not impact the performance much.

Using a more advanced profiler, such as Valgrind, slows down the application
considerably, but gives us more accuracy. We will consider in the future to run
a separate load test using a more advanced profiler.

### Analyzing published metrics

We will use [Github Action for Continuous
Benchmarking](https://github.com/benchmark-action/github-action-benchmark) to
track and analyze the history of load test runs. By transforming the metrics
raised in the application/provider into a format compatible with the Github
Action for Continuous Benchmarking, we satisfy the requirements for our load
tests. We:

- can support alerts as commits/comment on detected performance regressions
- can publish the performance benchmarks as GH Pages if we open-source the
  repository
- can keep the history as HTML file in the `docs` of our repository.
- must analyze the metrics and prepare it for consumption by the Action. This
  implies that we must manually transform, aggregate and calculate errors etc
  for the metrics that we want to track in the benchmark.

## Alternatives considered

### Using Azure Monitor

We considered using Azure Monitor to evaluate and store the metrics of the load
test runs _after the CI has completed_. It would:

- bind us to Azure Monitor, which is not ideal if we open-source the repository.
- be unclear in which subscription Application Insights/Log Analytics would run.
- give us advanced tools to evaluate metrics and be notified of regressions
  (Azure metrics explorer, alerts, workbooks, etc.)

To export the necessary metrics to Log Analytics, we could:

- use [appinsights-rs](https://github.com/dmolokanov/appinsights-rs) directly
  from the load test provider/application. This would be faster to implement,
  but creates a tight coupling to Application Insights.
- run the load test in AKS/Kubernetes/Container Instances, which would allow us
  to use [Container
  Insights](https://docs.microsoft.com/en-us/azure/azure-monitor/containers/container-insights-overview).
  While this gives us many things out of the box for analysis, it is a more
  complex setup for the load tests and is tightly coupled to Azure.
- run the [OMS
  Agent](https://docs.microsoft.com/en-us/azure/azure-monitor/containers/containers#install-and-configure-linux-container-hosts)
  agent. It is unclear whether the agent can also scrape Prometheus metrics (not
  documented, but is possible in Container Insights, which also uses this
  agent).
- not use the IoT Edge metrics collector for scraping Prometheus endpoints, as
  it needs an IoT Hub.
