# Registration and Service Discovery Interfaces

- Status: accepted
- Authors: Atif Aziz, Patrick Schuler
- Last updated: 2022-08-15

## Context and Problem Statement

Applications need to be able to make run-time decisions based on availability
of hardware, options and devices in a vehicle. Services like the Vehicle
Abstraction Service (VAS) need to be able to offer means for applications to
search, list and inspect objects, like doors, windows, HVAC, etc., that make
up the Vehicle Digital Twin (VDT).

The VAS offering inspection was the primary scenario considered, but options
were explored in light of more general application. For example, while the
hardware in a vehicle is expected to be exposed as objects with commands and
properties, it should be possible for any application/service to offer
inspection of arbitrary, but uniquely identifiable, items.

## Considered Options

### Request: All, no Filter

The fulfillment returns all results from the inspection without any query
specification that filters.

#### Pros

- Providers do not have to implement a query engine.

#### Cons

- Overly simplistic.
- Can return excessive results, especially in the case of VDT.
- Every client has to do their own filtering.

### Request: Query

The inspection intent will specify a query (in a standardized language) that
will return the matching subset.

#### Pros

- Universal query language for all client applications.
- Only the items matching the query are retured.

#### Cons

- Each provider has to implement a query engine, even if the query
  specification is the same across all of them.
- Query language needs to remain simple to avoid burdening the provider
  implementations.

### Response: Dedicated Messages for Commands and Properties

Dedicated protobuf messages for representing commands and properties as
response values.

#### Pros

- Easy to understand and work with due to them being defined as protobuf
  messages rather than being more loosely-defined with schemas being capture
  in documentation.

#### Cons

- Not general-purpose enough to support other scenarios through the `inspect`
  intent; we would have to adjust the messages to represnt schemas of things
  others than properties and commands.

#### Response: Detailed Responses for Command Arguments & Return

As part of the dedicated protobuf messages discussion, we considered detailed
responses to describe interface contracts (return types as well as arguments).
This would allow to discover required input for a command and execute it
through the invoke intent. However, we believe applications will know the
contract at compile-time, so we only need a unique `type` identifier on the
command and assume an explicit contract.

### Key-Value Pairs with Unique Keys

Simple flat list of key-value pairs. The key is always prefixed with a entry
from the underlying store (like the VDT). For example:

    vdt.cabin.hvac.temperature.member_type: property
    vdt.cabin.hvac.temperature.type: int32

#### Pros

- Simple structure.
- Easily portable to other use cases.

#### Cons

- Verbose due to long and repeating prefixes.
- String tokenization may be required to recognize parts, e.g. to split
  `vdt.cabin.hvac.temperature.type` into `vdt.cabin.hvac.temperature` and
  `type` for further processing.

### List of Key-Value Pairs per Parent

Every entry in the list is a key-value pair representing a match for the query
where the full path is repeated just once for the parent node. The keys are
known for the particular namespace (like `vdt`):

    [
        {
            "path": "vdt.cabin.doors.door2.lock",
            "member_type": "command",
            "type": "IAcmeDoor:Lock"
        },
        {
            "path": "vdt.cabin.hvac.temperature",
            "member_type": "property",
            "type:" "int32",
            "unit": "celsius",
            "read": true
        }
    ]

In the above example:

- A door lock has a command whose interface/signature is determined by a
  unique and opaque type identifier.
- A sensor exposes a property to read a temperature as a 32-bit integer value,
  where the value is expressd in celsius units.

#### Pros

- Simple structure.
- Easily portable to other use cases.
- Known keys can be used in the application.
- All values belonging to an entry are logically grouped together.

#### Cons

- Developers need to know the provided keys (no strong typing)

## Out of scope

- Access control - We do not filter any VDT entries based on current user tokens

## Decision Outcome

The inspect intent message will be as follows:

```proto
message InspectIntent {
    string query = 1;
}
```

We decided to take a simple approach for expressing the _inspection query_, as
the `query` string field shown above. It is assumed that services supporting
inspection will generally and logically organise items into a hierarchical
tree where parent nodes act as containers of child nodes. A period/dot (`.`)
will be used to separate node identifiers, as in `vdt.cabin.hvac.temperature`.
A query can use wildcards to match against several items depending on the
 wildcards and their placement in the query. A `*` can be used to match
against any part of an item identifier. A `**` can be used to match against
nodes at abitrary depth (for recursive inspection), such that
`vdt.**.temperature` will match `vdt.cabin.hvac.temperature`. It is also
possible to mix `*` and `**`, as in `vdt.**.door*.*`.

We explored several options with respect to the inspection fulfillment
reponse. They are described in the next section along with their pros and
cons.

The [List of Key-Value Pairs per Parent](#list-of-key-value-pairs-per-parent)
is the preferred option for the fulfillment result. It is flexible enough to
be used by different use cases and fits the bill for the VDT inspection we
require to support.
