#!/bin/bash
# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

set -e

function concat_image_registry() {
    if [ -z "$IMAGE_REGISTRY" ]; then
        echo "$1"
    else
        echo "$IMAGE_REGISTRY/$1"
    fi
}

if [ -z "$IMAGE_TAG" ]; then
    export IMAGE_TAG="1"
fi

# Build base image for all example applications
docker build --tag "intent_brokering_examples:base" --file ./intent_brokering/examples/applications/Dockerfile.base .

# Build Intent Brokering service
docker build --tag "$(concat_image_registry intent_brokering:"$IMAGE_TAG")" --file ./intent_brokering/examples/applications/Dockerfile.generic --build-arg APP_NAME=intent_brokering .

# Build Examples
docker build --tag "$(concat_image_registry cloud-object-detection-app:"$IMAGE_TAG")" --file ./intent_brokering/examples/applications/Dockerfile.cloud-object-detection --build-arg APP_NAME=cloud-object-detection-app .
docker build --tag "$(concat_image_registry dog-mode-logic-app:"$IMAGE_TAG")" --file ./intent_brokering/examples/applications/Dockerfile.generic --build-arg APP_NAME=dog-mode-logic-app .
docker build --tag "$(concat_image_registry kv-app:"$IMAGE_TAG")" --file ./intent_brokering/examples/applications/Dockerfile.generic --build-arg APP_NAME=kv-app .
# Local object detection build is not executed as it is currently not working due to missing tensorflow libraries in the image
# docker build --tag "$(concat_image_registry local-object-detection-app:"$IMAGE_TAG")" --file ./intent_brokering/examples/applications/Dockerfile --build-arg APP_NAME=local-object-detection-app .
docker build --tag "$(concat_image_registry mock-vas:"$IMAGE_TAG")" --file ./intent_brokering/examples/applications/Dockerfile.mock-vas --build-arg APP_NAME=mock-vas .
docker build --tag "$(concat_image_registry simulated-camera-app:"$IMAGE_TAG")" --file ./intent_brokering/examples/applications/Dockerfile.simulated-camera --build-arg APP_NAME=simulated-camera-app .
docker build --tag "$(concat_image_registry lt-provider-app:"$IMAGE_TAG")" --file ./intent_brokering/examples/applications/Dockerfile.generic --build-arg APP_NAME=lt-provider-app .
