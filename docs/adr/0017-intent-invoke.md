# Intent Invoke

- Status: accepted
- Authors: Daniele Antonio Maggio
- Last updated: 2022-08-22

## Context and Problem Statement

Chariott provides an API for applications wanting to invoke a command on a
service, without having the above mentioned application to first discover the
service and then directly connect to it for fulfilling the method invocation.

## Terms

- Application Programming Model (APM)
- APM Registry - Registry for all intent providers (see [ADR 0012](0012-intent-registration.md))

## Requirements

1. Applications should be unaware of which service is offering a specific command.
2. Applications should only be responsible of specifying a properly namespaced
   command name and correct arguments for its invocation.

## Decision

Any service offering the ability to call a publicly exposed command, has to
register each command, properly namespaced, as a separate `Invoke` intent
registration in the APM registry.

An application which wants to invoke a previously registered method in the APM
should be adhering to the `InvokeIntent` described in the following proto
contract. The invoker has to be sure that the command name is properly
namespaced and that all the required arguments are properly specified.

```proto
syntax = "proto3";

package chariott.common.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/any.proto";

message InvokeIntent {
    string command = 1;
    repeated Value args = 2;
}

message InvokeFulfillment {
    Value return = 1;
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
message List {
    repeated Value value = 1;
}
message Map {
    map<string, Value> map = 1;
}
enum NullValue {
    NULL_VALUE = 0;
}
```

A downside to take into account for this approach is the fact that there are no
strict/strong signatures for command invocations. While this approach is
providing more possibilities for extensibility and definitely easier
manageability, it could be more error prone and eventually less descriptive.
