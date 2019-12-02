#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

IMAGE="${1:-libra_dynamic:latest}"
export NODES="5"
BASE_IP="10"
export LISTEN="/ip4/0.0.0.0/tcp/1025"
export BOOTSTRAP="/ip4/172.18.0.$BASE_IP/tcp/1025"

docker network create --subnet 172.18.0.0/24 testnet || true

for ((node=1; node<$NODES; node++)); do
    ip_offset="$(expr $BASE_IP + $node)"
    docker run \
        -e IPV4="true" \
        -e BOOTSTRAP="$BOOTSTRAP" \
        -e LISTEN="$LISTEN" \
        -e NODE=$node \
        -e NODES="$NODES" \
        --ip "172.18.0.$ip_offset" \
        --network testnet \
        --detach \
        "$IMAGE"
done

docker run \
    -e IPV4="true" \
    -e LISTEN="$LISTEN" \
    -e NODE="0" \
    -e NODES="$NODES" \
    --ip "172.18.0.$BASE_IP" \
    --network testnet \
    --publish 8000:8000 \
    "$IMAGE"
