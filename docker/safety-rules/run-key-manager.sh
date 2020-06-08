#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# Example Usage (arguments that are not specified will be assigned defaults):
# ./run-key-manager.sh <IMAGE> <JSON_RPC_ENDPOINT> <ROTATION_PERIOD_SECS> <VALIDATOR_ACCOUNT> <VAULT_HOST> <VAULT_TOKEN>

IMAGE="${1:-libra_safety_rules:latest}"
JSON_RPC_ENDPOINT="${2:-http://127.0.0.1:8080}"
ROTATION_PERIOD_SECS="${3:-3600}"
VAULT_HOST="${4:-http://127.0.0.1:8200}"
VAULT_TOKEN="${5:-root_token}"

docker network create --subnet 172.18.0.0/24 testnet || true

docker run \
    -e JSON_RPC_ENDPOINT="$JSON_RPC_ENDPOINT" \
    -e ROTATION_PERIOD_SECS="$ROTATION_PERIOD_SECS" \
    -e VAULT_HOST="$VAULT_HOST" \
    -e VAULT_TOKEN="$VAULT_TOKEN" \
    --ip 172.18.0.3 \
    --network testnet \
    "$IMAGE" \
    /key-manager.sh
