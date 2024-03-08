#!/bin/bash

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT license.
# SPDX-License-Identifier: MIT

# Exits immediately on failure.
set -eu

# Function to display usage information
usage() {
    echo "Usage: $0 [-a|--arg-value] <ARGUMENT_VALUE> [-r|--regex] <ACCEPTED_REGEX>"
    echo "Example:"
    echo "  $0 -a \"\${APP_NAME}\" -r \"^[a-zA-Z_0-9-]+$\""
}

# Parse command line arguments
while [[ $# -gt 0 ]]
do
    key="$1"

    case $key in
        -a|--arg-value)
            arg_value="$2"
            shift # past argument
            shift # past value
            ;;
        -r|--regex)
            regex="$2"
            shift # past argument
            shift # past value
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $key"
            usage
            exit 1
    esac
done

# Check if all required arguments have been set
if [[ -z "${arg_value}" || -z "${regex}" ]]; then
    echo "Error: Missing required arguments:"
    [[ -z "${arg_value}" ]] && echo "  -a|--arg-value"
    [[ -z "${regex}" ]] && echo "  -r|--regex"
    echo -e "\n"
    usage
    exit 1
fi

sanitized=$(echo "${arg_value}" | tr -dc "${regex}");
[ "$sanitized" = "${arg_value}" ] || {
    echo "ARG is invalid. ARG='${arg_value}' sanitized='${sanitized}'";
    exit 1
}

echo -e "\nARG with value '${arg_value}' is sanitized"
exit 0