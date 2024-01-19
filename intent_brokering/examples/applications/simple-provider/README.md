# Simple Provider Example

This is an example Chariott provider that shows how to register a provider with chariott.

## Testing

Start Chariott followed by this application:

```bash
cargo run -p chariott &
cargo run -p simple-provider &
```

Once both are up and running successfully, use the following to 'discover'
the provider. This will let you know that the provider is registered with Chariott:

```bash
grpcurl -plaintext -d @ 0.0.0.0:4243 chariott.runtime.v1.ChariottService/Fulfill <<EOF
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
pkill chariott
pkill grpcurl
```
