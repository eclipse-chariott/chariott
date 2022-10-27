# `charc`

`charc` is a Bash script for simple and quick session-based interactions with
Chariott. `charc` stands for _Chariott console_.

## Usage

    Usage:
        charc inspect <namespace> <query>
        charc discover <namespace>
        charc read <namespace> <key>
        charc write <namespace> <key> <value>
        charc invoke <namespace> <command> [(<type> <arg>)...]
        charc listen <namespace>
        charc subscribe <namespace> <source>
        charc events [<namespace>] [-f]
        charc channel <namespace>
        charc show (req|request|rsp|response)
        charc end

## Sub-commands

### `inspect`

    charc inspect <namespace> <query>

Issues the _inspect intent_ to Chariott for fullfilment by some provider
providing the capability.

For inspection of the Chariott registry of providers, use `system.registry`
for the `<namespace>` argument:

    ./charc inspect system.registry "**"

### `discover`

    charc discover <namespace>

Issues the _discover intent_ to Chariott for fullfilment by some provider
providing the capability, e.g.:

    ./charc discover sdv.vdt

It can be used for discovering end-points of services for estabilishing direct
connections.

### `read`

    charc read <namespace> <key>

Issues the _read intent_ to Chariott for fullfilment by some provider
providing the capability, e.g.:

    ./charc read sdv.kvs time

### `write`

    charc write <namespace> <key> <value>

Issues the _write intent_ to Chariott for fullfilment by some provider
providing the capability, e.g.:

    ./charc write sdv.kvs time "$(date)"

### `invoke`

    charc invoke <namespace> <command> [(<type> <arg>)...]

Issues the _invoke intent_ to Chariott for fullfilment by some provider
providing the capability.

The arguments after `<command>` are always in pairs of `<type>` and `<arg>`,
where `<type>` is one of the below and `<arg>` the value of that type:

| Short | Long       | Type           |
|:-----:|:----------:|----------------|
| `-s`  | `--string` | String         |
| `-b`  | `--bool`   | Boolean        |
| `-n`  | `--int32`  | 32-bit integer |

Example:

    ./charc invoke sdv.vdt Vehicle.Cabin.HVAC.IsAirConditioningActive --bool true

### `listen`

    charc listen <namespace>

Establishes a direct streaming channel with a provider offering such a
service, e.g.

    ./charc listen sdv.kvs

Internally, this discovers the streaming gRPC end-point using the equivalent
of the following:

    ./charc discovery sdv.kvs

The end-point must use gRPC (`grpc+proto`) and conform to
`chariott.streaming.v1`.

This sub-command only needs to be issued once per namespace, but supports
concurrently listening to multiple streams. It is required to use
[`subscribe`](#subscribe).

Upon a successful run, it will print the channel identifier of the opened
stream.

If you use this sub-command, you should use the [`end`](#end) sub-command
eventually to close all opened streams.

### `subscribe`

    charc subscribe <namespace> <source>

Issues the _subscribe intent_ to Chariott for fullfilment by some provider
providing the capability, e.g.:

    ./charc subscribe sdv.kvs time

This sub-command requires that [`listen`](#listen) has been executed to
establish a streaming channel for receiving the events of the subscription.

### `events`

    charc events [<namespace>] [-f]

Lists all the events received for subscriptions.

If `-f` (_follow_) is supplied, then it lists the most recent events and
continues to monitor and print any subsequent events until the script is
terminated, e.g. via <kbd>CTRL</kbd>+<kbd>C</kbd>).

Examples:

    ./charc events sdv.kvs      # list events
    ./charc events sdv.kvs -f   # recent + follow

### `channel`

    charc channel <namespace>

Prints the channel identifier if [`listen`](#listen) was previously executed
for `<namespace>` to open a streaming channel.

### `show`

    charc show (req|request|rsp|response)

Shows the last gRPC request (`req`) sent or response (`rsp`) received by the
last sub-command that communicated with Chariott.

### `end`

    charc end

Ends the current session by killing sub-processes, such as those
launched for listening with [`listen`](#listen), and removing persisted
session state (under `~/.charc`).
