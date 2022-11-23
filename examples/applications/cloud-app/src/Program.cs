// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

using System;
using System.Collections.Generic;
using System.CommandLine.Parsing;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using Chariott.Common.V1;
using Chariott.Runtime.V1;
using Chariott.Streaming.V1;
using DocoptNet;
using Google.Protobuf;
using MQTTnet;
using MQTTnet.Client;
using MQTTnet.Formatter;
using MQTTnet.Protocol;
using static MoreLinq.Extensions.EvaluateExtension;
using static MoreLinq.Extensions.RepeatExtension;
using MoreEnumerable = MoreLinq.MoreEnumerable;

try
{
    return await ProgramArguments.ParseToMain(args, Main);
}
catch (Exception ex)
{
    Console.Error.WriteLine(ex);
    return 1;
}

static async Task<int> Main(ProgramArguments args)
{
    var jsonSerializerOptions = new JsonSerializerOptions { WriteIndented = true };
    var utf8 = new UTF8Encoding(encoderShouldEmitUTF8Identifier: false);

    var timeout = new Timeout(TimeSpan.FromSeconds(int.Parse(args.OptTimeout, NumberStyles.None, CultureInfo.InvariantCulture)));

    var mqttFactory = new MqttFactory();
    using var mqttClient = mqttFactory.CreateMqttClient();

    await timeout.ApplyAsync(cancellationToken =>
    {
        var options =
            mqttFactory.CreateClientOptionsBuilder()
                       .WithTcpServer(args.OptBroker)
                       .WithProtocolVersion(MqttProtocolVersion.V500)
                       .Build();
        return mqttClient.ConnectAsync(options, cancellationToken);
    });

    Console.Error.WriteLine("The MQTT client is connected.");

    var rpcClient = new ChariottRpcClient(mqttFactory, mqttClient);

    await timeout.ApplyAsync(cancellationToken =>
    {
        var options =
            mqttFactory.CreateSubscribeOptionsBuilder()
                       .WithTopicFilter(ChariottRpcClient.ResponseWildcardTopic)
                       .Build();
        return mqttClient.SubscribeAsync(options, cancellationToken);
    });

    var binName = Path.GetFileNameWithoutExtension(Environment.ProcessPath);

    var eventsChannelId = string.Join("/", Environment.MachineName,
                                           binName,
                                           Environment.ProcessId,
                                           "events");

    const string eventsFileExtension = ".cjson"; // https://en.wikipedia.org/wiki/JSON_streaming#Concatenated_JSON
    var eventFilesDirPath = Path.Join(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), $".{binName}", "events");
    Directory.CreateDirectory(eventFilesDirPath);

    var oldDate = DateTime.Today.AddDays(-30);
    var oldEventsFiles =
        from f in new DirectoryInfo(eventFilesDirPath).EnumerateFiles("*" + eventsFileExtension, new EnumerationOptions())
        where f.CreationTime.Date < oldDate
        select f;

    foreach (var file in oldEventsFiles)
        file.Delete();

    var eventsFilePath = Path.Join(eventFilesDirPath, eventsChannelId.Replace('/', '=') + eventsFileExtension);
    var eventsFileReadPosition = 0L;
    var eventsFileLock = new SemaphoreSlim(1);

    var prettyPrintEventsJson = args.OptPrettyEvents;

    mqttClient.ApplicationMessageReceivedAsync += async args =>
    {
        if (args.ApplicationMessage.Topic != eventsChannelId)
            return;

        var @event = Event.Parser.ParseFrom(args.ApplicationMessage.Payload);
        var json = @event.ToJsonEncoding(prettyPrintEventsJson ? jsonSerializerOptions : null);
        await eventsFileLock.WaitAsync();
        try
        {
            await using var stream = File.Open(eventsFilePath, FileMode.Append, FileAccess.Write, FileShare.Read);
            await using var writer = new StreamWriter(stream, utf8);
            await writer.WriteLineAsync(json);
        }
        finally
        {
            eventsFileLock.Release();
        }
    };

    await timeout.ApplyAsync(cancellationToken =>
    {
        var options =
            mqttFactory.CreateSubscribeOptionsBuilder()
                       .WithTopicFilter(eventsChannelId)
                       .Build();
        return mqttClient.SubscribeAsync(options, cancellationToken);
    });

    var isFirstSubscription = true;
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
                        await timeout.ApplyAsync(mqttClient.PingAsync);
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
                    case { CmdShow: true, CmdNew: true, CmdEvents: true }:
                    {
                        if (!File.Exists(eventsFilePath))
                        {
                            Console.Error.WriteLine("No events so far! Did you forget to subscribe?");
                            break;
                        }

                        try
                        {
                            await eventsFileLock.WaitAsync(TimeSpan.FromSeconds(5));
                            try
                            {
                                await using var stream = File.OpenRead(eventsFilePath);
                                stream.Position = eventsFileReadPosition;
                                using var reader = new StreamReader(stream, utf8);
                                while (await reader.ReadLineAsync() is { } fileLine)
                                    Console.WriteLine(fileLine);
                                eventsFileReadPosition = stream.Position;
                            }
                            finally
                            {
                                eventsFileLock.Release();
                            }
                        }
                        catch (Exception ex)
                        {
                            Console.Error.WriteLine(ex);
                        }
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
                    case { CmdSubscribe: true, ArgNamespace: var ns, ArgSource: var sources }:
                    {
                        Debug.Assert(ns is not null);

                        request = FulfillRequest(ns, fi =>
                        {
                            var subscribeIntent = new SubscribeIntent { ChannelId = eventsChannelId };
                            subscribeIntent.Sources.AddRange(sources);
                            fi.Subscribe = subscribeIntent;
                        });

                        if (isFirstSubscription)
                        {
                            Console.Error.WriteLine($"The events channel identifier is: {eventsChannelId}");
                            Console.Error.WriteLine(eventsFilePath);
                            isFirstSubscription = false;
                        }
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
            try
            {
                var response = await timeout.ApplyAsync(cancellationToken =>
                    rpcClient.ExecuteAsync(session.Vin, someRequest, cancellationToken));

                Console.WriteLine(response.ToJsonEncoding(jsonSerializerOptions));
            }
            catch (OperationCanceledException ex)
            {
                Console.Error.WriteLine(ex.Message);
            }
            catch (ChariottRpcException ex)
            {
                Console.Error.WriteLine(ex.Detail ?? ex.Message);
            }
        }
    }

    await timeout.ApplyAsync(cancellationToken =>
        mqttClient.DisconnectAsync(mqttFactory.CreateClientDisconnectOptionsBuilder().Build(), cancellationToken));

    return 0;
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
                    var message = args.ApplicationMessage;
                    if (message is { Topic: { } topic, CorrelationData: { } correlationData }
                        && topic == topics.Response && id == new Guid(correlationData))
                    {
                        var error = message.UserProperties.FirstOrDefault(p => p.Name is "error");
                        if (error is { Value: not "0" and not "" })
                        {
                            var detail
                                = message.PayloadFormatIndicator == MqttPayloadFormatIndicator.CharacterData
                                ? Encoding.UTF8.GetString(message.Payload)
                                : MediaTypeHeaderValue.TryParse(message.ContentType, out var contentType)
                                  && "application/x-proto+chariott.common.v1.Value".Equals(contentType.MediaType, StringComparison.OrdinalIgnoreCase)
                                ? Value.Parser.ParseFrom(message.Payload) switch
                                  {
                                      { String: var str } => str,
                                      var val => val.ToJsonEncoding()
                                  }
                                : null;

                            taskCompletionSource.TrySetException(new ChariottRpcException(null, detail));
                        }
                        else if (MediaTypeHeaderValue.TryParse(message.ContentType, out var contentType)
                                 && "application/x-proto+chariott.common.v1.FulfillResponse".Equals(contentType.MediaType, StringComparison.OrdinalIgnoreCase))
                        {
                            var response = FulfillResponse.Parser.ParseFrom(message.Payload);
                            taskCompletionSource.TrySetResult(response);
                        }
                        else
                        {
                            throw new ChariottRpcException("Unexpected response to remote procedure call.");
                        }
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
                           .WithQualityOfServiceLevel(MqttQualityOfServiceLevel.ExactlyOnce)
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

public class ChariottRpcException : Exception
{
    public ChariottRpcException() : this(null, null, null) { }
    public ChariottRpcException(string? message) : this(message, null, null) { }
    public ChariottRpcException(string? message, string? detail) : this(message, detail, null) { }
    public ChariottRpcException(string? message, Exception? inner) : this(message, null, inner) { }

    public ChariottRpcException(string? message, string? detail, Exception? inner) :
        base(message ?? "Remote procedure call failed.", inner) =>
        Detail = detail;

    public string? Detail { get; }
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
        $ show new events
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
            $ [--broker=<host>] [--vin=<vin>] [--timeout=<sec>] [--pretty-events]
            $ -h | --help

        Options:
            -h --help        Show this screen.
            --broker=<host>  MQTT broker address [default: localhost].
            --vin=<vin>      VIN umber [default: 1].
            --timeout=<sec>  Timeout in seconds [default: 5].
            --pretty-events  Pretty print events JSON.
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

readonly record struct Timeout(TimeSpan Duration);

static class Extensions
{
    public static string ToJsonEncoding(this IMessage message, JsonSerializerOptions? jsonSerializerOptions = null)
    {
        using var sw = new StringWriter();
        JsonFormatter.Default.Format(message, sw);
        var json = sw.ToString();
        return JsonSerializer.Serialize(JsonSerializer.Deserialize<JsonElement>(json), jsonSerializerOptions);
    }

    public static Task ApplyAsync(this Timeout timeout, Func<CancellationToken, Task> function) =>
        timeout.ApplyAsync(async cancellationToken =>
        {
            await function(cancellationToken);
            return 0;
        });

    public static async Task<T> ApplyAsync<T>(this Timeout timeout, Func<CancellationToken, Task<T>> function)
    {
        using var cts = timeout is { Duration: var delay } && delay >= TimeSpan.Zero
            ? new CancellationTokenSource(delay)
            : null;
        return await function(cts?.Token ?? CancellationToken.None);
    }
}
