# Integration- and E2E Testing

- Status: accepted

## Context and Problem Statement

The ADR is based on [ADR 0010](0010-e2e-tests.md). We do not consider
unit tests as part of this ADR, we only decide about integration and E2E tests.

The repository contains the following components that can be tested:

- Chariott runtime
- Docker images
- Examples (+ ESS)
- [charc][charc]

## Decision Drivers

Ensure system does behave and run correctly through an end to end scenario.

## Decisions

We will run E2E and integration tests in a pipeline on each PR that is targeting
our default branch. We will ensure that all our tests can be run locally, as
well as from the CI.

### Integration tests

Integration tests will be invoking the Chariott runtime server code directly,
without using the [upstream gRPC
layer](../../intent_brokering/proto/chariott/runtime/v1/runtime.proto). We will not mock out any parts
from the Chariott runtime when integration testing, hence we will need to set up
a provider that is exposing a gRPC endpoint in order to handle intent
fulfillments.

Because providers use a port, we will start by running the tests in series. We
can support concurrent test execution later.

### E2E Tests

For E2E tests, we will serve the Chariott runtime and some example applications
(e.g. the Key-Value store) to be able to exercise the most important paths
(Read, Write, Events, Discover, Inspect). We will not restart the components on
each test, but ensure that tests do not cause interference by choosing unique
identifiers for tests when appropriate.

The E2E tests will start from a gRPC client (to include the Chariott Runtime
gRPC layer), communicating with the runtime, eventually reaching the example
providers. This means that we will cover the ESS, some example providers, the
runtime and the gRPC layer in our tests (and potentially Docker images if we
choose to Dockerize our components).

This approach is equivalent to the setup in [ADR
0010](0010-e2e-tests.md).

## Alternatives Considered

### E2E Tests

- We could use the [charc][charc] to drive the E2E tests. The script is so far
  primarily used to drive an interactive demo, aiming to make it easier to
  understand the value proposal of our repository for new users. By reusing the
  same script for driving the E2E tests, we:
  - ensure that examples and scripts are in a working state and covered by
    tests.
  - ensure that the script user-friendly (as we want to write succinct E2E
    tests).
  - use a very minimal/reusable, Rust-independent setup for our E2E tests.
    However, by using Bash we are not platform-independent.

  The E2E test script(s) would source [charc][charc] as a means to
  interact with the Chariott runtime. Executing an E2E test will then become a
  matter of executing a Bash script. There are slight variations of this
  approach:
  - The demo script can either be written in:
    - Bash, which is well-known in the *nix community, but comes with only
      limited functionality with respect to command-line parsing, etc. It is not
      truly cross-platform.
    - Powershell Core, while cross-platform, is less-well known but comes with
      richer semantics.
  - The tests can either be written in:
    - Bash, for homogeneity, and direct usage of the demo script for E2E tests
      by involving no other programming language. This would mean that we need
      to provide our own logic as a replacement for a test framework, or take a
      dependency on a test framework, such as
      [BATS](https://github.com/bats-core/bats-core).
    - Rust, which would need to "wrap" the demo script, but then integrate with
      `cargo test`, and come with the advantages a test framework has without
      taking a dependency on test framework which we do not already use.

[charc]: ../../intent_brokering/tools/charc.md
