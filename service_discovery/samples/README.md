# Samples

- [Introduction](#introduction)
- [Simple Discovery Sample](#simple-discovery-sample)
  - [Provider](#provider)
  - [Consumer](#consumer)

## Introduction

The samples show applications that can be used with the Service Registry. For any sample, make sure that you first have service discovery running:

```shell
cargo run -p service_discovery
```

## Simple Discovery Sample

This example shows a how a provider and consumer communicate, using the service registry to register and discover. The two applicaitons then directly communicate with one another, through a known interface. You can add the same logic to your own applications to get started using Service Discovery.

The provider application consists of one gRPC service, the Hello World service. This service just logs and returns a message with "Hello, " followed by the input string from the request. The provider registers itself with the service registry, so that it can be consumed by other applications. The consumer "discovers" the location of the hello world service by calling discover on the service registry. It validates that the communication_kind (which can include the protocol and api specification type) and communication_reference (a string to identify the api specification) are what it expects and knows how to communicate with. The consumer then calls the SayHello method on the uri provided in the discover response to use the provider directly. You will see the output of these operations in the terminal windows when you run the services.

### Provider

To run the provider, from the simple discovery provider directory in a different terminal window run:

```shell
cargo run
```

### Consumer

To run the consumer, from the simple discovery consumer directory in a different terminal window run:

```shell
cargo run
```
