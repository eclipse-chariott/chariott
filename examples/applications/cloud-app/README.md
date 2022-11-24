# Car Bridge Cloud Application

This application is designed to work in the cloud and with the Car Bridge to
communicate with in-car Chariott applications.

## Requirements

The Car Bridge Cloud Application requires the following components running:

- Chariott
- [Eclipse Mosquitto] (MQTT) broker
- Car Bridge (connected to the MQTT broker)

## Usage

The application supports several command-line options:

    Car Bridge Cloud Application

    Usage:
        CarBridgeCloudApp [--broker=<host>] [--vin=<vin>] [--timeout=<sec>] [--pretty-events]
        CarBridgeCloudApp -h | --help

    Options:
        -h --help        Show this screen.
        --broker=<host>  MQTT broker address [default: localhost].
        --vin=<vin>      VIN umber [default: 1].
        --timeout=<sec>  Timeout in seconds [default: 5].
        --pretty-events  Pretty print events JSON.

The VIN number is important. It must be specified with the same value as the
Car Bridge.

## Running

Assuming the current working directory is the repository root, use the
following command to build and run the Car Bridge Cloud Application:

    dotnet run --project examples/applications/cloud-app/src

The Car Bridge Cloud Application takes commands from standard input that can
be used to drive interactivity with the Car Bridge and in-car applications:

    ping
    set vin <vin>
    get vin
    get events.file
    inspect <namespace> <query>
    read <namespace> <key>
    write <namespace> <key> <value>
    invoke <namespace> <command> [<arg>...]
    subscribe <namespace> <source>...
    show topics
    show new events
    show value <value>
    (quit | exit)
    help

When the Car Bridge Cloud Application starts, it connects to the MQTT broker
and starts accepting and executing commands until the standard input closes
(EOF). See the [Commands section](#commands) for details on each command.

The application also creates a topic to receive events, which is used for
subscriptions.

## Commands

### `ping`

    ping

Sends a ping to the MQTT broken and prints `Pong!` on success. This can be
used to test connectivity.

### `set vin`

    set vin <vin>

Sets the VIN for the session to `<vin>`. For example:

    set vin 1234

This should be the same as configured for the Car Bridge.

The VIN number can also be set with the command-line option `--vin` when
launching the application.

### `get vin`

    print vin

Prints the session VIN.

### `get events.file`

    get events.file

Prints the full path to the file where events from subscriptions are being
recorded. This can be useful to run a `tail -f` against the file and monitor
events arriving in near real-time.

### `inspect`

    inspect <namespace> <query>

Requests Chariott (via the Car Bridge) to fulfill the _inspect_ intent by an
in-car application.

Example:

    inspect system.registry **
    inspect sdv.vdt **
    inspect sdv.vdt **.*Temp*

### `read`

    read <namespace> <key>

Requests Chariott (via the Car Bridge) to fulfill the _read_ intent by an
in-car application.

Examples:

    read sdv.vdt Vehicle.Cabin.HVAC.AmbientAirTemperature
    read sdv.vdt Vehicle.Cabin.HVAC.IsAirConditioningActive
    read sdv.kvs Feature.DogMode.Status

### `write`

    write <namespace> <key> <value>

Requests Chariott (via the Car Bridge) to fulfill the _write_ intent by an
in-car application.

Examples:

    write sdv.kvs Feature.DogMode.Status true

See also the [Specifying Values] section for details on how to express
different types of literals for `<value>`.

### `invoke`

    invoke <namespace> <command> [<arg>...]

Requests Chariott (via the Car Bridge) to fulfill the _invoke_ intent by an
in-car application. Multiple arguments (`<arg>...`) for a command are
space-separated. To include space in a, for example, a string argument, use
double-quotes, as in `"foo bar baz"`.

Examples:

    invoke sdv.vdt Vehicle.Cabin.HVAC.IsAirConditioningActive true

See also the [Specifying Values] section for details on how to express
different types of literals for arguments (`<arg>`).

### `subscribe`

    subscribe <namespace> <source>...

Requests Chariott (via the Car Bridge) to fulfill the _invoke_ intent by an
in-car application. Multiple sources (`<source>...`) are space-separated.

To see the events, use the `show new events` command.

Examples:

    subscribe sdv.vdt Vehicle.Cabin.HVAC.AmbientAirTemperature

### `show topics`

    show topics

Prints the MQTT topics being used for RPC (requests and responses) and events.

### `show new events`

    show new events

Prints the new events received since the same command was run.

### `show value`

    show value <value>

Parses `<value>` according to the [Specifying Values] section and prints the
parsed type, followed by colon (`:`) and space, followed by the parsed value.
If the parsing results in an error, then prints `ERROR`.

### `quit` or `exit`

    quit
    exit

Closes the connection with the MQTT broker and ends the application. The same
be done by closing the standard input stream (EOF) from which commands are
read.

### `help`

    help

Prints the commands and their usage.

### Specifying Values

Some commands like [`write`](#write) and [`invoke`](#invoke) allow you to
specify _values_. Car Bridge Cloud Application supports the following literal
syntax different types of values:

- `true` or `false` is treated as a Boolean.

- A sequence of digits is treated as a 32-bit integer.

- A sequence of digits followed by `L` is treated as a 64-bit integer.

- A fractional number, e.g. `1.2`, `.9` or `0.9`, following by `f` or `F` is
  treated like a 32-bit floating-point number.

- A fractional number, e.g. `1.2`, `.9` or `0.9`, is treated like a 64-bit
  floating-point number.

- Anything else is treated like string. If it starts and ends with a single
  quote (`'`) then the quotes are removed. This allows `'true'` and `'false'`
  to be treated like strings instead of Boolean values.

  [Specifying Values]: #specifying-values
  [Eclipse Mosquitto]: https://mosquitto.org/
