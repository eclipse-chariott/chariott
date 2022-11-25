// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

using System;
using System.IO;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Google.Protobuf;

namespace CarBridgeCloudApp;

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
