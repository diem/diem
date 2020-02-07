#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

image="${1:-libra_safety_rules:latest}"

docker network create --subnet 172.18.0.0/24 testnet || true

docker run \
    -e CFG_NODE_INDEX="0" \
    -e CFG_SAFETY_RULES_LISTEN_ADDR="0.0.0.0:8888" \
    --ip 172.18.0.2 \
    --network testnet \
    "$image"
