#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

cd /opt/libra/etc
[ -n "${NODE_CONFIG}" ] && echo "$NODE_CONFIG" > node.yaml
[ -n "${SEED_PEERS}" ] && echo "$SEED_PEERS" > seed_peers.yaml
[ -n "${NETWORK_KEYPAIRS}" ] && echo "$NETWORK_KEYPAIRS" > network_keypairs.yaml
[ -n "${CONSENSUS_KEYPAIR}" ] && echo "$CONSENSUS_KEYPAIR" > consensus_keypair.yaml
[ -n "${FULLNODE_KEYPAIRS}" ] && echo "$FULLNODE_KEYPAIRS" > fullnode_keypairs.yaml
exec /opt/libra/bin/libra-node -f node.yaml
