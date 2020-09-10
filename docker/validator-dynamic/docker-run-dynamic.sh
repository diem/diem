#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

declare -a params
if [ -n "${CFG_BASE_CONFIG}" ]; then # Path to base config
	    echo "${CFG_BASE_CONFIG}" > /opt/libra/etc/base.yaml
	    params+="-t /opt/libra/etc/base.yaml "
fi
if [ -n "${CFG_CHAIN_ID}" ]; then
        params+="--chain-id ${CFG_CHAIN_ID} "
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
if [ -n "${CFG_NUM_VALIDATORS_IN_GENESIS}" ]; then # Total number of nodes in genesis
	    params+="-g ${CFG_NUM_VALIDATORS_IN_GENESIS} "
fi
if [ -n "${CFG_SEED}" ]; then # Random seed to use
	    params+="-s ${CFG_SEED} "
fi
if [ -n "${CFG_SEED_PEER_IP}" ]; then # Seed peer ip for discovery
	    params+="--bootstrap /ip4/${CFG_SEED_PEER_IP}/tcp/6180 "
fi
if [ -n "${CFG_SAFETY_RULES_ADDR}" ]; then
    params+="--safety-rules-addr ${CFG_SAFETY_RULES_ADDR} "
fi

/opt/libra/bin/config-builder validator \
    --data-dir /opt/libra/data \
    --output-dir /opt/libra/etc/ \
    ${params[@]}


if [[ -n "${CFG_FULLNODE_SEED}" && ("${CFG_ENABLE_MGMT_TOOL}" = false || -z "${CFG_ENABLE_MGMT_TOOL}") ]]; then
	declare -a fullnode_params
	    fullnode_params+="-s ${CFG_FULLNODE_SEED} "
	    fullnode_params+="-a /ip4/${CFG_LISTEN_ADDR}/tcp/6181 "
	    fullnode_params+="-l /ip4/0.0.0.0/tcp/6181 "
	    fullnode_params+="-b /ip4/127.0.0.1/tcp/6180 "
	    fullnode_params+="-n ${CFG_NUM_VALIDATORS} "
	    fullnode_params+="-f ${CFG_NUM_FULLNODES} "
	    fullnode_params+="-c ${CFG_FULLNODE_SEED} "

	/opt/libra/bin/config-builder full-node extend \
	    --data-dir /opt/libra/data \
	    --output-dir /opt/libra/etc/ \
	    ${fullnode_params[@]}

fi

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
      sed "s/^  ${KEY}:.*/  ${KEY}: ${VAL}/g" /opt/libra/etc/node.yaml > /tmp/node.yaml
      mv /tmp/node.yaml /opt/libra/etc/node.yaml
  done
fi

exec /opt/libra/bin/libra-node -f /opt/libra/etc/node.yaml
