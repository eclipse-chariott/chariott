#!/usr/bin/env bash
# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

set -e
# run a loop to oscillate temperature
(
    >&2 echo "PID(temp) = $BASHPID"
    while true; do
        for x in $(seq 18 26; seq 27 -1 17); do
            echo temp $x
            sleep 1
        done;
    done
) &
# run a loop that drains the battery
(
    >&2 echo "PID(battery) = $BASHPID"
    while true; do
        for x in $(seq 100 -1 0); do
            echo battery $x
            sleep 10
        done;
    done
) &
# the PID of each loop is printed to kill them if needed
