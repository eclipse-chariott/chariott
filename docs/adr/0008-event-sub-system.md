# Event Sub-system (ESS)

- Status: superseded by [adr-0022](./0022-event-sub-system.md)

## Context and Problem Statement

Applications need a way to subscribe to the car telemetry and changes in the
state of the car via the middleware. The providers publish the data to the
middleware, which in turn stores the data in the V-Store. When data being
watched changes, the subscribing applications need to be notified via events.
The part of the middleware that manages the subscriptions and the delivery of
data events to the interested applications is what constitutes the Event
Sub-system (ESS).

The ESS should deliver _data-centric_ events; put another way, it should
deliver telemetry and state as _data events_. Instead of delivering distinct
events like _window-opened_ or _window-closed_, the events simply convey the
_window-level_, where for example, 100 could mean fully up, 0 could mean
fully down, and any value in between would indicate how much of the window is
up.

The ESS needs to address the following problems:

- How will applications express a data event subscription?
- How and where will the subscriptions be stored?
- How will the middleware deliver the data to the application based on
  potentially some subscription criteria?
- How to ensure only authorized applications can subscribe to the requested
  events?
- What happens to the subscriptions when an application crashes?
- What happens to the subscriptions when the middleware crashes?

## Decision Drivers

1. Performance
2. Efficiency
3. Persistence
4. Integrity

## Decisions

All decisions expressed concern the CTP milestone unless noted otherwise.

The ESS will live together with the V-Store in the same process as the
middleware for sake of simplicity and performance.

The set of keys in V-Store can change over time. The V-Store will expose the
following events for the ESS. The V-Store will fire:

- `key_added` when a new key is added where the event data is the added key.
- `key_removed` when an existing key is removed where the event data is the
  removed key.
- `data_updated` when the data for a key has been updated/changed where the
  event data is the key whose data was updated.

The ESS will broadcast the first two events to all applications that have the
permission for the keys and broker `data_updated` events (last kind) to the
interested applications only. If no application is interested in the
`data_updated` event of a specific key then the ESS will stop further
processing. To make this decision quick and efficient, the ESS will maintain a
_watched key set_ to determine whether any application is interested in
changes to a key or not. If at least one application is interested then the
ESS will ask the V-Store for the value of the key. This avoids cloning the
data unless it needs to be sent to some application. Cloning should also be
avoided if the data was updated but not changed. The _watched key set_ will be
updated by the ESS as applications register and deregister subscriptions for
events.

Keys can get potentially long, e.g. the adjusted position of one car seat
could be named `car.body.windows.row2.right.level`. This means that the event
data will often be smaller in size than the key length. Ideally, the keys will
be atomized as integers such that an event is just the integral key identifier
followed by the event data, but this requires a global key management system
in the middleware to assign unique numbers (even if monotonically increasing
and potentially recyclable) to keys. If such a system exists then the _watched
key set_ in the ESS could be implemented as a variable-length bitmap. The
V-Store will also expose the integral identifier with each `key_added` event.
See also “[A possible optimization]” in the ADR “[Provider-Middleware
interactions]”.

[A possible optimization]: 0006-provider-middleware-interface.md#a-possible-optimization
[Provider-Middleware interactions]: 0006-provider-middleware-interface.md

The ESS will maintain an in-memory map of subscriptions by key. It will be
used to determine which subscriptions and applications are interested in the
changes to the values of V-Store keys. The key of the map will be the V-Store
key and the value a list of subscriptions (in no defined order).

The ESS will maintain 3 queues per application (created as needed) where each
will queue will have a fixed set of characteristics like the maximum queue
length. The characteristics are fixed so applications cannot have undesired or
unbounded effects on the middleware's resource consumption. An application can
decide which queue to use for a particular event subscription. When a queue is
full due to back-pressure, the oldest `data_updated` event for a key is
dropped, thus keeping the latest change event. If there is no previous event
to drop then the new event is never queued and the update is effectively lost.
The queues shall be numbered 0, 1 and 2 (which could be expanded in the
future). An application can optionally configure the desired minimum and the
maximum length of each queue in terms of number events. The ESS will cap the
maximum length to 200.

The ESS will maintain an _event sequence number_ (starting with 1) per key,
per application, that is updated whenever a `data_updated` event for the key
is queued. The sequence number can be used by the application, if needed, to
detect a gap in events. Following is an example of what a `data_updated` event
could look like in Rust:

```rust
pub struct DataUpdatedEvent {
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
will finally update its maps to record the subscription, including the
_watched key set_.

When an application wishes to subscribe to events, it will invoke `register`
and provide the list of desired event subscriptions. The ESS will return a
list as a reponse indicating which subscriptions succeeded based on whether
the application was authorized (always the case for CTP). Registering for a
previously registered subscription overrides that subscription. To deregister,
an application does the same with `deregister`, and like registration,
multiple deregistrations of the same subscription has no effect. During
deregistration, the ESS will update its maps to remove the subscription record
and refresh the _watched key set_.

At the time of registration, the application can indicate whether the current
data of a key is desired. If so, the ESS will return the current data for the
key in the V-Store in the `register` response. This permits the application to
make a bulk query to initialize its state. The application can also indicate,
per key, whether it is interested in events when the key data changes or is
simply updated (with either the same or a different data). An event on update
is interesting for critical events where while the data may be unchanged, the
application must ensure that it is still current. Imagine that an application
must be sure that the cabin temperature remains constant over some time. If
the ESS only emitted events on temperature changes then the application could
not guarantee that data from the cabin temperature sensor is still arriving
into the V-Store.

The ESS delivers events as they happen. The registrations do not express any
additional criteria about the conditions under which the event is generated,
such as change frequency or interval. While this might change in the future,
the current design of the V-Store limits criteria based on the value of the
event data, such as firing an event only when a value is in a range. This is
because the data is a byte array that is entirely opaque to the ESS.

If the middleware crashes, the application must re-register for event
subscriptions whenever the middleware is available again and the connection
restored.

If an application crashes, the ESS should release all resources associated
with the application and remove all records of subscription linked to the
application. The crash detection is done outside of the ESS. For example, the
middleware could monitor application processes or use a heartbeat mechanism
and notify the ESS when an application terminates abnormally.

The ESS does not guarantee that a malicious application cannot mask, hijack or
alter events or their data between the source (an application or a provider)
and the sinks (the applications). It is expected that the middleware or the
V-Store provide this guarantee through some grant system where, for example,
keys are owned or allowed for writing by one or more applications.
