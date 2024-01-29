// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

using System.Collections;
using System.Collections.Specialized;
using System.Text.Json;
using System.Threading.Channels;
using Grpc.Core;
using Grpc.Net.Client;
using Microsoft.AspNetCore.Mvc;
using IntentBrokeringCommon = IntentBrokering.Common.V1;
using IntentBrokeringRuntime = IntentBrokering.Runtime.V1;
using IntentBrokeringStreaming = IntentBrokering.Streaming.V1;

var clients = new ConcurrentList<ChannelWriter<object>>();

var builder = WebApplication.CreateBuilder(new WebApplicationOptions
{
    Args = args,
    WebRootPath = "wwwroot",
});
builder.Services.AddHostedService<SdvEventReadingService>();
builder.Services.AddSingleton<IEnumerable<ChannelWriter<object>>>(clients);
builder.Services.AddSingleton(_ => new SocketsHttpHandler { EnableMultipleHttp2Connections = true });

builder.Services.AddGrpcClient<IntentBrokeringRuntime.IntentBrokeringService.IntentBrokeringServiceClient>((sp, options) =>
{
    options.Address = new Uri("http://localhost:4243/"); // DevSkim: ignore DS162092
    options.ChannelOptionsActions.Add(options =>
    {
        options.HttpHandler = sp.GetRequiredService<SocketsHttpHandler>();
        if (options.ServiceProvider is { } services)
            options.LoggerFactory = services.GetService<ILoggerFactory>();
    });
});

var app = builder.Build();
app.UseDefaultFiles();
app.UseStaticFiles();

app.MapPost("/dog-mode", async (HttpContext context, [FromServices] IntentBrokeringRuntime.IntentBrokeringService.IntentBrokeringServiceClient client) =>
{
    _ = await client.FulfillAsync(new IntentBrokeringRuntime.FulfillRequest
    {
        Namespace = KeyValueStoreProperties.Namespace,
        Intent = new IntentBrokeringCommon.Intent
        {
            Write = new IntentBrokeringCommon.WriteIntent
            {
                Key = KeyValueStoreProperties.DogModeStatus,
                Value = new IntentBrokeringCommon.Value { Bool = "on" == context.Request.Form["on"] }
            }
        }
    });
});

app.MapGet("/events", async context =>
{
    var response = context.Response;
    response.ContentType = "text/event-stream";
    var channel = Channel.CreateUnbounded<object>();
    clients.Add(channel.Writer);
    try
    {
        await foreach (var obj in channel.Reader.ReadAllAsync(context.RequestAborted))
        {
            var dataLine = obj switch
            {
                IntentBrokeringStreaming.Event e =>
                    JsonSerializer.Serialize(new
                    {
                        id = e.Source,
                        data = e.Value.ValueCase switch
                        {
                            IntentBrokeringCommon.Value.ValueOneofCase.Int32 => (object)e.Value.Int32,
                            IntentBrokeringCommon.Value.ValueOneofCase.Bool => e.Value.Bool,
                            IntentBrokeringCommon.Value.ValueOneofCase.Blob => new { type = e.Value.Blob.MediaType, value = e.Value.Blob.Bytes.ToBase64() },
                            _ => "Unsupported value type",
                        }
                    }),
                string str => str,
                _ => null
            };

            if (dataLine is { } someDataLine)
            {
                if (dataLine.IndexOf('\n') >= 0)
                {
                    foreach (var line in dataLine.Split('\n'))
                        await response.WriteAsync($"data: {line}\n");
                    await response.WriteAsync("\n");
                }
                else
                {
                    await response.WriteAsync($"data: {someDataLine}\n\n");
                }
            }
        }
    }
    catch (OperationCanceledException) // DevSkim: ignore DS176209 TODO investigate why this is needed;
    {                                  // seems to "sometimes" crash the process
        // ignore                      // if browser is closed (request aborted).
    }
    finally
    {
        clients.Remove(channel.Writer);
    }
});

app.Run();

sealed class SdvEventReadingService : BackgroundService
{
    readonly IntentBrokeringRuntime.IntentBrokeringService.IntentBrokeringServiceClient _client;
    readonly IEnumerable<ChannelWriter<object>> _writers;
    readonly SocketsHttpHandler _httpHandler;
    readonly ILoggerFactory _loggerFactory;
    readonly ILogger<SdvEventReadingService> _logger;

    public SdvEventReadingService(IntentBrokeringRuntime.IntentBrokeringService.IntentBrokeringServiceClient client,
                                  IEnumerable<ChannelWriter<object>> writers,
                                  SocketsHttpHandler httpHandler,
                                  ILoggerFactory loggerFactory,
                                  ILogger<SdvEventReadingService> logger)
    {
        _client = client;
        _writers = writers;
        _httpHandler = httpHandler;
        _loggerFactory = loggerFactory;
        _logger = logger;
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        while (true)
        {
            var newClientNoticeChannel = Channel.CreateUnbounded<ChannelWriter<object>>();

            void OnCollectionChanged(object? sender, NotifyCollectionChangedEventArgs args)
            {
                if (args.Action is NotifyCollectionChangedAction.Add && args.NewItems?[0] is ChannelWriter<object> writer)
                {
                    _ = Task.Run(cancellationToken: stoppingToken, function: async () =>
                    {
                        try
                        {
                            await newClientNoticeChannel.Writer.WriteAsync(writer, stoppingToken);
                        }
                        catch (Exception ex)
                        {
                            _logger.LogDebug(ex, null);
                        }
                    });
                }
            }

            {
                if (_writers is INotifyCollectionChanged changeSource)
                    changeSource.CollectionChanged += OnCollectionChanged;
            }

            try
            {
                IDisposable? d1 = null;
                IDisposable? d2 = null;
                IDisposable? d3 = null;

                try
                {
                    (d1, var vdtStream) = await StreamAsync(VdtProperties.Namespace, new[] { VdtProperties.CabinTemperature, VdtProperties.BatteryLevel, VdtProperties.AirConditioningState }, stoppingToken);
                    (d2, var keyValueStoreStream) = await StreamAsync(KeyValueStoreProperties.Namespace, new[] { KeyValueStoreProperties.DogModeStatus }, stoppingToken);
                    (d3, var cameraStream) = await StreamAsync(SimulatedCameraProperties.Namespace, new[] { SimulatedCameraProperties.DesiredFrequency }, stoppingToken);

                    await using var @event = AsyncEnumerableEx.Merge(vdtStream, keyValueStoreStream, cameraStream).GetAsyncEnumerator(stoppingToken);
                    var nextEventTask = @event.MoveNextAsync(stoppingToken).AsTask();
                    var newClientNoticeTask = newClientNoticeChannel.Reader.ReadAsync(stoppingToken).AsTask();

                    while (true)
                    {
                        var completedTask = await Task.WhenAny(nextEventTask, newClientNoticeTask);

                        while (newClientNoticeChannel.Reader.TryRead(out var writer))
                            await writer.WriteAsync("connected", stoppingToken);

                        if (completedTask == newClientNoticeTask)
                        {
                            var writer = await newClientNoticeTask;
                            await writer.WriteAsync("connected", stoppingToken);
                            newClientNoticeTask = newClientNoticeChannel.Reader.ReadAsync(stoppingToken).AsTask();
                        }

                        if (completedTask == nextEventTask)
                        {
                            if (!await nextEventTask)
                                break;

                            foreach (var writer in _writers)
                                await writer.WriteAsync(@event.Current, stoppingToken);

                            nextEventTask = @event.MoveNextAsync(stoppingToken).AsTask();
                        }
                    }
                }
                finally
                {
                    d1?.Dispose();
                    d2?.Dispose();
                    d3?.Dispose();
                }
            }
            catch (Exception ex) when (ex is RpcException { StatusCode: StatusCode.Unavailable or StatusCode.Cancelled }
                                          or OperationCanceledException)
            {
                _logger.LogDebug(ex, null);

                if (_writers is INotifyCollectionChanged changeSource)
                    changeSource.CollectionChanged -= OnCollectionChanged;

                foreach (var writer in _writers)
                    await writer.WriteAsync("disconnected", stoppingToken);

                if (ex is RpcException { StatusCode: StatusCode.Unavailable })
                    await Task.Delay(TimeSpan.FromSeconds(5), stoppingToken);
            }
        }
    }

    sealed class DisposableList : IDisposable
    {
        readonly List<IDisposable> _disposables = new();

        public void Add(IDisposable disposable) =>
            _disposables.Add(disposable);

        public void Dispose() => _disposables.ForEach(d => d.Dispose());
    }

    async Task<(IDisposable, IAsyncEnumerable<IntentBrokeringStreaming.Event>)> StreamAsync(string @namespace, IEnumerable<string> sources, CancellationToken cancellationToken)
    {
        var disposables = new DisposableList();

        try
        {
            var streamingAddressCandidates = await _client.FulfillAsync(new IntentBrokeringRuntime.FulfillRequest
            {
                Namespace = @namespace,
                Intent = new IntentBrokeringCommon.Intent { Discover = new IntentBrokeringCommon.DiscoverIntent() }
            },
            cancellationToken: cancellationToken);

            var streamingAddress = streamingAddressCandidates.Fulfillment.Discover.Services
                .First(s => s.SchemaReference == "intent_brokering.streaming.v1" && s.SchemaKind == "grpc+proto")
                .Url;

            var channel = GrpcChannel.ForAddress(streamingAddress, new GrpcChannelOptions
            {
                LoggerFactory = _loggerFactory,
                HttpHandler = _httpHandler
            });

            disposables.Add(channel);

            var streamingClient = new IntentBrokeringStreaming.ChannelService.ChannelServiceClient(channel);
            var streamingCall = streamingClient.Open(new IntentBrokeringStreaming.OpenRequest(), cancellationToken: cancellationToken);
            disposables.Add(streamingCall);
            var channelId = (await streamingCall.GetResponseHeadersAsync(cancellationToken)).Get("x-chariott-channel-id")?.Value ??
                throw new InvalidOperationException("Channel ID not present in response header.");

            using (var timeoutCancellationTokenSource = new CancellationTokenSource(TimeSpan.FromSeconds(5)))
            using (var linkedCancellationTokenSource = CancellationTokenSource.CreateLinkedTokenSource(timeoutCancellationTokenSource.Token, cancellationToken))
                _ = await streamingCall.GetResponseHeadersAsync(linkedCancellationTokenSource.Token);

            foreach (var writer in _writers)
                await writer.WriteAsync("connected", cancellationToken);

            var rsr = await _client.FulfillAsync(new IntentBrokeringRuntime.FulfillRequest
            {
                Namespace = @namespace,
                Intent = new IntentBrokeringCommon.Intent
                {
                    Subscribe = new IntentBrokeringCommon.SubscribeIntent
                    {
                        ChannelId = channelId,
                        Sources = { sources }
                    }
                }
            }, cancellationToken: cancellationToken);

            _logger.LogDebug(rsr.ToString());

            return (disposables, streamingCall.ResponseStream.ReadAllAsync());
        }
        catch (Exception)
        {
            disposables.Dispose();
            throw;
        }
    }

    public override Task StartAsync(CancellationToken cancellationToken)
    {
        _logger.LogInformation($"{nameof(SdvEventReadingService)} is starting.");
        return base.StartAsync(cancellationToken);
    }

    public override Task StopAsync(CancellationToken cancellationToken)
    {
        _logger.LogInformation($"{nameof(SdvEventReadingService)} is stopping.");
        return base.StopAsync(cancellationToken);
    }
}

static class VdtProperties
{
    public const string Namespace = "sdv.vdt";
    public const string CabinTemperature = "Vehicle.Cabin.HVAC.AmbientAirTemperature";
    public const string BatteryLevel = "Vehicle.OBD.HybridBatteryRemaining";
    public const string AirConditioningState = "Vehicle.Cabin.HVAC.IsAirConditioningActive";
}

static class KeyValueStoreProperties
{
    public const string Namespace = "sdv.kvs";
    public const string DogModeStatus = "Feature.DogMode.Status";
}

static class SimulatedCameraProperties
{
    public const string Namespace = "sdv.camera.simulated";
    public const string DesiredFrequency = "camera.12fpm";
}

static class Extensions
{
    public static async Task<Metadata> GetResponseHeadersAsync<T>(this AsyncServerStreamingCall<T> call, CancellationToken cancellationToken)
    {
        var done = 0;

        await using (cancellationToken.Register(() =>
        {
            if (Interlocked.CompareExchange(ref done, 1, 0) == 0)
                call.Dispose();
        }))
        {
            Metadata metadata;

            try
            {
                metadata = await call.ResponseHeadersAsync;
            }
            catch (RpcException ex) when (ex.StatusCode is StatusCode.Cancelled
                                          && cancellationToken.IsCancellationRequested)
            {
                throw new OperationCanceledException(null, ex, cancellationToken);
            }

            if (Interlocked.CompareExchange(ref done, 1, 0) == 1)
                throw new OperationCanceledException(cancellationToken); // lost the race

            return metadata;
        }
    }
}

/// <summary>
/// An <see cref="IList{T}"/> wrapper that provides synchronized access to the
/// underlying list with copy-on-write semantics such that members reading the
/// list provide a snapshot in time. It also allows the collection to be
/// observed for changes via <see cref="INotifyCollectionChanged"/>.
/// </summary>

sealed class ConcurrentList<T> : IList<T>, INotifyCollectionChanged
{
    readonly object _lock = new();
    readonly IList<T> _items;
    IList<T>? _copy;

    public ConcurrentList() : this(new List<T>()) { }
    public ConcurrentList(IList<T> items) => _items = items;

    public event NotifyCollectionChangedEventHandler? CollectionChanged;

    IList<T> Items
    {
        get
        {
            lock (_lock)
                return _copy ??= _items.ToArray();
        }
    }

    void Update(Action<IList<T>> action) => Update(action, static (items, action) => action(items));

    void Update<TArg>(TArg arg, Action<IList<T>, TArg> action) =>
        Update(action, arg, static (items, action, arg) => action(items, arg));

    void Update<T1, T2>(T1 arg1, T2 arg2, Action<IList<T>, T1, T2> action) =>
        Update(action, arg1, arg2, static (items, action, arg1, arg2) =>
        {
            action(items, arg1, arg2);
            return 0;
        });

    TResult Update<TArg, TResult>(TArg arg, Func<IList<T>, TArg, TResult> func) =>
        Update(func, arg, static (items, func, arg) => func(items, arg));

    TResult Update<T1, T2, TResult>(T1 arg1, T2 arg2, Func<IList<T>, T1, T2, TResult> func) =>
        Update(func, arg1, arg2, static (items, func, arg1, arg2) => func(items, arg1, arg2));

    TResult Update<T1, T2, T3, TResult>(T1 arg1, T2 arg2, T3 arg3, Func<IList<T>, T1, T2, T3, TResult> func)
    {
        lock (_lock)
        {
            _copy = null;
            var result = func(_items, arg1, arg2, arg3);
            return result;
        }
    }

    public int Count => Items.Count;
    public bool IsReadOnly => _items.IsReadOnly; // unprotected access is okay

    public void Add(T item)
    {
        Update(item, static (items, item) => items.Add(item));
        CollectionChanged?.Invoke(this, new(NotifyCollectionChangedAction.Add, item));
    }

    public void Clear()
    {
        Update(static items => items.Clear());
        CollectionChanged?.Invoke(this, new(NotifyCollectionChangedAction.Reset));
    }

    public bool Remove(T item)
    {
        var removed = Update(item, static (items, item) => items.Remove(item));
        if (removed)
            CollectionChanged?.Invoke(this, new(NotifyCollectionChangedAction.Remove, item));
        return removed;
    }

    public void Insert(int index, T item)
    {
        Update(index, item, static (items, index, item) => items.Insert(index, item));
        CollectionChanged?.Invoke(this, new(NotifyCollectionChangedAction.Add, item, index));
    }

    public void RemoveAt(int index)
    {
        var item = Update(index, static (items, index) =>
        {
            var item = items[index];
            items.RemoveAt(index);
            return item;
        });
        CollectionChanged?.Invoke(this, new(NotifyCollectionChangedAction.Remove, item, index));
    }

    public bool Contains(T item) => Items.Contains(item);
    public void CopyTo(T[] array, int arrayIndex) => Items.CopyTo(array, arrayIndex);
    public int IndexOf(T item) => Items.IndexOf(item);

    public T this[int index]
    {
        get => Items[index];
        set => Update(index, value, static (items, index, value) => items[index] = value);
    }

    IEnumerator IEnumerable.GetEnumerator() => GetEnumerator();
    public IEnumerator<T> GetEnumerator() => Items.GetEnumerator();
}
