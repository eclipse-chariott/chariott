# Invoke Command Example

This is an example provider that shows how the invoke intent works.
It shows an example of a command that takes a json string as an input.

## Testing

Start Chariott followed by this application:

```bash
cargo run -p chariott &
cargo run -p invoke-command &
```

Once both are up and running successfully, use the following to 'discover'
the provider. This will let you know that the provider is registered with Chariott:

```bash
grpcurl -plaintext -d @ 0.0.0.0:4243 chariott.runtime.v1.ChariottService/Fulfill <<EOF
{
  "namespace": "sdv.invoke.controller",
  "intent": {
    "discover": {}
  }
}
EOF
```

Once the service is confirmed registered, the below command can be run to call a command
on the provider. The command takes a json string and parses it/prints it out on the provider side.
The 'command' parameter is used to tell the desired provider what command to run.The 'args'
parameter is a list of arguments for the command. The args can be several types, defined
in `proto\chariott\common\v1\common.proto` file under the `Value` message.

```bash
grpcurl -plaintext -d @ 0.0.0.0:4243 chariott.runtime.v1.ChariottService/Fulfill <<EOF
{
  "namespace": "sdv.invoke.controller",
  "intent": {
    "invoke": {
      "command": "parse_and_print_json",
      "args": [
        {"string": "{\"age\":43,\"name\":\"John Doe\",\"phones\":[\"+44 1234567\",\"+44 2345678\"]}"}
      ]
    }
  }
}
EOF
```

To clean-up from the above commands, run:

```bash
pkill invoke-command
pkill chariott
pkill grpcurl
```
