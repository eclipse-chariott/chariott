# Local object detection application

This code sample shows you an implementation of object detection based on
TensorFlow and ssd_mobilenet_v1_coco model.

To run the application:

1. Start the Intent Brokering runtime by executing `cargo run -p intent_brokering`
2. Start detection application by executing `cargo run` from the
   `intent_brokering/examples/applications/local-object-detection` directory.
3. In the root directory create a `detect_image.json` file with the following
   message structure:

    ```json
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
                                "bytes": "base64 encoding of the image"
                            }
                        }
                    }
                ]
            }
        },
        "namespace": "sdv.detection"
    }
    ```

4. Execute detection with `grpcurl -v -plaintext -import-path proto/ \
   -import-path intent_brokering/examples/applications/proto -use-reflection -proto \
   intent_brokering/examples/applications/proto/examples/detection/v1/detection.proto -d @ \
   localhost:4243 intent_brokering.runtime.v1.IntentBrokeringService/Fulfill < \
   detect_image.json`
