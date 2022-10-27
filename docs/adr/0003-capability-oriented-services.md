# Capability-Oriented Services

**Status**: superseded by [ADR-0013](./0013-programming-model.md)

**Authors**: Atif Aziz, Bastian Burger, Daniele Antonio Maggio, Tim Park, Maggie
Salak, Patrick Schuler

**Date**: 29.04.2022

## Context and problem statement

We need the ability for the service-oriented automotive applications that offer
services to register their services under capabilities and other applications to
discover and connect based on a subset of those capabilities. This would enable
a more dynamic ability for application to connect beyond just some known set of
service interfaces. We will use the following example as a basis for thought
processes and code snippets in this ADR.

### Navigation Example

Navigation application that consists of a routing service that has an in-vehicle
and a cloud delivered implementation of the same routing capability. The cloud
delivered portion provides routing based on an enhanced set of data (like
traffic) while the in-vehicle option provides baseline topography based routing
even when disconnected. It is an unbundled application in that it is installed
and updated just like any 3rd party application would be.

In addition to these two application delivered capabilities this application
requires access to the following general vehicle capabilities:

- GPS Location
- Traffic information (either supplied via radio or other network)

## Decision Drivers

- Multiple providers must be able to offer the same capability with different
  conditions for their usage, Quality of Service (QoS) or priority.
  - An example for QoS is the desired language in a text-to-speech scenario or
    the resolution of a GPS location service
  - A case with special semantics is a maximum allowed latency for a response.
    This will be discussed in detail in the ADR.

> Note: A provider is in this document used as a synonym for an application
> providing a capability.

- Applications should work on different car platforms. Some capabilities are not
  critical to the operation of the application and should be treated as
  optional.
- Providers, applications and matching them via capabilities takes into account
  versions of providers and capabilities.

## Out of scope

- In the document we assume that we have access to some sort of well-formed
  manifest. We assume that the manifest can be trusted; How exactly we ensure
  that it can be trusted is out of scope.
- The control plane is out of scope of this ADR and we will make as few
  assumptions as possible about the control plane.
- For the purpose of this ADR we assume that capabilities are associated with
  well-defined interfaces. Applications and provider could have generated SDKs
  that ensures that the capability interfaces match.

## Decision Outcome

### Registration and Metadata

All applications define the requirements they have for execution in terms of the
capabilities they require and the specific permissions they need against them.
The middleware enforces that the application is only able to access these
capabilities and only with the permissions detailed in the manifest. Below is an
excerpt of how a manifest may look like. It contains elements from other
decision outcomes.

```yaml
name: navigation-app
namespace: contoso-navigation.de
type: application
spec:
    requires:
    - capability: gps
      required: true
      version: 1
      namespace: contoso-car.de
      permissions:
      - name: location
        type: field
        access: read
      policies:
        latency:
          le: 300ms
    - capability: network
      # app will still function in vehicles without network
      # the local provider will always be used
      required: false
      version: 1.2
      namespace: contoso-car.de
      permissions:
      - name: available_bandwidth
        type: field
        access: read
    - capability: traffic
      required: true
      version: 1.0
      namespace: contoso-navigation.de
      permissions:
      - name: query
        type: function
      policies:
        staleness:
          le: 30s
```

The network traffic provider's manifest looks like:

```yaml
name: traffic-network
namespace: contoso-navigation.de
type: provider

spec:
    capabilities:
    - capability: traffic
      version: 1.0.0
      priority: 100
      policy:
        available_bandwidth:
          ge: 300kbps
        staleness:
          le: 1m
```

And the radio traffic provider's manifest looks like:

```yaml
name: traffic-radio
namespace: contoso-navigation.de
type: provider

spec:
    capabilities:
    - capability: traffic
      version: 1.0.1
      priority: 20
      policy:
        staleness:
          le: 15s
```

The control plane could take control for starting and stopping the providers
based on the manifests (which is outside the scope of this ADR).

At provider startup, [the respective manifest/provider is registered with the
middleware](./0001-service-discovery.md). This enables the middleware to route
incoming requests for that capability to the respective providers backing that
capability using policy.

For example, when the driver asks for a route between Zürich and Bern, the
application requests the `traffic` capability and then asks that capability for
traffic information. Per the bundled policy, the middleware does this by
requesting a maximum latency for the response. The middleware executes the
`traffic-network` provider (if the available bandwidth, which is a value in the
V-Store, is high enough) and the `traffic-radio` provider in parallel to ensure
it can deliver a response within the requested latency. This decision is
transparent to the application, and this fact allows the application to both be
isolated from these details, but also be equally deployable in a lower end
vehicle without a network unit. In this case, only the `traffic-radio` provider
would be likely deployed to the car, and it would be the only `traffic` provider
to the application, so the middleware would always choose it, but all of this
would happen behind the scenes and would not require any modifications to the
application.

### Static Capability Validation

We will validate as early as possible whether an application's required
capabilities are met, which means that we validate before the application is
actually deployed in a car. The following properties will enable deployment-time
validation of capabilities:

We need to compare what is offered with what is available through:

- the set of required capabilities (incl. versioning)
- the set of optional capabilities (incl. versioning). This could be used by the
  middleware or the control plane to conditionally start providers, based on
  whether a consumer is present. We will not discuss this optimization further
  in this version of the ADR.
- the set of offered capabilities (incl. versioning)

### Optional Capability Detection

In addition to required capability validation, we assume that it is useful for
applications to know which of its declared optional capabilities are provided in
the car. Whether this is done at runtime (through querying the middleware) or at
deployment time (through configuration injection via the control plane or SDK
generation) depends on the control plane and is therefore outside the scope of
this ADR.

### Priorities, Intents, Quality of Service policies

#### Evaluation

The middleware decides which provider should be used to handle requests. The
clients can specify their intent and requirements in the requests through a
mutually agreed mechanism (e.g. a tag for a maximum latency for a function
invocation, required level of precision, etc.). The middleware offers a
best-effort solution to address the requirements but in certain cases (e.g.
increased load, no bandwidth) some preferences might not be met. The middleware
then choses the best option for the current runtime environment to answer the
request from the application.

In a future version, we will consider that apps can indicate their safety level
and providers can be dedicated to certain levels. For example, critical and
safety-related capabilities would always be delivered from the local provider.

#### Priorities

A provider declares the following metadata when offering a capability:

- `priority`: a unsigned integer inversely proportional to the order of which
  the results should be preferred. Default priority is 0 (maximum priority). As
  an example, if provider 'A' is offering capability 'X' with priority 100 and
  provider 'B' is offering same capability with priority 50, if all QoS policies
  and constraints are met for both, result from provider 'B' should be preferred.
- `default`: a boolean defaulting to false. This should be set to true when
  current provider should behave as fallback in situation where other providers
  are offering the same capability but their QoS policies are not met. As an
  example, if provider 'A' is marked as default for capability 'X' and provider
  'B' is offering same capability but with higher priority, if all QoS policies
  and constraints are met for both, result from provider 'B' should be
  preferred, otherwise if QoS policies from provider 'B' are not satisfied,
  results from provider 'A' should be used as fallback.

#### QoS Policies

In terms of QoS policies, application declaring `request` and providers are
declaring `offer`; in other words an application could request for a policy to
be matched while a provider could announce offering policies for a specific
capability.

The middleware should be aware of at least the following kinds of policies:

- `MATCH`: where a match is requested for a specific value. A provider can
  statically or dynamically (or e.g. via the v-store) expose a value that will
  be used for comparison in the middleware. As an example, the navigation
  application could `request` to match English as preferred language for
  text-to-speech and there might be multiple providers to `offer` different
  text-to-speech languages. This policy must be both `request`ed and `offer`ed
  to provide a match. Evaluation of this policy happens before we select a
  provider.
- `THRESHOLD`: where an application is defining the maximum allowed threshold
  for a value to actually accept a response from a capability providing
  service/application. The evaluation of this kind of policy might be immediate
  if the value is available as reference in v-store, or might be deferred in
  scenarios like the `LATENCY_BUDGET` described below. This policy should only
  be `request`ed. Evaluation of a threshold policy may happen before or after we
  select a provider, depending on the specific policy.

We will allow the specification of requested policies both statically in the
manifest, and at runtime requested on a request-by-request basis by an
application. This solution can be implemented incrementally, starting with
static policy evaluation and in a later stage taking into account dynamic policy
evaluation.

Let's imagine, with regards to the hybrid-navigation scenario, that an
application is requesting the ETA for the current set trip. The middleware
registry includes both a local and cloud provider that can satisfy the
requirement of providing an ETA. Because of external factors, the application
requesting the ETA needs the value before 100ms to go on with further
computations. It is foreseeable to define and apply a QoS policy about maximum
accepted latency, such as a `LATENCY_BUDGET` QoS `THRESHOLD` policy.

At the same time, we can also imagine that the "cloud" provider might be
unavailable or might not answer within the maximum allowed threshold for the
latency. Therefore, a `DEFAULT` has to be specified to the local service capable
of acting as fallback in this scenario.

Last, but not least, the two services should define a `PRIORITY` for those
scenarios where both are providing a response in the allocated slot of time
requested by the `LATENCY_BUDGET` QoS `THRESHOLD` policy. We might assume a
higher priority for the cloud based provider, as it is capable of offering a
more accurate ETA.

The middleware has now all the required information to be able to properly route
the request to the correct provider based on the defined and announced policies.
In this particular "race" scenario, the `LATENCY_BUDGET` is not a value that
could be found in v-store, therefore the middleware needs to call both providers
and eventually cancel any request to the cloud provider if that is taking more
than the requested `THRESHOLD` and if a `DEFAULT`-marked provider is able to
return a response before the defined threshold. Instead, if both providers are
returning a result before the 100ms time frame, the cloud provider based result
should be returned to the application as it is the one with highest `PRIORITY`.

> Note: Since providers race against each other in the scenario where we require
> a certain latency, we must guarantee that the different providers do not
> conflict with each other. For example, two cloud providers based on the same
> cloud store would need to ensure transactional guarantees if they are having
> side effects that may influence each other.

### V-Store and Events

Providers will be able to write values into the V-Store. Changes to these values
can be observed by different applications via the event subsystem. We will
implement a last-write wins semantics on these values in the V-Store initially
and leave it up to the user to make sure that publishing the same value from
different providers does not conflict. While this is a very simple solution
(discussed in the considered options section), it can be extended and improved
in later iterations.

### Versioning

To support updating capabilities to newer versions including breaking changes,
we require the registration of a capability to indicate its `version`. The
capability providers indicate their current version in the manifest and the
consumer applications can indicate with what they are compatible.

For example, the navigation app requests the route capability to be 1.x and
there are 4 providers registered:

- cloud, 1.0.1
- cloud, 2.0.0
- radio, 1.0.2
- radio, 2.0.1

To ensure backward compatibility request from the navigation app are routed to
cloud, 1.0 and not the new major version 2.0. To access the latest feature, the
consuming application will need to be updated to work with the latest version.

## Considered Options

### Optional Services

Capabilities resolve to services (~= providers and/or applications) that are
able to provide the capability. Therefore, we can think about an optional
service on two levels, namely:

1. an optional service being present for a mandatory capability, which means
   that the capability must be provided by *at least one* service at all times,
   though some services may at times be unavailable/may be deployed at a later
   time. We do not discuss this further, as this is one of the main advantages
   that capability-oriented resolution would offer and it typically involves
   multiple services providing a capability.
2. an optional service resulting in an optional capability as well: a capability
   which may or may not be provided in the car at any time. In the following we
   will discuss this scenario.

The existence of optional capabilities implies that there are also mandatory
capabilities. It would be desirable to validate as early as possible whether an
application's required capabilities are met before the application is deployed
in a car. This means that there must be some mechanism to match the required
capabilities of an application (e.g. based on a manifest) with the capabilities
that are always present in a given car.

- Assuming that we do not have deployment-time validation of capabilities would
  mean that *all* capabilities are treated as optional during deployment time.

### Client vs. Middleware Intent/Policy Evaluation

We can consider delegating this fully to either the client side or the
middleware. Below we outline the pros and cons of each option.

#### Evaluation on the client

This option would mean that the application itself determines whether it
requires a certain capability to be provided by a local service (provider) or
via a call to the cloud.

Advantages:

- In the current thinking, the middleware is not aware of the purpose of the
  application, e.g. whether it's a critical safety-related app requiring precise
  real-time input or an entertainment app which is not at all related to the
  car's driving capabilities. Because of this, the app itself is best suited to
  determine how the data should be provided – e.g. what is the maximum allowed
  latency and how precise the data must be.
- Having the clients responsible for evaluation of how the data should be
  provided would simplify the middleware. The middleware would simply follow the
  request to either make a call to the cloud or fetch the data from a local
  provider and would not need to know about the client's intentions.

Disadvantages:

- App developers might tend to request more expensive cloud data to improve the
  user experience of that particular application, without giving concern for the
  overall data usage.
- The logic to choose between providers is likely to be similar between
  applications, which duplicates logic between apps and adds more to the app
  developers backlog.
- Apps might not be aware of which capabilities are offered by cloud providers
  and which are not. The middleware would still need to check whether the
  selected provider is available and, if not, use a different one.
- Apps would have a tight understanding and binding around the providers. This
  makes it harder on the application developers to have their app span car
  models with different capabilities (e.g. imagine a low end car having
  navigation, but no cloud connectivity) - the app would have to manage all of
  these differences itself.

#### Evaluation on the middleware

In this scenario the middleware would determine if the requested data should be
fetched from the cloud or a locally available service. Client requests could
specify their intent or preferences to a certain extent, e.g. using tags such as
min available bandwidth for a cloud call, required level of precision, etc.

Advantages:

- Centralizes the logic of selecting providers such that it reduces app
  developer effort.
- The middleware can make decisions based on available bandwidth and request
  load which clients are not aware of.
- If the clients specify their intent (e.g. using tags on the requests), the
  middleware can also take into account client-specific requirements and
  prioritize them.
- The middleware could also take into account overall costs such as data usage
  and (potentially) request patterns of certain apps to decide how to handle new
  requests. Possible optimization would be introducing caching on the middleware
  for most frequent requests for cheaper and faster processing which would be
  transparent to clients.
- Load balancing mechanism could be introduced on the middleware to distribute
  requests over local and cloud providers in situations of increased load.

Disadvantages:

- The approach with clients tagging their requests with intent/preferences has
  its limits. The middleware would still lack the knowledge of the overall
  priority of a certain client (e.g. safety-related app vs entertainment) or how
  critical the functionality that is requesting the data is. A possible solution
  here could be to express the priority of the app in a token that is sent with
  the request.
- The mechanism of evaluating the requests in the middleware with the use of
  tagging, if introduced, could be challenging to build and maintain. It might
  be difficult to come up with a set of tags which will be specific enough and
  exhaustive to fairly prioritize requests from different clients but at the
  same time flexible enough for the app developers to use.
- Certain apps might require different providers depending on the current
  circumstances. For example, a car navigation app might prefer to get the
  current location from a location service provider instead of the GPS signal if
  the accuracy is lower than required. On the other hand, the required accuracy
  could be expressed in the request and the middleware itself could decide which
  provider to choose.

### Policies

Currently we only consider policies that must match. It would be useful to have
more selection options (`MUST`, `PREFER`, `REJECT`). Also, there are several
interesting policies worth considering:

- Circuit breaker policy
- "Ensured delivery": Supports offline scenarios by queuing requests when no
  provider is currently available to handle the request. The middleware will
  dispatch the queued requests as soon as a provider which can handle the
  requests comes online.

While we can support these policies in the future, we will not do this in an
initial version.

### V-Store and Events

> Note: this section is a discussion for *capability providers* and is not
> relevant for *hardware providers*.

If we allow capability providers to write values into the V-Store, this would
enable applications to standardize and/or provide vehicle state. Changes to
these values can be observed by different applications via the event subsystem.
As multiple providers are able to write the same values, we must decide how we
handle the values from different providers. Since different providers have
different associated QoS values, we need to decide how (and whether) to apply
policies to the values generated by the different providers.

There are two main approaches we can take there:

- We use last-write wins semantics for the same value generated by different
  providers in the V-Store.
  - This can be extended in the future by annotating these values with QoS
    values from the provider by which it was generated
  - This could mean that the values from the V-Store are synced into the cloud.
    We could allow more fine-grained control in the future by specifying the set
    of values synced into the cloud.
- We filter value changes in the middleware through the generating provider's
  QoS values and bubble up only events that match policies to providers.
  - This is potentially hard to do, as different applications have different
    policies and therefore caching the events for all providers and filtering in
    the middleware may be complex.
  - Since events have different semantics than functions, policies such as a
    latency threshold may not make sense for events.
