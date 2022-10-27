#!/usr/bin/env bash
# Copyright (c) Microsoft Corporation. All rights reserved.
# Licensed under the MIT license.

set -e
cd "$(dirname "$0")"
cargo tarpaulin --ignore-tests --skip-clean --exclude-files "spikes/*" --exclude-files "examples/*" "$@"
