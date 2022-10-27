# Cloud Connectivity

- Status: deprecated (not applicable to Chariott)
- Authors: Patrick Schuler, Dariusz Parys
- Date: 2022-04-27

## Context

The programming model is targeting the so called Head Unit in the car. The modem
for any external communication is located in the communication unit (Com Unit).
We need a way to communicate with the cloud from the Head Unit through the
communication unit. The protocol specific part of the communication, should be
abstracted from the applications.

### Message Based Communication

#### Device to Cloud

For the synchronization of car state (twin values) we are aiming for a message
based approach. We are targeting an MQTT message broker in the cloud. To
facilitate the communication from and to the Head Unit, we suggest a message
broker located in the Com Unit as a gateway to the cloud. The Head Unit is able
to talk to the communication unit directly through MQTT. The Head Unit does
create a connection to the Com Unit, authenticating using a certificate.
Messages are sent to a predefined topic for delivery to the cloud. The Com Unit
then forwards the messages when connectivity is established. Messages indicate a
'type' to allow further routing in the cloud.

The synchronization is split in 3 parts:

1. A service running in the middleware keeping track of changes to the vehicle
   store. The service synchronizes changed values to the cloud on a predefined
   schedule. For example, every 15min.
2. An API is provided to applications to indicate immediate synchronization of
   specific values. This allows to enforce higher frequency values to be
   synchronized more often. A potential rate limiting system could be added in
   the middleware to avoid overloading the cloud side.
3. An API is provided to applications to send other message types to the cloud.

#### Cloud to Device

An MQTT topic is setup in the Com Unit to receive Cloud to Device messages. The
Head Unit subscribes to the Cloud to Device topic and dispatches them to
applications handling a particular message type. Cloud to Device messages have a
time to live (TTL) to avoid delayed message processing for commands that require
a certain minimum latency for processing (opening the door for example). MQTT
5.0 allows to specify expiration in seconds. [MQTT 5.0 - Message Expiry
Interval](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901112).

#### Assumptions - Message Based

1. Head Unit can directly communicate with the Com Unit using MQTT.
2. There is a way to authenticate the Head Unit to the Com Unit and vice versa.
3. There is a way to authorize access to specific topics.

#### Out of scope

1. If applications decide when and what to send to the cloud, or the middleware
   is running a service to handle it, is out of scope of this ADR.
2. Cloud authentication is part of the cloud connectivity stream.

### Synchronous Communication

Synchronous communication requires connectivity for the duration of the call. We
will be focusing on outbound communication as a start to be able to overcome
certain limits (message size for example) with the message based communication.

Two options need to be evaluated:

1. Communication is handled through the middleware (generic endpoint to make
   HTTP requests and get a response back)

Going through the middleware can allow us to limit the traffic coming from a
single IP and trust that traffic in the Com Unit. What application is allowed to
access what address can be enforced in the middleware.

#### Assumptions - Sync

1. Head Unit can directly communicate with the Com Unit using HTTP.
2. There is a way to authenticate the Head Unit to the Com Unit and vice versa.
3. Applications are authorized to access certain URLs.
4. Authorization and authentication to the cloud is handled by cloud
   connectivity stream.

## Decisions

1. Message based communication is established with the Com Unit.
   - The Com Unit  synchronizes the messages with the cloud. The middleware
     provides interfaces to forward messages to the topic for synchronization.
     Applications can send the values they like to have synchronized on a higher
     frequency directly through that interface. A synchronization service in the
     middleware does sync values on a conservative schedule to have eventual
     consistency in the cloud, keeping track of changes and only sync the
     changes up to the cloud.
   - The cloud can send messages to the car (C2D). To ensure the message can be
     delivered in offline scenarios, we provide a queue per device in the cloud
     that is synchronized to the car when connectivity allows to. C2D messages
     can employ a TTL to expire the delivery.

2. Synchronous communication is provided through the middleware. The middleware
   is aware through ACL if an application is allowed to access a specific URL.
