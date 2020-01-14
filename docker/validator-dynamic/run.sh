#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

image="${1:-libra_validator_dynamic:latest}"
nodes="5"
base_ip="172.18.0"
ip_offset="10"
bootstrap="$base_ip.$ip_offset"

docker network create --subnet 172.18.0.0/24 testnet || true

for ((node=1; node<$nodes; node++)); do
    nodes_ip_offset="$(expr $ip_offset + $node)"
    node_ip="$base_ip.$nodes_ip_offset"
    docker run \
        -e CFG_LISTEN_ADDR="$node_ip" \
        -e CFG_NODE_INDEX=$node \
        -e CFG_NUM_VALIDATORS="$nodes" \
        -e CFG_SEED_PEER_IP="$bootstrap" \
        --ip $node_ip \
        --network testnet \
        --detach \
        "$image"
done

docker run \
    -e CFG_LISTEN_ADDR="$bootstrap" \
    -e CFG_NODE_INDEX="0" \
    -e CFG_NUM_VALIDATORS="$nodes" \
    -e CFG_SEED_PEER_IP="$bootstrap" \
    --ip $bootstrap \
    --network testnet \
    --publish 8000:8000 \
    --publish 9101:9101 \
    --publish 9102:9102 \
    "$image"
