#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -ex

IMAGE="${1:-libra_e2e:latest}"
CONFIGDIR="$(dirname "$0")/../../terraform/validator-sets/dev/"

SEED_PEERS="$(sed 's,SEED_IP,172.18.0.10,' $CONFIGDIR/seed_peers.config.toml)"
GENESIS_BLOB="$PWD/$CONFIGDIR/genesis.blob"
CONSENSUS_PEERS="$PWD/$CONFIGDIR/consensus_peers.config.toml"
CONFIGDIR="$CONFIGDIR/val"
NETWORK_PEERS="$PWD/$CONFIGDIR/network_peers.config.toml"
docker network create --subnet 172.18.0.0/24 testnet || true

docker run -e NODE_CONFIG="$(sed 's,\${self_ip},172.18.0.10,;s,\${peer_id},8deeeaed65f0cd7484a9e4e5ac51fbac548f2f71299a05e000156031ca78fb9f,' $CONFIGDIR/node.config.toml)" -e SEED_PEERS="$SEED_PEERS" -v "$NETWORK_PEERS:/opt/libra/etc/network_peers.config.toml" -v "$CONSENSUS_PEERS:/opt/libra/etc/consensus_peers.config.toml" -v "$GENESIS_BLOB:/opt/libra/etc/genesis.blob" -e NETWORK_KEYPAIRS="$(cat $CONFIGDIR/8deeeaed65f0cd7484a9e4e5ac51fbac548f2f71299a05e000156031ca78fb9f.node.network.keys.toml)" -e CONSENSUS_KEYPAIR="$(cat $CONFIGDIR/8deeeaed65f0cd7484a9e4e5ac51fbac548f2f71299a05e000156031ca78fb9f.node.consensus.keys.toml)" --ip 172.18.0.10 --network testnet --detach "$IMAGE"

docker run -e NODE_CONFIG="$(sed 's,\${self_ip},172.18.0.11,;s,\${peer_id},1e5d5a74b0fd09f601ac0fca2fe7d213704e02e51943d18cf25a546b8416e9e1,' $CONFIGDIR/node.config.toml)" -e SEED_PEERS="$SEED_PEERS" -v "$NETWORK_PEERS:/opt/libra/etc/network_peers.config.toml" -v "$CONSENSUS_PEERS:/opt/libra/etc/consensus_peers.config.toml" -v "$GENESIS_BLOB:/opt/libra/etc/genesis.blob" -e NETWORK_KEYPAIRS="$(cat $CONFIGDIR/1e5d5a74b0fd09f601ac0fca2fe7d213704e02e51943d18cf25a546b8416e9e1.node.network.keys.toml)" -e CONSENSUS_KEYPAIR="$(cat $CONFIGDIR/1e5d5a74b0fd09f601ac0fca2fe7d213704e02e51943d18cf25a546b8416e9e1.node.consensus.keys.toml)" --ip 172.18.0.11 --network testnet --detach "$IMAGE"

docker run -e NODE_CONFIG="$(sed 's,\${self_ip},172.18.0.12,;s,\${peer_id},ab0d6a54ce9d7fc79c061f95883a308f9bdfc987262b6a34a360fdd788fcd9cd,' $CONFIGDIR/node.config.toml)" -e SEED_PEERS="$SEED_PEERS" -v "$NETWORK_PEERS:/opt/libra/etc/network_peers.config.toml" -v "$CONSENSUS_PEERS:/opt/libra/etc/consensus_peers.config.toml" -v "$GENESIS_BLOB:/opt/libra/etc/genesis.blob" -e NETWORK_KEYPAIRS="$(cat $CONFIGDIR/ab0d6a54ce9d7fc79c061f95883a308f9bdfc987262b6a34a360fdd788fcd9cd.node.network.keys.toml)" -e CONSENSUS_KEYPAIR="$(cat $CONFIGDIR/ab0d6a54ce9d7fc79c061f95883a308f9bdfc987262b6a34a360fdd788fcd9cd.node.consensus.keys.toml)" --ip 172.18.0.12 --network testnet --detach "$IMAGE"

docker run -e NODE_CONFIG="$(sed 's,\${self_ip},172.18.0.13,;s,\${peer_id},57ff83747054695f2228042c26eb6a243ac73de1b9038aea103999480b076d45,' $CONFIGDIR/node.config.toml)" -e SEED_PEERS="$SEED_PEERS" -v "$NETWORK_PEERS:/opt/libra/etc/network_peers.config.toml" -v "$CONSENSUS_PEERS:/opt/libra/etc/consensus_peers.config.toml" -v "$GENESIS_BLOB:/opt/libra/etc/genesis.blob" -e NETWORK_KEYPAIRS="$(cat $CONFIGDIR/57ff83747054695f2228042c26eb6a243ac73de1b9038aea103999480b076d45.node.network.keys.toml)" -e CONSENSUS_KEYPAIR="$(cat $CONFIGDIR/57ff83747054695f2228042c26eb6a243ac73de1b9038aea103999480b076d45.node.consensus.keys.toml)" --ip 172.18.0.13 --network testnet --publish 8000:8000 "$IMAGE"
