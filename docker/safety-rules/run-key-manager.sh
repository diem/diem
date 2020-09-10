#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# Example Usage:
# ./run-key-manager.sh <IMAGE> <CFG_CHAIN_ID> <CFG_JSON_RPC_ENDPOINT> <CFG_ROTATION_PERIOD_SECS> \
#   <CFG_VAULT_HOST> <CFG_VAULT_TOKEN> <STRUCT_LOGGER> <STRUCT_LOGGER_LOCATION>
#
# Note:
# (i) if arguments are not specified they will be assigned defaults.
# (ii) structured logger (STRUCT_LOGGER) must be one of STRUCT_LOG_FILE or STRUCT_LOG_TCP_ADDR, with
#      STRUCT_LOGGER_LOCATION set according to the chosen logger.

IMAGE="${1:-libra_safety_rules:latest}"
CFG_CHAIN_ID="${2:-TESTING}"
CFG_JSON_RPC_ENDPOINT="${3:-http://127.0.0.1:8080}"
CFG_ROTATION_PERIOD_SECS="${4:-3600}"
CFG_VAULT_HOST="${5:-http://127.0.0.1:8200}"
CFG_VAULT_TOKEN="${6:-root_token}"
STRUCT_LOGGER="${7:-STRUCT_LOG_TCP_ADDR}"
STRUCT_LOGGER_LOCATION="${8:-127.0.0.1:5044}"

docker network create --subnet 172.18.0.0/24 testnet || true

docker run \
    -e CFG_CHAIN_ID="$CFG_CHAIN_ID" \
    -e CFG_JSON_RPC_ENDPOINT="$CFG_JSON_RPC_ENDPOINT" \
    -e CFG_ROTATION_PERIOD_SECS="$CFG_ROTATION_PERIOD_SECS" \
    -e CFG_VAULT_HOST="$CFG_VAULT_HOST" \
    -e CFG_VAULT_TOKEN="$CFG_VAULT_TOKEN" \
    -e STRUCT_LOGGER="$STRUCT_LOGGER" \
    -e STRUCT_LOGGER_LOCATION="$STRUCT_LOGGER_LOCATION" \
    --ip 172.18.0.3 \
    --network testnet \
    "$IMAGE" \
    /key-manager.sh
