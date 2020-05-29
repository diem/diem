#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# Example Usage (arguments that are not specified will be assigned defaults):
# ./run-safety-rules.sh <IMAGE> <CFG_NODE_INDEX> <CFG_SAFETY_RULES_LISTEN_ADDR>

IMAGE="${1:-libra_safety_rules:latest}"
CFG_NODE_INDEX="${2:-0}"
CFG_SAFETY_RULES_LISTEN_ADDR="${3:-0.0.0.0:8888}"

docker network create --subnet 172.18.0.0/24 testnet || true

docker run \
    -e CFG_NODE_INDEX="$CFG_NODE_INDEX" \
    -e CFG_SAFETY_RULES_LISTEN_ADDR="$CFG_SAFETY_RULES_LISTEN_ADDR" \
    --ip 172.18.0.2 \
    --network testnet \
    "$IMAGE"
