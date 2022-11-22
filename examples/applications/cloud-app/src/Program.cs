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
using System.CommandLine.Parsing;

return await ProgramArguments.ParseToMain(args, Main);

static async Task<int> Main(ProgramArguments args)
{
    try
    {
        var mqttFactory = new MqttFactory();

        using var mqttClient = mqttFactory.CreateMqttClient();

        var mqttClientOptions =
            mqttFactory.CreateClientOptionsBuilder()
                       .WithTcpServer(args.OptBroker)
                       .WithProtocolVersion(MqttProtocolVersion.V500)
                       .Build();

        await mqttClient.ConnectAsync(mqttClientOptions, CancellationToken.None);
        Console.Error.WriteLine("The MQTT client is connected.");

        var rpcClient = new ChariottRpcClient(mqttFactory, mqttClient);

        var options =
            mqttFactory.CreateSubscribeOptionsBuilder()
                       .WithTopicFilter(ChariottRpcClient.ResponseWildcardTopic)
                       .Build();
        await mqttClient.SubscribeAsync(options, CancellationToken.None);

        var jsonSerializerOptions = new JsonSerializerOptions { WriteIndented = true };

        var session = new Session { Vin = new(args.OptVin) };

        var quit = false;
        while (!quit && Console.ReadLine() is { } line)
        {
            FulfillRequest? request = null;
            switch (PromptArguments.CreateParser().Parse(CommandLineStringSplitter.Instance.Split(line)))
            {
                case IArgumentsResult<PromptArguments> { Arguments: var promptArgs }:
                {
                    switch (promptArgs)
                    {
                        case { CmdQuit: true } or { CmdExit: true }:
                        {
                            quit = true;
                            break;
                        }
                        case { CmdHelp: true }:
                        {
                            PromptArguments.PrintUsage(Console.Out);
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
                            Console.WriteLine(session.Vin);
                            break;
                        }
                        case { CmdSet: true, CmdVin: true, ArgVin: var vin }:
                        {
                            Debug.Assert(vin is not null);
                            session = session with { Vin = new(vin) };
                            break;
                        }
                        case { CmdShow: true, CmdTopics: true }:
                        {
                            var topics = ChariottRpcClient.GetTopics(session.Vin);
                            Console.WriteLine($"req {topics.Request}");
                            Console.WriteLine($"rsp {topics.Response}");
                            break;
                        }
                        case { CmdInspect: true, ArgNamespace: var ns, ArgQuery: var query }:
                        {
                            Debug.Assert(ns is not null);

                            request = FulfillRequest(ns, fi => fi.Inspect = new() { Query = query });
                            break;
                        }
                        case { CmdRead: true, ArgNamespace: var ns, ArgKey: var key }:
                        {
                            Debug.Assert(ns is not null);

                            request = FulfillRequest(ns, fi => fi.Read = new() { Key = key });
                            break;
                        }
                        case { CmdWrite: true, ArgNamespace: var ns, ArgKey: var key, ArgValue: var value }:
                        {
                            Debug.Assert(ns is not null);
                            Debug.Assert(value is not null);

                            request = FulfillRequest(ns, fi => fi.Write = new() { Key = key,  Value = ParseValue(value) });
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
                    PromptArguments.PrintUsage(Console.Error);
                    break;
                }
            }

            if (request is { } someRequest)
            {
                using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(5));
                try
                {
                    var response = await rpcClient.ExecuteAsync(session.Vin, someRequest, cts.Token);
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
}

static Value ParseValue(string input)
{
    input = input.Trim();

    if (Regex.Match(input, @"^(?:true|false)$") is { Success: true, Value: var flag })
        return new() { Bool = flag == "true" };

    if (Regex.Match(input, @"^[0-9]+$") is { Success: true, Value: var n32 })
        return new() { Int32 = int.Parse(n32, CultureInfo.InvariantCulture) };

    if (Regex.Match(input, @"^[0-9]+(?=L$)") is { Success: true, Value: var n64 })
        return new() { Int64 = long.Parse(n64, CultureInfo.InvariantCulture) };

    if (Regex.Match(input, @"^[0-9]*.[0-9]+(?=[fF]$)") is { Success: true, Value: var f32 })
        return new() { Float32 = float.Parse(f32, CultureInfo.InvariantCulture) };

    if (Regex.Match(input, @"^[0-9]*.[0-9]+$") is { Success: true, Value: var f64 })
        return new() { Float64 = double.Parse(f64, CultureInfo.InvariantCulture) };

    return new() { String = input };
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

record Session
{
    public required Vin Vin { get; init; }
}

readonly record struct Vin(string Value)
{
    public override string ToString() => Value;
}

readonly record struct RpcTopicPair(string Request, string Response);

sealed class ChariottRpcClient : IDisposable
{
    readonly MqttFactory _factory;
    readonly IMqttClient _client;
    readonly IEnumerator<Guid> _correlation;

    public const string ResponseWildcardTopic = "c2d/+/rsvp";

    public ChariottRpcClient(MqttFactory factory, IMqttClient client) :
        this(factory, client, MoreEnumerable.Return(Guid.NewGuid).Repeat().Evaluate()) { }

    public ChariottRpcClient(MqttFactory factory, IMqttClient client, IEnumerable<Guid> correlations)
    {
        _factory = factory;
        _client = client;
        _correlation = correlations.GetEnumerator();
    }

    public void Dispose() => _correlation.Dispose();

    public static RpcTopicPair GetTopics(Vin vin) => new($"c2d/{vin}", $"c2d/{vin}/rsvp");

    public Task<FulfillResponse> ExecuteAsync(Vin vin, FulfillRequest request,
                                              CancellationToken cancellationToken) =>
        ExecuteAsync(_factory, _client, request, _correlation, GetTopics(vin),
                     cancellationToken);

    static Task<FulfillResponse> ExecuteAsync(MqttFactory factory,
                                              IMqttClient client,
                                              FulfillRequest request,
                                              IEnumerator<Guid> correlation,
                                              RpcTopicPair topics,
                                              CancellationToken cancellationToken)
    {
        return !correlation.MoveNext()
             ? throw new InvalidOperationException()
             : Async(correlation.Current);

        async Task<FulfillResponse> Async(Guid id)
        {
            var taskCompletionSource = new TaskCompletionSource<FulfillResponse>();

            Task OnApplicationMessageReceivedAsync(MqttApplicationMessageReceivedEventArgs args)
            {
                try
                {
                    if (args.ApplicationMessage is { Topic: { } topic, CorrelationData: { } correlationData }
                        && topic == topics.Response && id == new Guid(correlationData))
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
                    factory.CreateApplicationMessageBuilder()
                           .WithTopic(topics.Request)
                           .WithPayload(request.ToByteArray())
                           .WithCorrelationData(id.ToByteArray())
                           .WithResponseTopic(topics.Response)
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
partial class PromptArguments
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

[DocoptArguments]
partial class ProgramArguments
{
    const string Help = """
        Car Bridge Cloud Application

        Usage:
            $ [--broker=<host>] [--vin=<vin>]
            $ (-h | --help)
            $ --version

        Options:
            -h --help        Show this screen.
            --version        Show version.
            --broker=<host>  MQTT broker address [default: localhost].
            --vin=<vin>      VIN umber [default: 1]
        """;

    public static Task<int> ParseToMain(string[] args, Func<ProgramArguments, Task<int>> main)
    {
        return CreateParser().Parse(args)
                             .Match(main,
                                    result => Print(Console.Out, result.Help),
                                    result => Print(Console.Error, result.Usage, 1));

        static Task<int> Print(TextWriter writer, string text, int exitCode = 0)
        {
            writer.WriteLine(text.Replace("$", Path.GetFileName(Environment.ProcessPath)));
            return Task.FromResult(exitCode);
        }
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
