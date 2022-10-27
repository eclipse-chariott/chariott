# Cloud Providers

- Status: deprecated (not valid for Chariott)
- Authors: Bastian Burger, Patrick Schuler
- Date: 02.05.2022

## Context

This ADR is an extension to the [capability-oriented services
ADR](./0003-capability-oriented-services.md). In this ADR we discuss how to
implement cloud-based capability providers that need not be associated with a
running process in the car.

## Decision Drivers

- Cloud-based providers add more flexibility to the SDV programming model in
  addition to the existing local providers.
- Cloud-based providers should be indistinguishable from local providers from an
  application's point of view.
- Cloud-based providers are distinct from local providers that communicate with
  the cloud.

## Out of scope

- AuthZ/AuthN
- How application SDKs are generated and how capabilities define their
  interfaces is out of scope.

We could support cloud providers by wrapping them in a custom local provider. In
that case, we won't have to make assumptions about cloud providers, as all
responsibility to communicate with the cloud is delegated to the provider
developer and we can treat it as we do with the existing capability model. Since
we do get this out of the box when we support capability providers in general,
no further effort is needed for this (except for Auth discussions, which are out
of scope for this ADR).

- This comes at the cost of duplication code that connects to a cloud provider.
  Each cloud provider would need to be wrapped in a local provider, which would
  typically be boilerplate/duplicated code.

## Considered Options

### Provider registration

The middleware needs to be aware which cloud providers can be accessed from the
car. As the car does not control directly the providers in the cloud, we would
need to ensure that a cloud provider fulfills certain conditions that apply to
all providers. Some options to consider:

#### Versioning

- We can include the version with each request (either in the payload or as
  metadata based on the protocol) when sending requests to the cloud provider
  and specify a valid version range in the provider manifest.
  - Requires this change also for local providers.
  - Local providers could potentially benefit from knowing which version the
    caller requested.
- We can specify different providers (e.g. different DNS names, paths) for
  different versions of a provider in the manifest.
  - Does not require a change as it is compatible with the current manifest
    structure.

#### Discovery

Cloud providers need to be integrated into the middleware capability registry. We
assume that cloud providers are highly available and hence we do not need to
keep track of the liveness of each provider. We consider the following options:

- We could imagine that the component which is responsible for translating
  deployment manifests into container deployments (e.g. an application, or the
  control plane) makes the manifests available to the middleware (e.g. as
  files). The middleware registers the cloud providers in the registry on
  startup.
  - takes a dependency on either the control plane or the application that
    interacts with the control plane to start containers from deployments.
  - manifest could be reused for other aspects as well, e.g. for capability
    detection (which potentially includes provider detection).
- A separate application registers the cloud providers:
  - We can have a separate application running that is aware of all cloud
    providers (e.g. an auto-generated application) responsible for registering
    all cloud providers to the middleware.
    - This application would have to be generated based on the information in
      the manifest.
    - Adding or removing a cloud provider means that this application will need
      to be regenerated and redeployed.
  - A local provider takes care of registering a cloud provider (providing the
    same capabilities!) using the same approach it uses to register itself.
    - The local provider would need awareness of the cloud provider.
    - Local providers would need logic to register on-behalf of cloud providers
      in addition to their own registration.
    - Cloud providers have fallback local providers if they are registered by a
      local provider with the same capability.
- We can have the cloud providers being registered in a remote registry
  component.
  - The middleware would need internet connection to detect from the remote
    registry which cloud providers are available.
  - Such a remote registry would need awareness of all deployed cloud providers
    for each car.
  - This would be another component that we would need to maintain/write.

### Method invocations on a cloud provider

An application should be able to invoke a method on a cloud provider. We can
consider the following strategies there:

- the middleware acts as a proxy for cloud provider communication. The
  application invokes a function as it would on a local provider, the middleware
  detects that it's a cloud provider and uses a well-known communication method
  (= contract) to communicate with the cloud provider.
  - The API defined between a remote provider and the middleware could be
    identical to the API to other local providers.
- we call the cloud provider from the application directly, but use the ACL
  layer from the middleware etc. This would suffer from the same disadvantages
  as we have for local provider calls that are made directly by applications.

### Writing values to the V-Store

It may be useful if a cloud provider publishes values into the V-Store that can
be observed as events in the car. This adds complexity for the cloud provider,
as it means that there must be a flow from the cloud to the car and into the
V-Store on the middleware. On the cloud side, there would need to be some sort
of mechanism for having a TTL on these messages, as the car is not guaranteed to
be connected to the internet all the time. We consider this as an incremental
feature that can be supported at a later point. For the time being this means
that cloud-provided capabilities do not include events, only methods.

## Decision outcome

We will treat remote providers as we treat local providers. Their metadata is
described in a manifest, which may look like:

```yaml
name: traffic-cloud
namespace: contoso-navigation.de
type: provider
spec:
    capabilities:
    - capability: traffic
      version: 1.0.1
      priority: 20
      policy:
        available_bandwidth:
          ge: 500kbps
      endpoint: contoso-cloud-traffic.de:1234
```

```yaml
name: traffic-radio
namespace: contoso-navigation.de
type: provider
spec:
    capabilities:
    - capability: traffic
      version: 1.0.5
      priority: 100
      default: true
```

- The key difference to local providers is the specification of an `endpoint`,
  which makes it clear that the provider is not a locally running instance but a
  cloud provider.
- A local provider will register a cloud provider (which is providing a subset
  of the capabilities of the local provider) to the middleware. If there is only
  a cloud provider for a capability, there needs to be a separate app taking
  care of the registration of the cloud provider.
- To support different versions of the same cloud provider we will rely on a
  separate manifest entry for each cloud provider version.
- The middleware accesses the cloud providers as it would access a local
  provider, which is based on synchronous communication from the [cloud
  connectivity ADR](./0002-cloud-connectivity.md). This means that the cloud
  providers need to implement the same API that the local providers implement.
- Cloud providers will not be able to publish values into the V-Store.
