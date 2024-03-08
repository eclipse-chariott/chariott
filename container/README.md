## Containers

This repository provides several Dockerfiles to enable building of OCI container images. This
document has instructions for building and running the provided Dockerfiles in
[Docker](#docker-containers) and [Podman](#podman-containers). Refer to the
[Dockerfiles](#dockerfiles) section to select the appropriate Dockerfile.

### Dockerfiles

#### Service Discovery

- [Dockerfile.service_discovery.amd64](../Dockerfile.service_discovery.amd64) - Dockerfile used to build the `Service Discovery Service` for the
x86-64 architecture.
- [Dockerfile.service_discovery.arm64](../Dockerfile.service_discovery.arm64) - Dockerfile used to build the `Service Discovery Service` for the
aarch64 architecture.
- [Dockerfile.service_discovery.multi](../Dockerfile.service_discovery.multi) - Dockerfile used to build the `Service Discovery Service` for multiple architectures based on the TARGETARCH argument. 

#### Intent Brokering

- [Dockerfile.intent_brokering.amd64](../Dockerfile.intent_brokering.amd64) - Dockerfile used to build the `Intent Brokering Service` for the
x86-64 architecture.
- [Dockerfile.intent_brokering.arm64](../Dockerfile.intent_brokering.arm64) - Dockerfile used to build the `Intent Brokering Service` for the
aarch64 architecture.

### Docker Containers

#### Prequisites

[Install Docker](https://docs.docker.com/engine/install/)

#### Running in Docker

To run the service in a Docker container:

1. Run the following command in the project root directory to build the docker container from the
Dockerfile:

    ```shell
    docker build -t <image_name> -f <Dockerfile> .
    ```

    For example, to build an image for the `service_discovery` component:

    ```shell
    docker build -t service_discovery -f Dockerfile.service_discovery.amd64 .
    ```

    Or to build a multi-platform image for the `service_discovery` component and push it to a
    container registry:
    You must first create a new builder using the docker-container driver, which gives you access
    to more complex features like multi-platform build. See more information here:
    [multi-platform builds.](https://docs.docker.com/build/building/multi-platform/#cross-compilation)
    
    ```shell
    docker buildx create --name multibuilder --driver docker-container --use
    docker buildx build --platform=linux/amd64,linux/arm64 -f Dockerfile.service_discovery.multi -t <container_registry>/service_discovery_multi --push .
    ```

1. Once the container has been built, start the container in interactive mode with the following
command in the project root directory:

    ```shell
    docker run --name <container_name> --network=host -it --rm <image_name>
    ```

    For example, to run the `service_discovery` image built in step 1:

    ```shell
    docker run --name service_discovery --network=host -it --rm service_discovery
    ```

    >Note: A custom network is recommended when using a container for anything but testing.

1. To detach from the container, enter:

    <kbd>Ctrl</kbd> + <kbd>p</kbd>, <kbd>Ctrl</kbd> + <kbd>q</kbd>

1. To stop the container, enter:

    ```shell
    docker stop <container_name>
    ```

    For example, to stop the `service_discovery` container started in step 2:

    ```shell
    docker stop service_discovery
    ```

### Podman Containers

#### Prequisites

[Install Podman](https://podman.io/docs/installation)

#### Running in Podman

To run the service in a Podman container:

1. Run the following command in the project root directory to build the podman container from the
Dockerfile:

    ```shell
    podman build -t <image_name> -f <Dockerfile> .
    ```

    For example, to build an image for the `service_discovery` component:

    ```shell
    podman build -t service_discovery -f Dockerfile.amd64 .
    ```

1. Once the container has been built, start the container with the following command in the project
root directory:

    ```shell
    podman run --network=host <image_name>
    ```

    For example, to run the `service_discovery` image built in step 1:

    ```shell
    podman run --network=host service_discovery
    ```

    >Note: A custom network is recommended when using a container for anything but testing.

1. To stop the container, run:

    ```shell
    podman ps -f ancestor=<image_name> --format="{{.Names}}" | xargs podman stop
    ```

    For example, to stop the `service_discovery` container started in step 2:

    ```shell
    podman ps -f ancestor=localhost/service_discovery:latest --format="{{.Names}}" | xargs podman stop
    ```
