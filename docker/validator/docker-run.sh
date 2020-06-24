#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

cd /opt/libra/etc
echo "$NODE_CONFIG" > node.yaml
echo "$SEED_PEERS" > seed_peers.yaml
echo "$NETWORK_KEYPAIRS" > network_keypairs.yaml
echo "$CONSENSUS_KEYPAIR" > consensus_keypair.yaml
echo "$FULLNODE_KEYPAIRS" > fullnode_keypairs.yaml
exec /opt/libra/bin/libra-node -f node.yaml
