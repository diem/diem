#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

docker network create --subnet 172.18.0.0/24 testnet || true

docker run \
    --cap-add=IPC_LOCK \
    -e 'VAULT_DEV_LISTEN_ADDRESS=0.0.0.0:8200' \
    -e 'VAULT_DEV_ROOT_TOKEN_ID=root_token' \
    --ip 172.18.0.3 \
    --network testnet \
    --publish 8200:8200 \
    --detach \
    vault
