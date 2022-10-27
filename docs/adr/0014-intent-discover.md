# Discovery Intent

- Status: accepted
- Authors: Atif Aziz, Bastian Burger, Daniele Antonio Maggio, Dariusz Parys, Patrick Schuler
- Last updated: 2022-07-27

## Context and Problem Statement

The Application Programming Model (APM) needs to support the ability for
applications to discover service end-points exposed by other applications in
order to communicate with them directly.

If the provider does register the `discover` intent it will provide metadata
information to describe it's service offerings with endpoint and protocol
information for applications to directly communicate with it.

## Considered Options

### Merging with Intent to Inspect

One requirement is to be able to inspect the Vehicle Digital Twin contents. To
satisfy this requirement, we plan on having an intent that fulfills inspections.
Discovery can be considered a special case of inspection, and could hence be the
same intent as inspection.

Pros:

- Explicit interfaces for describing different requirements

Cons:

- Additional intents

### Decision Outcome: Dedicated Intent

We allow providers to register a discover intent. When registering, they are
required to handle the `DiscoverIntent` request. Providers should only register
for the discover intent if they expose endpoints for direct interaction outside
of the APM.

The discover intent request does not have any parameters. All exposed endpoints
and their respective metadata should be returned. We explicitly skipped adding a
query capability for now, as standardization on the implementation across
multiple providers was out of scope for the initial version.

The format of the response is described in the [proto
definition](#proto-definition).

#### proto definition

```proto
syntax = "proto3";

package chariott.v1;

service coa {
    rpc InvokeIntent(IntentRequest) returns (IntentResponse) {}
}

// discover vdt

message IntentRequest {
    Intent intent = 1;
}

message IntentResponse {
    Fulfillment fulfillment = 1;
}

message Intent {
    string namespace = 1;
    oneof intent {
        DiscoverIntent discover = 2; // this is an extension to other intents defined in other ADRs
    }
}

message Fulfillment {
    oneof fulfillment {
        DiscoveryFulfillment discover = 1; // this is an extension to other fulfillments defined in other ADRs
    }
}

message DiscoverIntent {
    //string query = 1;
}

message DiscoveryFulfillment {
    repeated Service services = 1;
}

message Service {
    string url = 1;
    string schema_kind = 2;
    string schema_reference = 3;
    map<string, Value> metadata = 4;
}

```
