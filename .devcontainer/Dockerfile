# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.231.5/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (buster breaks rust-analyzer extension #508)
ARG VARIANT="bullseye"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends git-lfs cmake protobuf-compiler libprotobuf-dev

RUN mkdir /usr/bin/grpcurl.d \
    && curl -sSL https://github.com/fullstorydev/grpcurl/releases/download/v1.8.7/grpcurl_1.8.7_linux_x86_64.tar.gz | tar -xvz --directory /usr/bin/grpcurl.d \
    && ln /usr/bin/grpcurl.d/grpcurl /usr/bin/grpcurl

RUN PREFIX="/usr/local" && \
    VERSION="1.8.0" && \
    curl -sSL \
    "https://github.com/bufbuild/buf/releases/download/v${VERSION}/buf-$(uname -s)-$(uname -m).tar.gz" | \
    tar -xvzf - -C "${PREFIX}" --strip-components 1

RUN curl -sSL https://dot.net/v1/dotnet-install.sh > /usr/bin/dotnet-install && \
    chmod +x /usr/bin/dotnet-install

USER vscode

ARG DOTNET_VERSION="7.0.100"
RUN dotnet-install --install-dir "$HOME/.dotnet" --version "${DOTNET_VERSION}" && \
    dotnet-install --install-dir "$HOME/.dotnet" --version 6.0.11 --runtime aspnetcore && \
    export PATH="$PATH:$HOME/.dotnet"

# We rely on the base image of the Rust Devcontainer for OS and security updates
# but want to have control over the version of Rust used. We do this by
# overriding the version of the base image with the one specified in the
# rust-toolchain file.
#
# NB: changing the contents of the rust-toolchain file will not cause VSCode to
# prompt to rebuild the container nor the Docker build cache to be invalidated.
# Make sure to rebuild the container to ensure a fresh install of the new Rust
# version also works.
COPY ../rust-toolchain.toml .

# NB: we have to install also any components we need for our specific toolchain
# version.
RUN rustup component add clippy && \
    rustup component add rustfmt
