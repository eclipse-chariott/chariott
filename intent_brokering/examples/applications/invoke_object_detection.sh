#!/usr/bin/env bash
# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

set -e
cd "$(dirname "$(readlink -f "$0")")"

if [ $# -eq 0 ] && [ ! -t 0 ]; then
    set -- "${1:-$(base64 -w0 </dev/stdin)}"
else
    echo>&2 -n "Object detection invocation script

In order to use this script, you must pass, via stdin, the bytes of the image to use for object detection.

    curl -sSL https://image_url --output - | $0
    cat image_path | $0
"
    exit 1
fi


DETECTION_NAMESPACE="sdv.detection"
if [ "$(../../intent_brokering/tools/charc inspect system.registry $DETECTION_NAMESPACE | jq '. | length')" -eq 0 ]
then
    echo>&2 "No providers registered for $DETECTION_NAMESPACE."
    exit 1
fi

REQ="$(cat <<EOF
    {
        "intent": {
            "invoke": {
                "command": "detect",
                "args": [
                    {
                        "any": {
                            "@type": "examples.detection.v1.DetectRequest",
                            "blob": {
                                "media_type": "image/jpg",
                                "bytes": "$1"
                            }
                        }
                    }
                ]
            }
        },
        "namespace": "$DETECTION_NAMESPACE"
    }
EOF
)"

grpcurl -plaintext --import-path ../../proto/ -import-path ./proto \
        -use-reflection -proto ./proto/examples/detection/v1/detection.proto \
        -d @ "${INTENT_BROKER_ADDRESS:-0.0.0.0:4243}" \
        intent_brokering.runtime.v1.IntentBrokeringService/Fulfill < <(echo "$REQ") \
        | jq .fulfillment.invoke.return.any.entries
