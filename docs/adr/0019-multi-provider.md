# Multi Provider

- Status: accepted
- Authors: Patrick Schuler

## Context and Problem Statement

Chariott allows multi provider registration for the same namespace and intent.
Intent fulfillment requests from application may result in multiple potential
target providers. This ADR describes the strategy to choose a single provider for
those scenarios and how to execute the intent request.

## Terms

- Namespace: hint from an application request side what service the intent is
  directed at
- Registry - Registry for all intent providers (see [ADR 0012](./0012-intent-registration.md))
- Provider: applications registering intents with the registry offered for a
  particular namespace
- Consumers: applications requesting intent fulfillment with chariott

## Requirements

1. Multiple providers for a particular namespace and intent combination are
   supported
2. Consumers receive only one intent fulfillment response from a single provider
3. The chariott broker handles the selection transparently
4. A list of potential providers is used to try to satisfy the request. If a
   provider fails, the runtime does fallback to the next one.

## Considered Evaluation Options

### Constraints and Properties

The system allows providers to specify n-number of properties with their
registration. Those are free form and a provider can add any key-value pair they
desire. The consumers do specify constraints with their requests and can
indicate if they are desired or required to be fulfilled. For example, the
consumer can indicate that it "desires to be executed in the cloud", for example
`desire: cloud=true`. Registrations that have provided cloud=true in their registration
properties would be chosen over registrations that do not satisfy that request.
Multiple candidates would be resolved using round-robin.

#### Pros

- Flexible system that can be extended easily to other use-cases
- No knowledge in the chariott required for specific handling

#### Cons

- Consumers and providers need to agree on a format for their properties /
  request expressions
- Specific use cases handling is harder to implement (conditional cloud routing,
  when we have enough bandwidth for example)
- Higher complexity to implement while known use cases for multi provider are
  limited at the moment
- Consumers need to express their intents more precisely

### Priorities

Providers can specify a priority when registering. This would allow, for
example, a cloud provider to specify a higher priority than a local provider
(delegating compute to the cloud as a fundamental decision). Chariott would then
sort the providers by priority and start with the highest priority. If that
fails, the next in the list will be used to try to fulfill the request.
Same priorities would be resolved using round-robin.

#### Pros

- Consumers don't need to have any understanding of specifics about registered
  providers (properties for example)
- Consumers don't need to express anything besides the intended request
- Logic for handling the request is simpler than with the constraint system

#### Cons

- Less flexible than free constraints / properties matching
- Optimized handling for specific use cases is harder to implement
- More logic on how to resolve multiple providers in the chariott runtime
- No evidence of use cases that require the flexibility at the moment

### Cloud and Local Types

Adding a simple type annotation to the registration message for the providers,
allows providers to choose from a predefined set of options for their type. For
example: `cloud`, `local`. The chariott runtime does have an explicit
understanding of types and their priorities. In case of connectivity (to be
defined what this exactly means) `cloud` providers are receiving higher priority
executing intent requests. If there is no connectivity `local` is chosen. If a
request to a `cloud` provider fails, we fallback to the `local` implementation
should it exist.

#### Pros

- Targeting the specific use case, we have for multi provider (local vs. cloud
  execution)
- Specific handling of the cloud provider type (connectivity validation) is
  possible
- Consumers don't need to have any understanding of specifics about registered
  providers (properties for example)
- Consumers don't need to express anything besides the intended request
- Logic for handling the request is simpler than with the constraint system

#### Cons

- Targets the cloud use case only.

## Considered Execution Options

### Serial

Multiple providers are processed one by one (in case of failures). We assume
that most of the time the initial selection of the first provider will yield a
valid response and do not optimize for latency.

#### Pros

- Only performing work needed and hence saving compute
- Lower complexity

#### Cons

- When the first provider fails, the latency will increase
- No latency optimized scenario (racing) - see Parallel

### Parallel

Multiple providers are processed in parallel. There are multiple options on how
to chose which result to pick from potentially multiple returning. If we have
limited connectivity or changing bandwidth constraints while we are executing,
the decision might have negative impact on the latency for the request. This can
be mitigated by simply firing multiple parallel requests and act only on the
ones that come back within a defined latency (consumer specific).

- Choose the fastest coming back and abort the other - latency optimized
- Use a max latency definition and then chose the highest in priority that comes
  back within the allocated latency - potentially highest quality result within
  possible latency constraints

#### Pros

- Optimized latency for requests - also when fallbacks need to be executed
- Additional scenarios (racing) are possible

#### Cons

- Higher complexity managing multiple requests and synchronizing them
- Potential throw away work performed on limited compute (local)

## Decision

### Evaluation

We decided to use [Cloud/Local Types](#cloud-and-local-types) when evaluating
multiple providers to limit the implementation complexity and support the most
obvious use case we have. This will ensure we do not over engineer the solution.
If there are new requirements coming up, we reconsider the other two options to
add more flexibility based on the scenarios. The team agrees that we would go
with a more open system using constraints/properties if there are use cases
requiring more flexibility.

### Execution

For the CTP-2 milestone we start with the simpler [serial processing](#serial).
We would like to add more options to the execution model that are driven by
request policies from the consumer. The consumer should decide based on the
requirements regarding latency and compute utilization, how a request is
processed. We will consider this in the next milestone.
