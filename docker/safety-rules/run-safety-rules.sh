#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# Example Usage:
# ./run-safety-rules.sh <IMAGE> <CFG_NODE_INDEX> <CFG_SAFETY_RULES_LISTEN_ADDR> <STRUCT_LOGGER> <STRUCT_LOGGER_LOCATION>
#
# Note:
# (i) if arguments are not specified they will be assigned defaults.
# (ii) structured logger (STRUCT_LOGGER) must be one of STRUCT_LOG_FILE or STRUCT_LOG_UDP_ADDR, with
#      STRUCT_LOGGER_LOCATION set according to the chosen logger.


IMAGE="${1:-libra_safety_rules:latest}"
CFG_NODE_INDEX="${2:-0}"
CFG_SAFETY_RULES_LISTEN_ADDR="${3:-0.0.0.0:8888}"
STRUCT_LOGGER="${4:-STRUCT_LOG_UDP_ADDR}"
STRUCT_LOGGER_LOCATION="${5:-127.0.0.1:5044}"

docker network create --subnet 172.18.0.0/24 testnet || true

docker run \
    -e CFG_NODE_INDEX="$CFG_NODE_INDEX" \
    -e CFG_SAFETY_RULES_LISTEN_ADDR="$CFG_SAFETY_RULES_LISTEN_ADDR" \
    -e STRUCT_LOGGER="$STRUCT_LOGGER" \
    -e STRUCT_LOGGER_LOCATION="$STRUCT_LOGGER_LOCATION" \
    --ip 172.18.0.2 \
    --network testnet \
    "$IMAGE"
