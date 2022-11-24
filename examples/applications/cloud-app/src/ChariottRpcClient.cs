// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

using System;
using System.Collections.Generic;
using System.Linq;
using System.Net.Http.Headers;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Chariott.Common.V1;
using Chariott.Runtime.V1;
using Google.Protobuf;
using MoreLinq;
using MQTTnet;
using MQTTnet.Client;
using MQTTnet.Protocol;

namespace CarBridgeCloudApp;

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
            var taskCompletionSource = new TaskCompletionSource<FulfillResponse>(TaskCreationOptions.RunContinuationsAsynchronously);

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
