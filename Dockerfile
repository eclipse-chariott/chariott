# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

ARG RUST_VERSION=1.65
FROM rust:${RUST_VERSION} AS builder

# Dockerfile for building Eclipse Chariott runtime container
#
# This Dockerfile utilizes a two step build process. It builds Chariott with
# statically linked dependencies (using musl) for a x86_64 architecture.

# Chariott user id
ARG CHARIOTT_UID=10001

RUN apt update && apt upgrade -y
RUN apt install -y cmake protobuf-compiler

# unprivileged identity to run Chariott as
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${CHARIOTT_UID}" \
    chariott

WORKDIR /sdv

COPY ./ .

RUN rustup target add x86_64-unknown-linux-musl

RUN cargo build --release --target=x86_64-unknown-linux-musl -p service_discovery

####################################################################################################
## Final image
####################################################################################################
FROM alpine:latest

# Import Chariott user and group from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /sdv

# Copy our build
COPY --from=builder /sdv/target/x86_64-unknown-linux-musl/release/service_discovery /sdv/service_discovery

# Use the unprivileged chariott user during execution.
USER chariott:chariott

CMD ["./service_discovery"]
