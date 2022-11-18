// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

using System;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Text.Json;
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

    while (Console.ReadLine() is { } line)
    {
        if (!TryParse(line, out var command, out var error))
        {
            Console.WriteLine(error);
            continue;
        }

        switch (command)
        {
            case PingCommand:
            {
                await mqttClient.PingAsync(CancellationToken.None);
                Console.WriteLine("Pong!");
                break;
            }
            case SubscribeCommand { Topic: var topic }:
            {
                var options =
                    mqttFactory.CreateSubscribeOptionsBuilder()
                               .WithTopicFilter(f => f.WithTopic(topic))
                               .Build();
                await mqttClient.SubscribeAsync(options, CancellationToken.None);
                break;
            }
            case PublishCommand { Topic: var topic, Message: var payload }:
            {
                var message =
                    new MqttApplicationMessageBuilder()
                               .WithTopic(topic)
                               .WithPayload(payload)
                               .Build();
                await mqttClient.PublishAsync(message, CancellationToken.None);
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

static bool TryParse(string input,
                     [NotNullWhen(true)] out Command? command,
                     [MaybeNullWhen(true)] out string error)
{
    error = null;
    command = null;

    var scanner = new Scanner(input);
    var commandWord = scanner.ReadWordOrDefault();
    scanner.SkipWhitespace();

    switch (commandWord)
    {
        case "ping":
        {
            command = new PingCommand();
            break;
        }
        case "publish":
        {
            if (!scanner.TryReadWord(out var topic))
            {
                error = "Missing publishing topic!";
                return false;
            }

            scanner.SkipWhitespace();
            if (scanner.ReadAll().TrimEnd() is not { IsEmpty: false } message)
            {
                error = "Missing publishing message!";
                return false;
            }

            command = new PublishCommand(topic.ToString(), message.ToString());
            Console.Error.WriteLine($"Published to \"{topic}\": {message}");
            break;
        }
        case "subscribe":
        {
            if (!scanner.TryReadWord(out var topic))
            {
                error = "Missing subscription topic!";
                return false;
            }
            command = new SubscribeCommand(topic.ToString());
            Console.Error.WriteLine($"Subscribed to \"{topic}\".");
            break;
        }
        default:
        {
            error = "Invalid command!";
            return false;
        }
    }

    scanner.SkipWhitespace();
    if (scanner.ReadAll() is { IsEmpty: false } rest)
    {
        error = $"Unexpected input for command: {rest}";
        return false;
    }

    return true;
}

abstract record Command;
sealed record PingCommand : Command;
sealed record PublishCommand(string Topic, string Message) : Command;
sealed record SubscribeCommand(string Topic) : Command;

ref struct Scanner
{
    ReadOnlySpan<char> _input;

    public Scanner(ReadOnlySpan<char> input) => _input = input;

    public ReadOnlySpan<char> ReadAll()
    {
        var result = _input;
        _input = default;
        return result;
    }

    public void SkipWhitespace() =>
        _input = _input.TrimStart();

    public ReadOnlySpan<char> ReadWordOrDefault() =>
        TryReadWord(out var word) ? word : default;

    public bool TryReadWord(out ReadOnlySpan<char> value) =>
        TryReadUntil(ch => !char.IsWhiteSpace(ch), out value);

    public bool TryReadUntil(Func<char, bool> predicate, out ReadOnlySpan<char> value)
    {
        var i = 0;
        while (i < _input.Length && predicate(_input[i]))
            i++;

        if (i is 0)
        {
            value = default;
            return false;
        }

        value = _input[..i];
        _input = _input[i..];
        return true;
    }
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
