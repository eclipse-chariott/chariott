# Project Eclipse Chariott

<p align="center">
    <a href="https://github.com/eclipse-chariott/chariott/tags"><img alt="Version tag" src="https://img.shields.io/github/v/tag/eclipse-chariott/chariott?label=version"></a>
    <a href="https://github.com/eclipse-chariott/chariott/issues"><img alt="issues: N/A" src="https://img.shields.io/github/issues/eclipse-chariott/chariott"></a>
    <a href="https://github.com/eclipse-chariott/chariott/actions/workflows/rust-ci.yml"><img alt="build: N/A" src="https://img.shields.io/github/actions/workflow/status/eclipse-chariott/chariott/rust-ci.yml"></a>
    <img src="https://img.shields.io/badge/status-maintained-green.svg" alt="status: maintained">
    <a href="https://github.com/eclipse-chariott/chariott/blob/main/LICENSE"><img alt="license: MIT" src="https://img.shields.io/github/license/eclipse-chariott/chariott"></a>
</p>

## Index

- [What is Chariott?](#what-is-chariott)
- [How to develop with Chariott](#how-to-develop-with-chariott)
- [Trademarks](#trademarks)

## What is Chariott?

Chariott is a [gRPC](https://grpc.io) service that provides a common interface for interacting with
applications. It facilitates Service Discovery of applications which advertise their capabilities
by registering themselves with Chariott's service registry. Other applications that need to consume
resources and capabilities can Discover services through Chariott's service registry. There are two
components in this project: [Intent Brokering](./intent_brokering/README.md) and
[Service Discovery](./service_discovery/README.md). Today, they are two separate components which
do not interact. Each has its own gRPC service.

With Intent Brokering, applications can also communicate with each other through Chariott. This is
done by having applications register an _intent_ which Chariott will then _fulfill_ by brokering
the communication with the appropriate application to fulfill that intent. Applications which
fulfill these intents are known as _providers_. More information on Intent Brokering's current
design with diagrams can be found [here](./docs/design/intent_brokering_design.md).

## How to develop with Chariott

Chariott provides gRPC interfaces to interact from a client application. The
client application can be written in any language that supports gRPC. The
examples in this repository are written in Rust, but the same concepts apply to
any language.

## Trademarks

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft
trademarks or logos is subject to and must follow
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
