# Simulated camera application

This code sample shows you an implementation of a simulated camera streaming,
which is looping through the files in a `images` folder and streaming those
frames at the following rates: 2/6/12 frames per minute or respectively in
manual mode where you specify the frame rate event yourself.

## To run the application

1. Start the Intent Brokering runtime by executing `cargo run -p intent_brokering` from the root directory
2. Navigate to `intent_brokering/examples/applications/simulated-camera` directory
3. Create an `images` directory and place there all the `.jpg` files you want the
   camera application to stream
4. Start camera application by executing `cargo run` from the
   `intent_brokering/examples/applications/simulated-camera` directory.
5. In another terminal, open a channel to the simulated-camera with `grpcurl -v \
   -plaintext -import-path proto -proto \
   intent_brokering/proto/intent_brokering/streaming/v1/streaming.proto localhost:50066 \
   intent_brokering.streaming.v1.ChannelService/Open` and take a note of the returned
   channel id in the metadata _x-chariott-channel-id_.
6. In yet another terminal, call the following, using the channel id from the
   previous step:

   ```shell
   grpcurl -v -plaintext -d @ localhost:4243 intent_brokering.runtime.v1.IntentBrokeringService/Fulfill <<EOM
   {
       "namespace": "sdv.camera.simulated",
       "intent": {
           "subscribe": {
               "channel_id": "",
               "sources": ["camera.12fpm"]
           }
       }
   }
   EOM
   ```

   This makes a single request for a subscription intent for simulated camera
   frames arriving at the desired frame per minute rate. The Intent Broker then passes
   these intents to the simulated camera application that can fulfill it.

   Other allowed sources (i.e. event rate) can be found via `Inspect` intent:

   ```shell
   grpcurl -v -plaintext -d @ localhost:4243 intent_brokering.runtime.v1.IntentBrokeringService/Fulfill <<EOM
    {
        "namespace": "sdv.camera.simulated",
        "intent": {
            "inspect": {
                "query": "*"
            }
        }
    }
   EOM
   ```

## To run the application in manual mode

You follow the same procedure as in the previous section, but instead of
running step 4 you exchange it with the following command

```shell
cargo run -- -m
```

This will start the application in manual mode, where you can specify the images
through a stdin stream. The syntax for the stream is as follows:

```shell
load <path to image file> <frame rate in frames per minute>
```

For example, to load an image `image.jpg` at 2 frames per minute, you would
send the following to the application:

```shell
load image.jpg camera.2fpm
```

Valid frame rates are `camera.2fpm`, `camera.6fpm` and `camera.12fpm`. The image
path is relative from where you start the application.
