#!/usr/bin/env bash
# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

set -e
cd "$(dirname "$0")"
cargo tarpaulin --ignore-tests --skip-clean --exclude-files "spikes/*" --exclude-files "examples/*" "$@"
