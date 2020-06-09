#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# Parse parameters and execute config builder
declare -a params
if [ -n "${KEY_MANAGER_CONFIG}" ]; then # Path to key manager config
    echo "${KEY_MANAGER_CONFIG}" > /opt/libra/etc/key_manager.config.toml
    params+="--template /opt/libra/etc/key_manager.config.toml "
fi
if [ -n "${JSON_RPC_ENDPOINT}" ]; then
    params+="--json-rpc-endpoint ${JSON_RPC_ENDPOINT} "
fi
if [ -n "${ROTATION_PERIOD_SECS}" ]; then
    params+="--rotation-period-secs ${ROTATION_PERIOD_SECS} "
fi
if [ -n "${SLEEP_PERIOD_SECS}" ]; then
    params+="--sleep-period-secs ${SLEEP_PERIOD_SECS} "
fi
if [ -n "${TXN_EXPIRATION_SECS}" ]; then
    params+="--txn-expiration-secs ${TXN_EXPIRATION_SECS} "
fi
if [ -n "${VAULT_HOST}" ]; then
    params+="--vault-host ${VAULT_HOST} "
fi
if [ -n "${VAULT_TOKEN}" ]; then
    params+="--vault-token ${VAULT_TOKEN} "
fi
if [ -n "${VAULT_NAMESPACE}" ]; then
    params+="--vault-namespace ${VAULT_NAMESPACE} "
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

exec ${logger} /opt/libra/bin/libra-key-manager /opt/libra/etc/key_manager.config.toml
