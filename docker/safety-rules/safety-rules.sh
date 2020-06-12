#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# Parse parameters and execute config builder
declare -a params
if [ -n "${CFG_BASE_CONFIG}" ]; then # Path to base config
	    echo "${CFG_BASE_CONFIG}" > /opt/libra/etc/base.config.toml
	    params+="-t /opt/libra/etc/base.config.toml "
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
if [ -n "${CFG_SAFETY_RULES_LISTEN_ADDR}" ]; then
    params+="--safety-rules-addr ${CFG_SAFETY_RULES_LISTEN_ADDR} "
fi
if [ -n "${CFG_SAFETY_RULES_BACKEND}" ]; then
    params+="--safety-rules-backend ${CFG_SAFETY_RULES_BACKEND} "
fi
if [ -n "${CFG_SAFETY_RULES_HOST}" ]; then
    params+="--safety-rules-host ${CFG_SAFETY_RULES_HOST} "
fi
if [ -n "${CFG_SAFETY_RULES_TOKEN}" ]; then
    params+="--safety-rules-token ${CFG_SAFETY_RULES_TOKEN} "
fi
if [ -n "${CFG_SAFETY_RULES_NAMESPACE}" ]; then
    params+="--safety-rules-namespace ${CFG_SAFETY_RULES_NAMESPACE} "
fi

/opt/libra/bin/config-builder safety-rules \
    --data-dir /opt/libra/data/common \
    --output-dir /opt/libra/etc/ \
    ${params[@]}

# Parse logger environment variables and execute safety rules
declare logger
if [ -n "${STRUCT_LOGGER}" ]; then
    if [ -n "${STRUCT_LOGGER_LOCATION}" ]; then
      logger="env ${STRUCT_LOGGER}=${STRUCT_LOGGER_LOCATION}"
    else
      echo "STRUCT_LOGGER has been set but STRUCT_LOGGER_LOCATION is not set!"
      exit 1
    fi
fi

exec ${logger} /opt/libra/bin/safety-rules /opt/libra/etc/node.config.toml
