// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

using System;

namespace CarBridgeCloudApp;

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
