# Mock Vehicle Abstraction Service (VAS)

This code sample shows you how to use the mocked VAS for the dog mode scenario.
The dog mode allows a car owner to keep their dog safe, while they are away from
the car.

## How-to consume a streaming service using Chariott Intent Broker

> As an application developer, I want to consume events from a streaming
> service.

The example application [dog-mode-logic](../dog-mode-logic/) showcases how to
achieve this using a Rust gRPC client. In this section we show how to do this
using [gRPCurl](https://github.com/fullstorydev/grpcurl) calls from the command line.

From the root directory:

1. In a terminal (A) start Intent Brokering with `cargo run -p intent_brokering`.
2. In another terminal (B) start the mock-vas with `cargo run -p mock-vas`.
3. In a terminal (C), open a channel to the mock-vas with `grpcurl -v -plaintext \
   -import-path proto -proto proto/intent_brokering/streaming/v1/streaming.proto \
   localhost:50051 intent_brokering.streaming.v1.ChannelService/Open` and take a note of
   the returned channel id in the metadata _x-chariott-channel-id_.
4. In another terminal D call the following, using the channel id from the
   previous step:

   ```shell
   grpcurl -v -plaintext -d @ localhost:4243 intent_brokering.runtime.v1.IntentBrokeringService/Fulfill <<EOM
   {
       "namespace": "sdv.vdt",
       "intent": {
           "subscribe": {
               "channel_id": "",
               "sources": ["Vehicle.Cabin.HVAC.AmbientAirTemperature"]
           }
       }
   }
   EOM
   ```

   This makes a single request for 2 subscription intents for the
   dog mode status and the temperature events to the Intent Broker. The Intent Broker then passes these
   intents to the mock-vas that can fulfill them.
5. In terminal B, type `temp 20` in the standard input and notice the response
   coming through terminal C. However, if you type `battery 10` you see that
   this is not coming through as we did not subscribe to battery updates.

## How-to implement a streaming service

> As a provider developer, I want to create a streaming service for events.

In order to do so, you need to:

- Implement the [streaming proto](../../../proto/intent_brokering/streaming/v1/streaming.proto)
  and specifically the `OpenRequest` endpoint with a service.
  - This is done in the common examples library in [streaming.rs](../../common/src/intent_brokering/streaming.rs)
  - Make sure to serve this service with your gRPC server.
- The application will send `SubscribeIntent` that your service would need to
  handle.
  - In order to create the required client and register the subscriptions, you
    can use the [Event Sub System crate aka ESS crate](../../../ess/).
  - This is done in mock-vas in [intent_provider.rs](./src/intent_provider.rs)
