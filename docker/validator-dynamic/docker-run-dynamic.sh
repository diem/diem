#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

declare -a params
if [ -n "${CFG_BASE_CONFIG}" ]; then # Path to base config
	    params+="-t ${CFG_BASE_CONFIG} "
fi
if [ -n "${CFG_LISTEN_ADDR}" ]; then # Advertised listen address for network config
	    params+="-a /ip4/${CFG_LISTEN_ADDR}/tcp/6180 "
fi
if [ -n "${CFG_LISTEN_ADDR}" ]; then # Listen address for node
	    params+="-l /ip4/0.0.0.0/tcp/6180 "
fi
if [ -n "${CFG_NODE_INDEX}" ]; then
	    params+="-i ${CFG_NODE_INDEX} "
fi
if [ -n "${CFG_NUM_VALIDATORS}" ]; then # Total number of nodes in this network
	    params+="-n ${CFG_NUM_VALIDATORS} "
fi
if [ -n "${CFG_SEED}" ]; then # Random seed to use
	    params+="-s ${CFG_SEED} "
fi
if [ -n "${CFG_SEED_PEER_IP}" ]; then # Seed peer ip for discovery
	    params+="--bootstrap /ip4/${CFG_SEED_PEER_IP}/tcp/6180 "
fi

/opt/libra/bin/validator-config-builder \
    --data-dir /opt/libra/data/common \
    --output-dir /opt/libra/etc/ \
    ${params[@]}

exec /opt/libra/bin/libra-node -f /opt/libra/etc/node.config.toml
