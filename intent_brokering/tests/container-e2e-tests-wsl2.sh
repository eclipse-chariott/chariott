#!/bin/bash
# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

# This script is used to run the container e2e tests on WSL2
# inside of the containers. It will provision a network and
# run the containers inside this new bridged network.

set -e

# set up error handler to clean up docker containers and network
function cleanup {
    echo "Cleaning up containers and network"
    docker rm -f intent_brokering 2>/dev/null
    docker rm -f kv-app 2>/dev/null
}

trap cleanup EXIT

# first parameter is required
if [ -z "$1" ]; then
    echo "The first parameter must be the Dockerfile context"
    exit 1
fi
DOCKERFILE_CONTEXT=$1

# check for optional parameter TAG (tag of the images to pull) and set default to 1 if not provided
if [ -z "$TAG" ]; then
    export TAG="1"
fi

# clean up any existing containers and networks
if cleanup; then
    echo "Cleaned up existing containers and network"
fi

# build intent_brokering docker container
docker build --tag "intent_brokering:$TAG" --file "$DOCKERFILE_CONTEXT/Dockerfile" "$DOCKERFILE_CONTEXT"
docker run --init --rm --name intent_brokering --detach --publish 4243:4243 "intent_brokering:$TAG"

# build kv-app docker container
docker build --tag "kv-app:$TAG" --file "$DOCKERFILE_CONTEXT/examples/applications/Dockerfile.kv-app.ci" --build-arg APP_NAME=kv-app "$DOCKERFILE_CONTEXT"
docker run --init --rm --name kv-app --detach --publish 50064:50064 --env ANNOUNCE_URL=http://host.docker.internal:50064 --env INTENT_BROKER_URL=http://host.docker.internal:4243 "kv-app:$TAG" # DevSkim: ignore DS137138

# run the end to end tests
cargo test --test "*e2e"

# No need to stop containers here as the cleanup trap will be called

