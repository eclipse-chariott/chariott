# Event sub-system (ESS)

This crate contains the implementation of the event sub-system (ESS) based on
[ADR-0008](../docs/adr/0008-event-sub-system.md). On a high level, the ESS
connects components that subscribe to a set of events with components publishing
events.

Note: the ESS can also be integrated as an observer with the [key-value
store](../keyvalue/README.md).

For more detailed information, refer to the Rustdocs.
