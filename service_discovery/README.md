# Service Discovery

- [Introduction](#introduction)
- [High-level Design](#high-level-design)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Building](#building)
  - [Running the Tests](#running-the-tests)
- [Developing with Service Discovery](#developing-with-service-discovery)
- [Running the Service](#running-the-service)
- [Sample Applications](#sample-applications)
- [Trademarks](#trademarks)

## Introduction 
This Service Discovery system for Eclipse Chariott aims to simplify in-vehicle application development by abstracting out where services are running, so that applications wanting to leverage the resources and capabilities of another application can find the location and metadata necessary to communicate. This is a new version of Service Discovery which is decoupled from the intent broker, and it is currently under development. If you would like to use the existing version of Chariott with Service Discovery and intent brokering, please refer to the top level Readme for details (TODO: ADD LINK)

The [Getting Started](#getting-started) section shows how to run the Service Discovery system locally and get started.

The [Developing with Service Discovery](#developing-with-service-discovery) shows what is necessary to develop an application to use with Service Discovery.

## High-level Design
The service discovery system consists of 3 parts:
1.	The Service Registry
2.	A provider 
3.	A consumer

The Chariott Service Registry stores enough metadata about a service for a consumer to be able to communicate directly with the provider. A service is uniquely identified by its namespace, name, and version. Today, there can only be one service registered with the same namespace, name, version combination, and any attempt to register again will fail.

Providers and consumers are just applications, or any software components. An application can take on the role or "provider", "consumer", or both. A provider is an application which registers itself and gets added to the service registry, whereas a consumer is an appplication which searches for applications in the service registry, using one of the read operations on it (i.e. Discover to retrieve a single service).

## Getting Started 
### Prerequisites
This guide uses `apt` as the package manager in the examples. You may need to substitute your own
package manager in place of `apt` when going through these steps.

1. Install gcc:

    ```shell
    sudo apt install gcc
    ```

    > **NOTE**: Rust needs gcc's linker.

1. Install git, git-lfs, curl and [rust](https://rustup.rs/#) (using the default installation), for example:

    ```shell
    sudo apt update
    sudo apt install -y git git-lfs curl
    git lfs install
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
    You will need to restart your shell to refresh the environment variables.

    > **NOTE**: The rust toolchain version is managed by the rust-toolchain.toml file, so once you
                install rustup there is no need to manually install a toolchain or set a default.

1. Install Protobuf Compiler:

    ```shell
    sudo apt install -y protobuf-compiler
    ```

    > **NOTE**: The protobuf compiler is needed for building the project.

### Building
Running the following in the root directory of the Chariott repository will build everything in the service_discovery directory as well:

```shell
cargo build --workspace
```

Alternatively, you can run build in the core directory at `chariott/service_discovery/core`:

```shell
cargo build
```

### Running the Tests
After successfully building the service, you can run all of the unit tests by running the following at `chariott/service_discovery/core`:

```shell
cargo test
```

## Developing with Service Discovery

## Running Service Discovery

Below are the steps to run Service Discovery, which is gRPC based,
and an easy way to interact with it is through the use of the
[grpcurl](http://github.com/fullstorydev/grpcurl) command line tool.

### Start the Service

In a terminal window, run: 

```shell
cargo run -p service_discovery
```

### Interacting with the service

This service implements the gRPC methods defined in [service_registry.proto](./proto/core/v1/service_registry.proto)

Replace your path to service discovery directory in the command:

```shell
grpcurl -proto {path_to_service_discovery}/proto/core/v1/service_registry.proto -plaintext -d @ 0.0.0.0:50000 service_registry.ServiceRegistry/Register <<EOF
{
  "service": 
    {
      "namespace": "sdv.samples",
      "name": "service1",
      "version": "1.0.0.0",
      "uri": "https://localhost:1000",
      "communication_kind": "grpc+proto",
      "communication_reference": "sample.communication_reference.v1.proto"
    }    
}
EOF
```

If successful, this will return an empty "Ok" response, which looks like

```shell
{

}
```

In order to discover your newly registered service you can call discover, again replacing the path to service discovery:

```shell
grpcurl -proto {path_to_service_discovery}/proto/core/v1/service_registry.proto -plaintext -d @ 0.0.0.0:50000 service_registry.ServiceRegistry/Discover <<EOF
{
  "namespace": "sdv.samples",
  "name": "service1",
  "version": "1.0.0.0",
}
EOF
```

This will return all of the metadata stored for that service.

You can also add more services under the same namespace, and then discover by the namespace with the following command. This is helpful if you want to retrieve all services that share the same logical grouping.

```shell
grpcurl -proto service_discovery/proto/core/v1/service_registry.proto -plaintext -d @ 0.0.0.0:50000 service_registry.ServiceRegistry/DiscoverByNamespace <<EOF
{
      "namespace": "sdv.samples"
}
EOF
```

Once you've added multiple services to the registry, you can also list all of them:

```shell
grpcurl -proto service_discovery/proto/core/v1/service_registry.proto -plaintext -d @ 0.0.0.0:50000 service_registry.ServiceRegistry/List <<EOF
{
}
EOF
```

And when you want to unregister a service you can use the following:
```shell
grpcurl -proto {path_to_service_discovery}/proto/core/v1/service_registry.proto -plaintext -d @ 0.0.0.0:50000 service_registry.ServiceRegistry/Unregister <<EOF
{
  "namespace": "sdv.samples",
  "name": "service1",
  "version": "1.0.0.0",
}
EOF
```

### Sample applications
Sample applications, including the "simpl discovery" sample to help get started with development can be found [here](./samples/README.md)

## Trademarks

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft
trademarks or logos is subject to and must follow
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/en-us/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
