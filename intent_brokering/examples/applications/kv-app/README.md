# Key-Value Store Application

This is an example provider that offers the capability to read from
and write to an in-memory key-value store. It also supports subscribing to
changes in the key store where the events are delivered over an opened
channel.

## Testing

Start the Intent Brokering Service followed by this application:

```bash
cargo run -p intent_brokering &
cargo run -p kv-app &
```

Once both are up and running successfully, use the following to write a
key-value pair to the store via the Intent Brokering Service:

```bash
grpcurl -plaintext -d @ 0.0.0.0:4243 intent_brokering.runtime.v1.IntentBrokeringService/Fulfill <<EOF
{
  "namespace": "sdv.kvs",
  "intent": {
    "write": {
      "key": "date-time",
      "value": {
        "string": "$(date)"
      }
    }
  }
}
EOF
```

To read the value of the key written in the above example, run:

```bash
grpcurl -plaintext -d @ 0.0.0.0:4243 intent_brokering.runtime.v1.IntentBrokeringService/Fulfill <<EOF
{
  "namespace": "sdv.kvs",
  "intent": {
    "read": {
      "key": "date-time"
    }
  }
}
EOF
```

To discover the service end-points:

```bash
grpcurl -plaintext -d @ 0.0.0.0:4243 intent_brokering.runtime.v1.IntentBrokeringService/Fulfill <<EOF
{
  "namespace": "sdv.kvs",
  "intent": {
    "discover": {
    }
  }
}
EOF
```

Open a channel for receiving events:

```bash
grpcurl -v -plaintext -import-path proto -proto \
    intent_brokering/proto/intent_brokering/streaming/v1/streaming.proto 0.0.0.0:50064 \
    intent_brokering.streaming.v1.ChannelService/Open | tee events.log &
```

The above command will run in the background and write the events to the file
`events.log` as well as to the standard output.

The channel identifier, which is returned as the response header named
`x-chariott-channel-id`, can be extracted from the file into a variable:

```bash
export CHANNEL_ID=$(grep -E "^x-chariott-channel-id:" events.log \
    | sed -E s/^x-chariott-channel-id:\\s//g)
```

Next, subscribe to a key for events to be delivered to the open channel:

```bash
grpcurl -plaintext -d @ 0.0.0.0:4243 intent_brokering.runtime.v1.IntentBrokeringService/Fulfill <<EOF
{
  "namespace": "sdv.kvs",
  "intent": {
    "subscribe": {
        "channel_id": "$CHANNEL_ID",
        "sources": [
            "date-time"
        ]
    }
  }
}
EOF
```

Finally, writing to the key:

```bash
grpcurl -plaintext -d @ 0.0.0.0:4243 intent_brokering.runtime.v1.IntentBrokeringService/Fulfill <<EOF
{
  "namespace": "sdv.kvs",
  "intent": {
    "write": {
      "key": "date-time",
      "value": {
        "string": "$(date)"
      }
    }
  }
}
EOF
```

should generate a new event.

To clean-up from the above commands, run:

```bash
pkill kv-app
pkill intent_brokering
pkill grpcurl
unset CHANNEL_ID
```
