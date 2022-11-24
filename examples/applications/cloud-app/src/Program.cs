// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

using System;
using System.CommandLine.Parsing;
using System.Diagnostics;
using System.Diagnostics.CodeAnalysis;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using CarBridgeCloudApp;
using Chariott.Common.V1;
using Chariott.Runtime.V1;
using Chariott.Streaming.V1;
using DocoptNet;
using MQTTnet;
using MQTTnet.Formatter;

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

    var eventsTopic = string.Join("/", Environment.MachineName,
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

    var eventsFilePath = Path.Join(eventFilesDirPath, eventsTopic.Replace('/', '=') + eventsFileExtension);
    var eventsFileReadPosition = 0L;
    var eventsFileLock = new SemaphoreSlim(1);

    var prettyPrintEventsJson = args.OptPrettyEvents;

    var newEventCountLock = new object();
    var newEventCount = 0;

    mqttClient.ApplicationMessageReceivedAsync += async args =>
    {
        Console.Error.WriteLine("Program: " + args.ApplicationMessage.Topic);

        if (args.ApplicationMessage.Topic != eventsTopic)
            return;

        var @event = Event.Parser.ParseFrom(args.ApplicationMessage.Payload);
        var json = @event.ToJsonEncoding(prettyPrintEventsJson ? jsonSerializerOptions : null);

        if (!await eventsFileLock.WaitAsync(TimeSpan.FromSeconds(5)))
        {
            Console.Error.WriteLine("Warning! An event was lost due to time-out waiting for recording.");
            return;
        }

        try
        {
            await using var stream = File.Open(eventsFilePath, FileMode.Append, FileAccess.Write, FileShare.Read);
            await using var writer = new StreamWriter(stream, utf8);
            await writer.WriteLineAsync(json);

            lock (newEventCountLock)
                newEventCount++;
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
                       .WithTopicFilter(eventsTopic)
                       .Build();
        return mqttClient.SubscribeAsync(options, cancellationToken);
    });

    var session = new Session { Vin = new(args.OptVin) };

    var quit = false;
    var isOutputRedirected = Console.IsOutputRedirected;

    async Task<string?> PromptAsync()
    {
        int eventCount;
        lock (newEventCountLock)
            eventCount = newEventCount;
        if (eventCount > 0)
            Console.Error.WriteLine($"There are new events ({eventCount}). Use \"show new events\" to see them.");

        if (!isOutputRedirected)
            Console.Write("> ");

        return await Task.Factory.StartNew(Console.ReadLine, TaskCreationOptions.DenyChildAttach
                                                           | TaskCreationOptions.LongRunning);
    }

    while (!quit && await PromptAsync() is { } line)
    {
        if (line.AsSpan().TrimEnd().Length is 0)
            continue;

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
                    case { CmdGet: true, CmdEventsDotFile: true }:
                    {
                        Console.WriteLine(eventsFilePath);
                        break;
                    }
                    case { CmdShow: true, CmdTopics: true }:
                    {
                        var topics = ChariottRpcClient.GetTopics(session.Vin);
                        Console.WriteLine($"request: {topics.Request}");
                        Console.WriteLine($"response: {topics.Response}");
                        Console.WriteLine($"events: {eventsTopic}");
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
                            if (!await eventsFileLock.WaitAsync(TimeSpan.FromSeconds(5)))
                                throw new TimeoutException();

                            try
                            {
                                await using var stream = File.OpenRead(eventsFilePath);
                                stream.Position = eventsFileReadPosition;
                                using var reader = new StreamReader(stream, utf8);
                                while (await reader.ReadLineAsync() is { } fileLine)
                                    Console.WriteLine(fileLine);
                                eventsFileReadPosition = stream.Position;

                                lock (newEventCountLock)
                                    newEventCount = 0;
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
                    case { CmdWrite: true, ArgNamespace: var ns, ArgKey: var key, ArgValue: var valueText }:
                    {
                        Debug.Assert(ns is not null);
                        Debug.Assert(valueText is not null);

                        if (TryParseValue(valueText, out var value))
                            request = FulfillRequest(ns, fi => fi.Write = new() { Key = key, Value = value });
                        else
                            Console.Error.WriteLine($"Invalid value: {valueText}");
                        break;
                    }
                    case { CmdInvoke: true, ArgNamespace: var ns, ArgCommand: var cmd, ArgArg: var cmdArgs }:
                    {
                        Debug.Assert(ns is not null);
                        Debug.Assert(cmdArgs is not null);

                        var argValuePairs =
                            cmdArgs.Select(arg => TryParseValue(arg, out var v) ? (Arg: arg, Value: v) : (arg, null))
                                   .ToList();

                        if (argValuePairs.All(av => av is (_, not null)))
                        {
                            request = FulfillRequest(ns, fi =>
                            {
                                var invokeIntent = new InvokeIntent { Command = cmd };
                                invokeIntent.Args.AddRange(from av in argValuePairs select av.Value);
                                fi.Invoke = invokeIntent;
                            });
                        }
                        else
                        {
                            Console.Error.WriteLine($"Invalid value: {argValuePairs.First(v => v.Value is null).Arg}");
                        }
                        break;
                    }
                    case { CmdSubscribe: true, ArgNamespace: var ns, ArgSource: var sources }:
                    {
                        Debug.Assert(ns is not null);

                        request = FulfillRequest(ns, fi =>
                        {
                            var subscribeIntent = new SubscribeIntent { ChannelId = eventsTopic };
                            subscribeIntent.Sources.AddRange(sources);
                            fi.Subscribe = subscribeIntent;
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

static bool TryParseValue(string input, [NotNullWhen(true)] out Value? value)
{
    input = input.Trim();
    value = null;

    if (input is var @bool and ("true" or "false"))
    {
        value = new() { Bool = @bool is "true" };
    }
    else if (Regex.Match(input, @"^(?:\\-|\+)?[0-9]+$") is { Success: true, Value: var n32s })
    {
        if (int.TryParse(n32s.AsSpan().TrimStart('\\'), NumberStyles.Integer, CultureInfo.InvariantCulture, out var n32))
            value = new() { Int32 = n32 };
    }
    else if (Regex.Match(input, @"^(?:\\-|\+)?[0-9]+(?=L$)") is { Success: true, Value: var n64s })
    {
        if (long.TryParse(n64s.AsSpan().TrimStart('\\'), NumberStyles.Integer, CultureInfo.InvariantCulture, out var n64))
            value = new() { Int64 = n64 };
    }
    else if (Regex.Match(input, @"^(?:\\-|\+)?[0-9]*.[0-9]+(?=[fF]$)") is { Success: true, Value: var f32s })
    {
        if (float.TryParse(f32s.AsSpan().TrimStart('\\'), NumberStyles.Float, CultureInfo.InvariantCulture, out var f32))
            value = new() { Float32 = f32 };
    }
    else if (Regex.Match(input, @"^(?:\\-|\+)?[0-9]*.[0-9]+$") is { Success: true, Value: var f64s })
    {
        if (double.TryParse(f64s.AsSpan().TrimStart('\\'), NumberStyles.Float, CultureInfo.InvariantCulture, out var f64))
            value = new() { Float64 = f64 };
    }
    else
    {
        value = new() { String = input is ['\'', .., '\''] ? input[1..^1] : input };
    }

    return value is not null;
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

readonly record struct Timeout(TimeSpan Duration);

record Session
{
    public required Vin Vin { get; init; }
}

readonly record struct Vin(string Value)
{
    public override string ToString() => Value;
}

[DocoptArguments]
partial class PromptArguments
{
    const string Help = """
        Usage:
            $ ping
            $ set vin <vin>
            $ get vin
            $ get events.file
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
