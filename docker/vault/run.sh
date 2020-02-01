#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

docker run \
    --cap-add=IPC_LOCK \
    -e 'VAULT_DEV_LISTEN_ADDRESS=0.0.0.0:8200' \
    -e 'VAULT_DEV_ROOT_TOKEN_ID=root_token' \
    --publish 8200:8200 \
    --detach \
    vault
