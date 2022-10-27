# Event Sub-system (ESS)

- Status: extended by [ADR-0016](./0016-streaming-support.md)

## Context and Problem Statement

Applications need a way to subscribe to data changes from providers. When data
being watched changes, the subscribing applications need to be notified via
events. The reusable component that manages the subscriptions and the delivery
of data events to the interested applications is what constitutes the Event
Sub-system (ESS). One use case for this is the Vehicle Abstraction Service
(VAS), where applications want to observe changes to the vehicle properties. We
will use the VAS as an example for a provider using the ESS throughout this ADR.

The ESS should deliver _data-centric_ events; put another way, it should
deliver telemetry and state as _data events_. Instead of delivering distinct
events like _window-opened_ or _window-closed_, the events simply convey the
_window-level_, where for example, 100 could mean fully up, 0 could mean
fully down, and any value in between would indicate how much of the window is
up.

The ESS needs to address the following problems:

- How will applications express a data event subscription?
- How and where will the subscriptions be stored?
- How will the provider deliver the data to the application based on potentially
  some subscription criteria?
- How to ensure only authorized applications can subscribe to the requested
  events?
- What happens to the subscriptions when an application crashes?
- What happens to the subscriptions when the provider crashes?

## Decision Drivers

1. Performance
2. Efficiency
3. Persistence
4. Integrity

## Decisions

All decisions expressed concern the CTP milestone unless noted otherwise.

The set of data keys can change over time, and keys can get potentially long,
e.g. the adjusted position of one car seat could be named
`car.body.windows.row2.right.level`. This means that the event data will often
be smaller in size than the key length. Ideally, the keys will be atomized as
integers such that an event is just the integral key identifier followed by the
event data, but this requires a global key management system in the ESS to
assign unique numbers (even if monotonically increasing and potentially
recyclable) to keys.

The ESS will maintain an in-memory map of subscriptions by key. It will be used
to determine which subscriptions and applications are interested in the changes
to the values associated with a key.

The ESS can maintain multiple queues per application (created as needed) where
each will queue will have a fixed set of characteristics like the maximum queue
length. The characteristics are fixed so applications cannot have undesired or
unbounded effects on the provider's resource consumption. An application can
decide which queue to use for a particular event subscription. When a queue is
full due to back-pressure, the oldest event for a key is dropped, thus keeping
the latest change event. If there is no previous event to drop then the new
event is never queued and the update is effectively lost. An application can
optionally configure the desired minimum and the maximum length of each queue in
terms of number events. The ESS will cap the maximum length to 200.

The ESS will maintain an _event sequence number_ (starting with 1) per key, per
application, that is updated whenever an event for the key is queued. The
sequence number can be used by the application, if needed, to detect a gap in
events. Following is an example of what an event could look like in Rust:

```rust
pub struct Event {
  key: u32,                   // supposing atomized key
  seq: u32,                   // update event sequence number
  time: time::SystemTime,     // time of event
  data: Option<std::Vec<u8>>; // key data if changed
}
```

The `time` field permits an application to determine the interval between two
events, whether for the same or a different key. When events are being
dropped, the time gap can serve as a critical information for an application.

The ESS will expose an event service endpoint to applications using which
applications can send the request:

1. `get_events` to open a stream over which events will be delivered.
2. `register` to register an event subscription.
3. `deregister` to deregister a previous event subscrpition.
4. `get_subscriptions` current subscriptions (of the requesting application).

An application must start with `get_events` before registering for event
subscriptions. The ESS will create a queue (based on the desired minimum and
maximum length arguments of the request) for the application where the events
will be delivered. It will then allocate a _logical thread_ that listens and
delivers those events to the application via, for example, a gRPC stream. It
will finally update its maps to record the subscription.

When an application wishes to subscribe to events, it will invoke `register` and
provide the list of desired event subscriptions. The ESS will return a list as a
reponse indicating which subscriptions succeeded based on whether the
application was authorized (always the case for CTP). Registering for a
previously registered subscription overrides that subscription. To deregister,
an application does the same with `deregister`, and like registration, multiple
deregistrations of the same subscription has no effect. During deregistration,
the ESS will update its maps to remove the subscription record.

At the time of registration, the application can indicate whether the current
data of a key is desired. If so, the ESS will return the current data for the
key in the `register` response. This permits the application to make a bulk
query to initialize its state. The application can also indicate, per key,
whether it is interested in events when the key data changes or is simply
updated (with either the same or a different data). An event on update is
interesting for critical events where while the data may be unchanged, the
application must ensure that it is still current. Imagine that an application
must be sure that the cabin temperature remains constant over some time. If the
ESS only emitted events on temperature changes then the application could not
guarantee that data from the cabin temperature sensor is still being refreshed
in the vehicle hardware.

The ESS delivers events as they happen. The registrations do not express any
additional criteria about the conditions under which the event is generated,
such as change frequency or interval.

If the provider crashes, the application must re-register for event
subscriptions whenever the provider is available again and the connection
restored.

If an application crashes, the ESS should release all resources associated with
the application and remove all records of subscription linked to the
application.

The ESS does not guarantee that a malicious application cannot mask, hijack or
alter events or their data between the source (an application or a provider) and
the sinks (the applications).
