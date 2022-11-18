# Car Bridge Cloud Application

This application is designed to work in the cloud and with the Car Bridge to
communicate with in-car Chariott applications.

In its current state, it represents a simple scaffolding of the application.

1. Start an [Eclipse Mosquitto] broker.

2. Start this application using:

       dotnet run --project src

Once the application is running, you can issue the following commands:

    ping
    subscribe TOPIC
    publish TOPIC PAYLOAD
    help | ?
    quit

  [Eclipse Mosquitto]: https://mosquitto.org/
