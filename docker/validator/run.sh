#!/bin/sh
set -ex

IMAGE="${1:-libra_e2e:latest}"
CONFIGDIR="$(dirname "$0")/../../terraform/validator-sets/dev/"

SEED_PEERS="$(sed 's,SEED_IP,172.18.0.10,' $CONFIGDIR/seed_peers.config.toml)"
GENESIS_BLOB="$(base64 $CONFIGDIR/genesis.blob)"
TRUSTED_PEERS="$PWD/$CONFIGDIR/trusted_peers.config.toml"

docker network create --subnet 172.18.0.0/24 testnet || true

docker run -e PEER_ID=8deeeaed65f0cd7484a9e4e5ac51fbac548f2f71299a05e000156031ca78fb9f -e NODE_CONFIG="$(sed 's,\${self_ip},172.18.0.10,' $CONFIGDIR/node.config.toml)" -e SEED_PEERS="$SEED_PEERS" -v "$TRUSTED_PEERS:/opt/libra/etc/trusted_peers.config.toml" -e GENESIS_BLOB="$GENESIS_BLOB" -e PEER_KEYPAIRS="$(cat $CONFIGDIR/8deeeaed65f0cd7484a9e4e5ac51fbac548f2f71299a05e000156031ca78fb9f.node.keys.toml)" --ip 172.18.0.10 --network testnet --detach "$IMAGE"
docker run -e PEER_ID=1e5d5a74b0fd09f601ac0fca2fe7d213704e02e51943d18cf25a546b8416e9e1 -e NODE_CONFIG="$(sed 's,\${self_ip},172.18.0.11,' $CONFIGDIR/node.config.toml)" -e SEED_PEERS="$SEED_PEERS" -v "$TRUSTED_PEERS:/opt/libra/etc/trusted_peers.config.toml" -e GENESIS_BLOB="$GENESIS_BLOB" -e PEER_KEYPAIRS="$(cat $CONFIGDIR/1e5d5a74b0fd09f601ac0fca2fe7d213704e02e51943d18cf25a546b8416e9e1.node.keys.toml)" --ip 172.18.0.11 --network testnet --detach "$IMAGE"
docker run -e PEER_ID=ab0d6a54ce9d7fc79c061f95883a308f9bdfc987262b6a34a360fdd788fcd9cd -e NODE_CONFIG="$(sed 's,\${self_ip},172.18.0.12,' $CONFIGDIR/node.config.toml)" -e SEED_PEERS="$SEED_PEERS" -v "$TRUSTED_PEERS:/opt/libra/etc/trusted_peers.config.toml" -e GENESIS_BLOB="$GENESIS_BLOB" -e PEER_KEYPAIRS="$(cat $CONFIGDIR/ab0d6a54ce9d7fc79c061f95883a308f9bdfc987262b6a34a360fdd788fcd9cd.node.keys.toml)" --ip 172.18.0.12 --network testnet --detach "$IMAGE"
docker run -e PEER_ID=57ff83747054695f2228042c26eb6a243ac73de1b9038aea103999480b076d45 -e NODE_CONFIG="$(sed 's,\${self_ip},172.18.0.13,' $CONFIGDIR/node.config.toml)" -e SEED_PEERS="$SEED_PEERS" -v "$TRUSTED_PEERS:/opt/libra/etc/trusted_peers.config.toml" -e GENESIS_BLOB="$GENESIS_BLOB" -e PEER_KEYPAIRS="$(cat $CONFIGDIR/57ff83747054695f2228042c26eb6a243ac73de1b9038aea103999480b076d45.node.keys.toml)" --ip 172.18.0.13 --network testnet --publish 8000:8000 "$IMAGE"
