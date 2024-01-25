# Simple Provider Example

This is an example Intent provider that shows how to register a provider with the Intent Brokering service.

## Testing

Start the Intent Brokering service followed by this application:

```bash
cargo run -p intent_brokering &
cargo run -p simple-provider &
```

Once both are up and running successfully, use the following to 'discover'
the provider. This will let you know that the provider is registered with the Intent Brokering service:

```bash
grpcurl -plaintext -d @ 0.0.0.0:4243 intent_brokering.runtime.v1.IntentBrokeringService/Fulfill <<EOF
{
  "namespace": "sdv.simple.provider",
  "intent": {
    "discover": {}
  }
}
EOF
```

To clean-up from the above commands, run:

```bash
pkill simple-provider
pkill intent_brokering
pkill grpcurl
```
