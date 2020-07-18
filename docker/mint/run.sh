#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Example Usage:
# ./run.sh <IMAGE> <AC_HOST> <AC_PORT> <LOG_LEVEL>
# ./run.sh libra_mint:latest ac.dev.aws.hlw3truzy4ls.com 80 info

set -ex

IMAGE="${1:-libra_mint:latest}"
AC_HOST="${2:-172.18.0.10}"
AC_PORT="${3:-8080}"
CFG_CHAIN_ID="${4:-TESTING}"
LOG_LEVEL="${5:-info}"
nodes="5"

docker network create --subnet 172.18.0.0/24 testnet || true

docker run \
    -e AC_HOST="$AC_HOST" \
    -e AC_PORT="$AC_PORT" \
    -e CFG_CHAIN_ID="$CFG_CHAIN_ID" \
    -e LOG_LEVEL="$LOG_LEVEL" \
    -e CFG_NUM_VALIDATORS="$nodes" \
	--network testnet --publish 9080:8000 "$IMAGE"
