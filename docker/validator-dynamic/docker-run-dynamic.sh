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

/opt/libra/bin/config-builder validator \
    --data-dir /opt/libra/data/common \
    --output-dir /opt/libra/etc/ \
    ${params[@]}


if [ -n "${CFG_FULLNODE_SEED}" ]; then # We have a full node seed, add fullnode network
	declare -a fullnode_params
	    fullnode_params+="-s ${CFG_FULLNODE_SEED} "
	    fullnode_params+="-a /ip4/${CFG_LISTEN_ADDR}/tcp/6181 "
	    fullnode_params+="-l /ip4/0.0.0.0/tcp/6181 "
	    fullnode_params+="-b /ip4/127.0.0.1/tcp/6180 "
	    fullnode_params+="-n ${CFG_NUM_VALIDATORS} "
	    fullnode_params+="-f ${CFG_NUM_FULLNODES} "
	    fullnode_params+="-c ${CFG_FULLNODE_SEED} "


	/opt/libra/bin/config-builder full-node extend \
	    --data-dir /opt/libra/data/common \
	    --output-dir /opt/libra/etc/ \
	    ${fullnode_params[@]}

fi

exec /opt/libra/bin/libra-node -f /opt/libra/etc/node.config.toml
