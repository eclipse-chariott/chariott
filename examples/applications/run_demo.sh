#!/usr/bin/env bash
# Copyright (c) Microsoft Corporation. All rights reserved.
# Licensed under the MIT license.

set -e
cd "$(dirname "$0")/../.."

if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    echo 'Run Chariott demo.

This script allows you to specify the following parameters:

- Cognitive services endpoint (--cognitive_endpoint)
- Cognitive services key (--cognitive_key)

If you specify a wrong endpoint/key, the demo will still run but use local object detection instead.
'
    exit 1
fi

cognitive_endpoint=""
cognitive_key=""

while [ $# -gt 0 ]; do

   if [[ $1 == *"--"* ]]; then
        param="${1/--/}"
        declare "$param"="$2"
   fi

  shift
done

trap cleanup SIGINT

cleanup()
{
    echo>&2 "Stopping applications..."
    pkill chariott || true
    pkill kv-app || true
    pkill dog-mode-logic-app || true
    pkill DogModeDashboard || true
    if [[ ! -z "$CLOUD_DETECTION_PID" ]]; then
        kill $CLOUD_DETECTION_PID || true
    fi
    kill $LOCAL_DETECTION_PID || true
    kill $MOCK_VAS_PID || true
    kill $CAMERA_PID || true
    exit 1
}

if [ ! -d target/logs ]; then
    mkdir -p target/logs
fi

cargo build --workspace

./target/debug/chariott > target/logs/chariott.txt 2>&1 &

sleep 2

./examples/applications/dog-mode-ui/mock_provider_dog_mode_demo.sh | ANNOUNCE_URL=http://localhost:50051 ./target/debug/mock-vas > target/logs/mock_vas.txt 2>&1 &
MOCK_VAS_PID=$!
ANNOUNCE_URL=http://localhost:50064 ./target/debug/kv-app > target/logs/kv_app.txt 2>&1 &
SIMULATED_CAMERA_APP_IMAGES_DIRECTORY=./examples/applications/simulated-camera/images ANNOUNCE_URL=http://localhost:50066 ./target/debug/simulated-camera-app > target/logs/simulated_camera_app.txt 2>&1 &
CAMERA_PID=$!
TENSORFLOW_LIB_PATH="$(dirname $(find target -name libtensorflow.so -printf '%T@\t%p\n' | sort -nr | cut -f 2- | head -1))"
LIBRARY_PATH=$TENSORFLOW_LIB_PATH LD_LIBRARY_PATH=$TENSORFLOW_LIB_PATH CATEGORIES_FILE_PATH=./examples/applications/local-object-detection/models/categories.json ANNOUNCE_URL=http://localhost:50061  ./target/debug/local-object-detection-app > target/logs/local_object_detection_app.txt 2>&1 &
LOCAL_DETECTION_PID=$!
if [[ ! -z "$cognitive_endpoint" || ! -z "$cognitive_key" ]]; then
    COGNITIVE_ENDPOINT=$cognitive_endpoint COGNITIVE_KEY=$cognitive_key ANNOUNCE_URL=http://localhost:50063 ./target/debug/cloud-object-detection-app > target/logs/cloud_object_detection_app.txt 2>&1 &
    CLOUD_DETECTION_PID=$!
else
    echo "Did not start cloud object detection application. Specify 'cognitive_endpoint' and 'cognitive_key' to start it."
fi

sleep 5

./target/debug/dog-mode-logic-app > target/logs/dog_mode_logic_app.txt 2>&1 &

sleep 2

dotnet run --project examples/applications/dog-mode-ui/src > ./target/logs/ui.txt 2>&1 &

wait
