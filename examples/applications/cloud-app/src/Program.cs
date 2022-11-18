// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

using System;
using System.ComponentModel.Design;
using System.IO;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using MQTTnet;
using MQTTnet.Client;
using MQTTnet.Formatter;

try
{
    var mqttFactory = new MqttFactory();

    using var mqttClient = mqttFactory.CreateMqttClient();

    var mqttClientOptions =
        new MqttClientOptionsBuilder()
            .WithTcpServer(args is [var server, ..] ? server : "localhost")
            .WithProtocolVersion(MqttProtocolVersion.V500)
            .Build();

    await mqttClient.ConnectAsync(mqttClientOptions, CancellationToken.None);
    Console.Error.WriteLine("The MQTT client is connected.");

    mqttClient.ApplicationMessageReceivedAsync += args =>
    {
        args.Dump();
        return Task.CompletedTask;
    };

    var quit = false;
    while (!quit && Console.ReadLine() is { } rawLine)
    {
        const string help = """
            ping
            subscribe TOPIC
            publish TOPIC PAYLOAD
            help | ?
            quit
            """;

        switch (rawLine.Trim())
        {
            case "ping":
            {
                await mqttClient.PingAsync(CancellationToken.None);
                Console.WriteLine("Pong!");
                break;
            }
            case "help" or "?":
            {
                Console.WriteLine(help);
                break;
            }
            case "quit":
            {
                quit = true;
                break;
            }
            case var line when Regex.Match(line, @"^subscribe\s+([^\s]+)$") is { Success: true, Groups: [ _, { Value: var st } ] }:
            {
                var options =
                    mqttFactory.CreateSubscribeOptionsBuilder()
                               .WithTopicFilter(f => f.WithTopic(st))
                               .Build();
                await mqttClient.SubscribeAsync(options, CancellationToken.None);
                break;
            }
            case var line when Regex.Match(line, @"^publish\s+([^\s]+)\s+(.+)$") is { Success: true, Groups: [ _, { Value: var pt }, { Value: var payload } ] }:
            {
                var message =
                    new MqttApplicationMessageBuilder()
                        .WithTopic(pt)
                        .WithPayload(payload)
                        .Build();
                await mqttClient.PublishAsync(message, CancellationToken.None);
                break;
            }
            default:
            {
                Console.Error.WriteLine($"Invalid command! Try one of the following:{Environment.NewLine}{help}");
                break;
            }
        }
    }

    await mqttClient.DisconnectAsync(mqttFactory.CreateClientDisconnectOptionsBuilder().Build(), CancellationToken.None);

    return 0;
}
catch (Exception ex)
{
    Console.Error.WriteLine(ex);
    return 1;
}

static class ObjectExtensions
{
    public static T Dump<T>(this T value, TextWriter? output = null)
    {
        var json = JsonSerializer.Serialize(value, new JsonSerializerOptions { WriteIndented = true });
        output ??= Console.Error;
        output.WriteLine($"[{value?.GetType().Name}]:{Environment.NewLine}{json}");
        return value;
    }
}
