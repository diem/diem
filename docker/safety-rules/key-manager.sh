#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

declare -a params
if [ -n "${CFG_BASE_CONFIG}" ]; then # Path to base config
    echo "${CFG_BASE_CONFIG}" > /opt/libra/etc/base.config.toml
    params+="--template /opt/libra/etc/base.config.toml "
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

exec /opt/libra/bin/libra-key-manager /opt/libra/etc/key_manager.config.toml
