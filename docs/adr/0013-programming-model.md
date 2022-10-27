# Programming Model

- Status: accepted
- Authors: Atif Aziz, Bastian Burger, Daniele Antonio Maggio, Dariusz Parys,
  Patrick Schuler
- Date: 10 August 2022

## Context

The goal of the CTP 2 is to focus on the definition of an _application
programming model_ (APM) that provides a unified means for client and server
applications to communicate with each other within the frame of a
_capability-oriented architecture_ (COA).

## Terminology

- Capability-Oriented Architecture (COA): An architecture where applications
  need and offer _capabilities_ without direct knowledge of each other and
  caring about how _intents_ are _fulfilled_ as long as a set of _conditions_
  are met.
- Application Programming Model (APM): An application programming model
  enabling COA.
- Chariott: an implementation of the APM.
- Intent: The request made by an application and brokered by Chariott for
  fulfillment by a capability offerer.
- Fulfillment: The response when for intent that has been completed by a
  capability offerer.
- Application: A program running in an untrusted zone that can assume the
  role of a client, server or both.

## Decision Drivers

- All the features of the model need to be exposed through a well-defined public
  API.

- Applications should be able to interact/communicate with Chariott without
  requiring an SDK or a library. It is expected that in the next iteration (CTP
  2), developers will use tools available in their development
  platform/environment to generate proxies for the public API.

- The APM should provide means to:
  - register applications that can fulfill intents
  - read/write properties/state
  - invoke commands
  - inspect properties, commands and telemetry of objects like that which
    comprise a vehicle digital twin
  - publish events
  - subscribe to events
  - discover registered providers

- Requests can dynamically be routed between different implementations based on
  qualified conditions.

- Application requests can dynamically be routed to cloud-based services.

- Applications can discover other services with which they can communicate
  _directly_ through some end-point and custom interface.

### Additional Considerations

- Policies must be respected within the communication with the APM.

## Out of scope

- AuthN/Z
- Control plane
- Support for multi publisher/single subscriber
- Any consideration of possible persistency of APM state. Currently not
  persisted.

## Considered Options

### CTP 1

In CTP 1, the middleware contained an integrated cabability
registration/discovery service.

#### Pros

- Simple to understand and implement
- Load-tested

#### Cons

- Values are opaque
- Requires language bindings
- Limited to functions and data

### In-Process COA

In-process library supporting querying a central registry and resolving to
different capability provider applications using constraints/conditions.

#### Pros

- Very flexible and extensible
- Multi-vendor support
- Dynamic selection of vendor based on conditions
- Extensive constraint handling
- Multi-bindings to different provider application end-points

#### Cons

- Requires extra effort to host out-of-process
- Dependencies are distributed (in-process)
- Requires different implementations for each language
- No dynamic change of vendor selection
- Decentralized decision about cloud connection
- Complexity of code maintenance for community

### Dapr

#### Pros

- Mature system
- Control plane available
- Easily extensible

#### Cons

- Large footprint
- No conditions for vendor selection
- Uses a side-car model

## Decision outcome: Out-of-Process COA

As none of the options satisfy the requirements laid out in the beginning of
the document, we decided to go with an out-of-process and service-based
implementation (Chariott) of the capability broker and registry. Chariott will
expose the APM via a public gRPC API for widest interoperability.

The API will be based on concepts such as _intents that get fulfilled_ to
enable a capability-oriented architecture.

Provider applications will register intents that they can fulfill in a central
registry. The semantics of an intent fulfillment will be implied by some
namespace. This is necessary for cases where the return value of the intent
specifies a type (e.g. `Value value` from the example in the
[appendix](#appendix)) which has no information about its semantics. It will
still be possible to support multiple provider applications offering the same
intent for the same namespace.

Consumer applications will express intents that need to be fulfilled. Chariott
will select the provider application that can fulfill the intent, forward the
request and deliver the result back to the initiating application.

Chariott can decide to dynamically select vendors that can satisfy the intent,
e.g. based on conditions such as the currently available network bandwidth.

Chariott can further decide to fulfill or reject the forwarded request based
on additional conditions.

For direct application-to-application communication, Chariott will support an
intent to discover endpoints of the services that adhere to some standardized
or custom interface. For more details, see [the ADR on the discovery
intent][discover-adr].

For CTP 2, implementation of conditions and policies will be a stretch goal.

### Pros

- Rich in function and semantics.
- Does not prevent direct application-to-service communication.
- Out-of-process implementation limits the dependencies in the application and
  thus their servicing.
- Protos allow different language bindings to be generated using readily
  available developer tools.

### Cons

- Highly generalized APIs around intents
- Adds another network hop compared to an in-process broker
- To be developed

### Appendix

An intent is an operation that is standardized in the APM. As an example,
there is an intent to read a value from some sort of store. The type of the
value can vary and depends on what is being read. In Protocol Buffers, this
can be expressed as (where `Value` is a union or "`oneof`" different types):

```proto
message ReadIntent {
    string id = 1;
}

message ReadFulfillment {
    Value value = 1;
}

message Value {
    oneof value {
        NullValue null = 1;
        google.protobuf.Any any = 2;
        bool bool = 3;
        int32 int32 = 4;
        double double = 5;
        string string = 6;
        google.protobuf.Timestamp timestamp = 7;
        List list = 8;
        Map map = 9;
    }
}
```

This implies that all applications that provide this intent will need to
adhere to this contract. Intents can also be more general, e.g. an intent to
inspect:

```proto
message InspectIntent {
    string query = 1;
}

message InspectFulfillment {
    message Entry {
        string path = 1;
        map<string, Value> items = 2;
    }
    repeated Entry entries = 1;
}
```

In this case, it is up to the issuer to know the semantics of the fulfillment
result based on what is being inspected.

[discover-adr]: 0014-intent-discover.md
