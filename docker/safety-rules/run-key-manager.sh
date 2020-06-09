#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# Example Usage:
# ./run-key-manager.sh <IMAGE> <JSON_RPC_ENDPOINT> <ROTATION_PERIOD_SECS> <STRUCT_LOGGER> <STRUCT_LOGGER_LOCATION> <VAULT_HOST> <VAULT_TOKEN>
#
# Note:
# (i) if arguments are not specified they will be assigned defaults.
# (ii) structured logger (STRUCT_LOGGER) must be one of STRUCT_LOG_FILE or STRUCT_LOG_UDP_ADDR, with
#      STRUCT_LOGGER_LOCATION set according to the chosen logger.

IMAGE="${1:-libra_safety_rules:latest}"
JSON_RPC_ENDPOINT="${2:-http://127.0.0.1:8080}"
ROTATION_PERIOD_SECS="${3:-3600}"
STRUCT_LOGGER="${4:-STRUCT_LOG_UDP_ADDR}"
STRUCT_LOGGER_LOCATION="${5:-127.0.0.1:5044}"
VAULT_HOST="${6:-http://127.0.0.1:8200}"
VAULT_TOKEN="${7:-root_token}"

docker network create --subnet 172.18.0.0/24 testnet || true

docker run \
    -e JSON_RPC_ENDPOINT="$JSON_RPC_ENDPOINT" \
    -e ROTATION_PERIOD_SECS="$ROTATION_PERIOD_SECS" \
    -e STRUCT_LOGGER="$STRUCT_LOGGER" \
    -e STRUCT_LOGGER_LOCATION="$STRUCT_LOGGER_LOCATION" \
    -e VAULT_HOST="$VAULT_HOST" \
    -e VAULT_TOKEN="$VAULT_TOKEN" \
    --ip 172.18.0.3 \
    --network testnet \
    "$IMAGE" \
    /key-manager.sh
