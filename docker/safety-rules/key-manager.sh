#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# Parse parameters and execute config builder
declare -a params
if [ -n "${KEY_MANAGER_CONFIG}" ]; then # Path to key manager config
    echo "${KEY_MANAGER_CONFIG}" > /opt/libra/etc/key_manager.yaml
    params+="--template /opt/libra/etc/key_manager.yaml "
fi
if [ -n "${CFG_CHAIN_ID}" ]; then
        params+="--chain-id ${CFG_CHAIN_ID} "
fi
if [ -n "${CFG_JSON_RPC_ENDPOINT}" ]; then
    params+="--json-rpc-endpoint ${CFG_JSON_RPC_ENDPOINT} "
fi
if [ -n "${CFG_ROTATION_PERIOD_SECS}" ]; then
    params+="--rotation-period-secs ${CFG_ROTATION_PERIOD_SECS} "
fi
if [ -n "${CFG_SLEEP_PERIOD_SECS}" ]; then
    params+="--sleep-period-secs ${CFG_SLEEP_PERIOD_SECS} "
fi
if [ -n "${CFG_TXN_EXPIRATION_SECS}" ]; then
    params+="--txn-expiration-secs ${CFG_TXN_EXPIRATION_SECS} "
fi
if [ -n "${CFG_VAULT_HOST}" ]; then
    params+="--vault-host ${CFG_VAULT_HOST} "
fi
if [ -n "${CFG_VAULT_NAMESPACE}" ]; then
    params+="--vault-namespace ${CFG_VAULT_NAMESPACE} "
fi
if [ -n "${CFG_VAULT_TOKEN}" ]; then
    params+="--vault-token ${CFG_VAULT_TOKEN} "
fi

/opt/libra/bin/config-builder key-manager \
    --data-dir /opt/libra/data/common \
    --output-dir /opt/libra/etc/ \
    ${params[@]}

# Parse logger environment variables and execute the key manager
declare logger
if [ -n "${STRUCT_LOGGER}" ]; then
    if [ -n "${STRUCT_LOGGER_LOCATION}" ]; then
      logger="env ${STRUCT_LOGGER}=${STRUCT_LOGGER_LOCATION}"
    else
      echo "STRUCT_LOGGER has been set but STRUCT_LOGGER_LOCATION is not set!"
      exit 1
    fi
fi

exec ${logger} /opt/libra/bin/libra-key-manager /opt/libra/etc/key_manager.yaml
