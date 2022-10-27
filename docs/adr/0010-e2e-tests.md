# E2E Tests

- Status: superseded by [ADR-0020](./0020-integration-and-e2e-testing.md)

## Context and Problem Statement

The repository contains the following components that can be E2E tested:

- Middleware binary
- Application library (core, without generated code)
- Provider library (core, without generated code)
- Docker images
- Samples
- In the future: generated client libraries

This ADR documents how we will approach E2E-testing these components.

## Decision Drivers

Ensure system does behave and run correctly through an end to end scenario.

## Decisions

We will run E2E tests in a pipeline on each PR that is targeting `main`. The
pipeline will build a docker image of the middleware and start a separate
container (on a different port) for each E2E test as demonstrated in a Spike. We
will ensure that E2E tests can be run as CI on Github-hosted agents as well as
locally from the Devcontainer.

The E2E tests will use the Provider and Application libraries directly
(=in-process) and emulate Provider behavior directly via the Provider library.
The Application library will be used for assertions.

Using this strategy, we can test the following components:

- Middleware binary + Docker image
- Application library
- Provider library

This implies that our dog mode samples and the "generated" (at the point of
writing it is still manual) event/function signatures are not tested.

## Alternatives considered

### Testing with "complete" Applications/Providers

We considered building a dockerized Provider/Application that can be controlled
via a (e.g. REST) API. While this gives us stronger guarantees (using a
deployed, or at least Dockerized, Application/Provider), it is also more
difficult to implement and requires more code and maintenance.

We can improve this at a later stage incrementally by:

- adding smoke tests in the future which assert that sample
  Providers/Applications (dockerized, including generated library part) can
  interact with each other through the middleware.
- Making these tests more dynamic by allowing control of Applications/Providers
  dynamically through (e.g. REST) APIs. This allows us to get more test coverage
  also for the generated code.

### Test concurrency

Instead of running each test against its own MW, we considered using unique
dynamic provider manifests for the tests. This also guarantees that E2E tests
can run concurrently without interfering each other.
