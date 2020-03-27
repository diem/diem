#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

declare -a params
if [ -n "${CFG_BASE_CONFIG}" ]; then # Path to base config
	    echo "${CFG_BASE_CONFIG}" > /opt/libra/etc/base.config.toml
	    params+="-t /opt/libra/etc/base.config.toml "
fi
if [ -n "${CFG_LISTEN_ADDR}" ]; then # Advertised listen address for network config
	    params+="-a /ip4/${CFG_LISTEN_ADDR}/tcp/6180 "
fi
if [ -n "${CFG_LISTEN_ADDR}" ]; then # Listen address for node
	    params+="-l /ip4/0.0.0.0/tcp/6180 "
fi
if [ -n "${CFG_FULLNODE_INDEX}" ]; then
	    params+="-i ${CFG_FULLNODE_INDEX} "
fi
if [ -n "${CFG_NUM_VALIDATORS}" ]; then # Total number of nodes in this network
	    params+="-n ${CFG_NUM_VALIDATORS} "
fi
if [ -n "${CFG_SEED}" ]; then # Random seed to use for validator network
	    params+="-s ${CFG_SEED} "
fi
if [ -n "${CFG_FULLNODE_SEED}" ]; then # Random seed to use for fullnode network
	    params+="-c ${CFG_FULLNODE_SEED} "
fi
if [ -n "${CFG_SEED_PEER_IP}" ]; then # Seed peer ip for discovery
	    params+="--bootstrap /ip4/${CFG_SEED_PEER_IP}/tcp/6181 "
fi
if [ -n "${CFG_NUM_FULLNODES}" ]; then # Random seed to use for fullnode network
	    params+="-f ${CFG_NUM_FULLNODES} "
fi


/opt/libra/bin/config-builder full-node create \
	--data-dir /opt/libra/data/common \
	--output-dir /opt/libra/etc/ \
	${params[@]}

# Set CFG_OVERRIDES to any values that you want to override in the config
# Example: CFG_OVERRIDES='grpc_max_receive_len=45,genesis_file_location="genesis2.blob",max_block_size=250'
# Note: Double quotes are required for string parameters and should be
# omitted for int parameters.
# Note: This currently does not work with parameters which are repeated.
if [ -n "${CFG_OVERRIDES}" ]; then
  IFS=',' read -ra CFG_OVERRIDES <<< "$CFG_OVERRIDES"
  for KEY_VAL in "${CFG_OVERRIDES[@]}"; do
    IFS='=' read -ra KEY_VAL <<< "$KEY_VAL"
      KEY="${KEY_VAL[0]}"
      VAL="${KEY_VAL[1]}"
      echo "Overriding ${KEY} = ${VAL} in node config"
      sed "s/^${KEY} = .*/${KEY} = ${VAL}/g" /opt/libra/etc/node.config.toml > /tmp/node.config.toml
      mv /tmp/node.config.toml /opt/libra/etc/node.config.toml
  done
fi

exec /opt/libra/bin/libra-node -f /opt/libra/etc/node.config.toml
