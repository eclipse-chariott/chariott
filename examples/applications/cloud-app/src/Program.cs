// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using Chariott.Common.V1;
using Chariott.Runtime.V1;
using DocoptNet;
using Google.Protobuf;
using MQTTnet;
using MQTTnet.Client;
using MQTTnet.Formatter;
using MoreEnumerable = MoreLinq.MoreEnumerable;
using static MoreLinq.Extensions.RepeatExtension;
using static MoreLinq.Extensions.EvaluateExtension;
using System.Text;

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

    var correlations = MoreEnumerable.Return(() => Guid.NewGuid().ToByteArray())
                                     .Repeat()
                                     .Evaluate();

    var rpcSession = await
        ChariottRpcSession.CreateAsync(mqttFactory, mqttClient,
                                       new Vin("1"), correlations,
                                       CancellationToken.None);

    var jsonSerializerOptions = new JsonSerializerOptions { WriteIndented = true };

    var quit = false;
    while (!quit && Console.ReadLine() is { } line)
    {
        FulfillRequest? request = null;
        switch (Prompt.CreateParser().Parse(line.Split(' ', StringSplitOptions.TrimEntries | StringSplitOptions.RemoveEmptyEntries)))
        {
            case IArgumentsResult<Prompt> { Arguments: var prompt }:
            {
                switch (prompt)
                {
                    case { CmdQuit: true } or { CmdExit: true }:
                    {
                        quit = true;
                        break;
                    }
                    case { CmdHelp: true }:
                    {
                        Prompt.PrintUsage(Console.Out);
                        break;
                    }
                    case { CmdPing: true }:
                    {
                        await mqttClient.PingAsync();
                        Console.WriteLine("Pong!");
                        break;
                    }
                    case { CmdGet: true, CmdVin: true }:
                    {
                        Console.WriteLine(rpcSession.Vin);
                        break;
                    }
                    case { CmdSet: true, CmdVin: true, ArgVin: var vin }:
                    {
                        Debug.Assert(vin is not null);

                        var oldVin = rpcSession.Vin;
                        if (await rpcSession.ChangeVinAsync(new(vin), CancellationToken.None))
                            Console.Error.WriteLine($"Okay (old = {oldVin}).");
                        break;
                    }
                    case { CmdShow: true, CmdTopics: true }:
                    {
                        Console.WriteLine($"req {rpcSession.RequestTopic}");
                        Console.WriteLine($"rsp {rpcSession.ResponseTopic}");
                        break;
                    }
                    case { CmdInspect: true, ArgNamespace: var ns, ArgQuery: var query }:
                    {
                        Debug.Assert(ns is not null);

                        request = FulfillRequest(ns, fi => fi.Inspect = new InspectIntent
                        {
                            Query = query
                        });
                        break;
                    }
                    case { CmdRead: true, ArgNamespace: var ns, ArgKey: var key }:
                    {
                        Debug.Assert(ns is not null);

                        request = FulfillRequest(ns, fi => fi.Read = new ReadIntent
                        {
                            Key = key
                        });
                        break;
                    }
                    case { CmdWrite: true, ArgNamespace: var ns, ArgKey: var key, ArgValue: var value }:
                    {
                        Debug.Assert(ns is not null);
                        Debug.Assert(value is not null);

                        request = FulfillRequest(ns, fi => fi.Write = new WriteIntent
                        {
                            Key = key,
                            Value = ParseValue(value)
                        });
                        break;
                    }
                    case { CmdInvoke: true, ArgNamespace: var ns, ArgCommand: var cmd, ArgArg: var cmdArgs }:
                    {
                        Debug.Assert(ns is not null);
                        Debug.Assert(cmdArgs is not null);

                        request = FulfillRequest(ns, fi =>
                        {
                            var invokeIntent = new InvokeIntent { Command = cmd };
                            invokeIntent.Args.AddRange(from arg in cmdArgs select ParseValue(arg));
                            fi.Invoke = invokeIntent;
                        });
                        break;
                    }
                    default:
                    {
                        Console.Error.WriteLine("Sorry, but this has not yet been implemented.");
                        break;
                    }
                }
                break;
            }
            case IInputErrorResult:
            {
                Console.Error.WriteLine("Invalid usage. Try one of the following:");
                Prompt.PrintUsage(Console.Error);
                break;
            }
        }

        if (request is { } someRequest)
        {
            using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(5));
            try
            {
                var response = await rpcSession.ExecuteAsync(someRequest, cts.Token);
                using var sw = new StringWriter();
                JsonFormatter.Default.Format(response, sw);
                var json = sw.ToString();
                json = JsonSerializer.Serialize(JsonSerializer.Deserialize<JsonElement>(json), jsonSerializerOptions);
                Console.WriteLine(json);
            }
            catch (OperationCanceledException ex)
            {
                Console.Error.WriteLine(ex.Message);
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

static Value ParseValue(string input)
{
    input = input.Trim();

    if (Regex.Match(input, @"^(?:true|false)$") is { Success: true, Value: var flag })
        return new Value { Bool = flag == "true" };

    if (Regex.Match(input, @"^[0-9]+$") is { Success: true, Value: var n32 })
        return new Value { Int32 = int.Parse(n32, CultureInfo.InvariantCulture) };

    if (Regex.Match(input, @"^[0-9]+(?=L$)") is { Success: true, Value: var n64 })
        return new Value { Int64 = long.Parse(n64, CultureInfo.InvariantCulture) };

    if (Regex.Match(input, @"^[0-9]*.[0-9]+(?=[fF]$)") is { Success: true, Value: var f32 })
        return new Value { Float32 = float.Parse(f32, CultureInfo.InvariantCulture) };

    if (Regex.Match(input, @"^[0-9]*.[0-9]+$") is { Success: true, Value: var f64 })
        return new Value { Float64 = double.Parse(f64, CultureInfo.InvariantCulture) };

    return new Value { String = input };
}

static FulfillRequest FulfillRequest(string ns, Action<Intent> intentInitializer)
{
    var intent = new Intent();
    intentInitializer(intent);
    var request = new FulfillRequest
    {
        Namespace = ns,
        Intent = intent
    };
    return request;
}

readonly record struct Vin(string Value)
{
    public override string ToString() => Value;
}

sealed class ChariottRpcSession : IDisposable
{
    readonly MqttFactory _factory;
    readonly IMqttClient _client;
    readonly IEnumerator<byte[]> _correlation;
    string? _cachedRequestTopic;
    string? _cachedResponseTopic;
    Vin _vin;

    public static async Task<ChariottRpcSession>
        CreateAsync(MqttFactory factory, IMqttClient client, Vin vin,
                    IEnumerable<byte[]> correlations,
                    CancellationToken cancellationToken)
    {
        var session = new ChariottRpcSession(factory, client, vin, correlations);
        await session.SubscribeAsync(cancellationToken);
        return session;
    }

    ChariottRpcSession(MqttFactory factory, IMqttClient client, Vin vin, IEnumerable<byte[]> correlations)
    {
        _factory = factory;
        _client = client;
        Vin = vin;
        _correlation = correlations.GetEnumerator();
    }

    public Vin Vin
    {
        get => _vin;

        private set
        {
            _vin = value;
            _cachedRequestTopic = _cachedResponseTopic = null;
        }
    }

    public string RequestTopic => _cachedRequestTopic ??= $"c2d/{Vin}";
    public string ResponseTopic => _cachedResponseTopic ??= $"c2d/{Vin}/rsvp";

    public void Dispose() => _correlation.Dispose();

    public async Task<bool> ChangeVinAsync(Vin newValue, CancellationToken cancellationToken)
    {
        if (Vin == newValue)
            return false;

        await UnsubscribeAsync(cancellationToken);
        Vin = newValue;
        await SubscribeAsync(cancellationToken);
        return true;
    }

    async Task SubscribeAsync(CancellationToken cancellationToken)
    {
        var options =
            _factory.CreateSubscribeOptionsBuilder()
                    .WithTopicFilter(ResponseTopic)
                    .Build();
        await _client.SubscribeAsync(options, cancellationToken);
    }

    async Task UnsubscribeAsync(CancellationToken cancellationToken)
    {
        var options =
            _factory.CreateUnsubscribeOptionsBuilder()
                    .WithTopicFilter(ResponseTopic)
                    .Build();
        await _client.UnsubscribeAsync(options, cancellationToken);
    }

    public Task<FulfillResponse> ExecuteAsync(FulfillRequest request, CancellationToken cancellationToken) =>
        ExecuteAsync(_client, request, _correlation, RequestTopic, ResponseTopic, cancellationToken);

    static Task<FulfillResponse> ExecuteAsync(IMqttClient client,
                                              FulfillRequest request,
                                              IEnumerator<byte[]> correlation,
                                              string requestTopic, string responseTopic,
                                              CancellationToken cancellationToken)
    {
        return !correlation.MoveNext()
             ? throw new InvalidOperationException()
             : Async(correlation.Current);

        async Task<FulfillResponse> Async(byte[] id)
        {
            var taskCompletionSource = new TaskCompletionSource<FulfillResponse>();

            Task OnApplicationMessageReceivedAsync(MqttApplicationMessageReceivedEventArgs args)
            {
                try
                {
                    if (args.ApplicationMessage.CorrelationData is { } correlationData
                        && id.SequenceEqual(correlationData))
                    {
                        var response = FulfillResponse.Parser.ParseFrom(args.ApplicationMessage.Payload);
                        taskCompletionSource.TrySetResult(response);
                    }
                }
                catch (Exception ex)
                {
                    taskCompletionSource.TrySetException(ex);
                }

                return Task.CompletedTask;
            }

            client.ApplicationMessageReceivedAsync += OnApplicationMessageReceivedAsync;

            try
            {
                var message =
                    new MqttApplicationMessageBuilder()
                        .WithTopic(requestTopic)
                        .WithPayload(request.ToByteArray())
                        .WithCorrelationData(id)
                        .WithResponseTopic(responseTopic)
                        .Build();

                await client.PublishAsync(message, cancellationToken);

                return await taskCompletionSource.Task.WaitAsync(cancellationToken);
            }
            finally
            {
                client.ApplicationMessageReceivedAsync -= OnApplicationMessageReceivedAsync;
            }
        }
    }
}

[DocoptArguments]
partial class Prompt
{
    const string Help = """
    Usage:
        $ ping
        $ set vin <vin>
        $ get vin
        $ inspect <namespace> <query>
        $ read <namespace> <key>
        $ write <namespace> <key> <value>
        $ invoke <namespace> <command> [<arg>...]
        $ subscribe <namespace> <source>...
        $ show topics
        $ show (req | request | rsp | response)
        $ (quit | exit)
        $ help
    """;

    public static void PrintUsage(TextWriter writer)
    {
        var e = Usage.AsSpan().EnumerateLines();
        e.MoveNext();
        while (e.MoveNext())
            writer.WriteLine(e.Current[6..]);
    }
}

static class Extensions
{
    public static T Dump<T>(this T value, TextWriter? output = null)
    {
        var json = JsonSerializer.Serialize(value, new JsonSerializerOptions { WriteIndented = true });
        output ??= Console.Error;
        output.WriteLine($"[{value?.GetType().Name}]:{Environment.NewLine}{json}");
        return value;
    }
}
