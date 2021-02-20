#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

# Colors
LBLUE='\033[1;34m'
NC='\033[0m' # No Color

# # # # # # # # # # # # # # # # # # # #
#       Create keys + addresses       #
# # # # # # # # # # # # # # # # # # # #

echo -e "\n${LBLUE}Creating Keys + Addresses${NC} "

KEYFILE="$DIR/private_keys.json"
touch "$KEYFILE"

# THIS IS FOR TESTING ONLY: THESE KEYS SHOULD NOT BE USED IN A PRODUCTION ENVIRONMENT!
create_key() {
  NAME=${1}
  # SEED=${2}
  NOW=$(date +%s)
  KEY=$(cargo run -p swiss-knife -- generate-test-ed25519-keypair --seed 0)
  echo "$KEY" | jq -c ".data | { ${NAME}: { last_update:${NOW}, value: .private_key }, ${NAME}_account: { last_update:${NOW}, value: .diem_account_address } }"
}

extend_keyfile() {
  MERGED=$(cat - | cat - "$KEYFILE" | jq --sort-keys -s add)
  echo "$MERGED" >"$KEYFILE"
}

# Create all of the private key jsons and add them to the KEYFILE json (along with their block address)
create_key diem_root | extend_keyfile
create_key operator | extend_keyfile
create_key owner | extend_keyfile
create_key treasury_compliance | extend_keyfile
create_key consensus | extend_keyfile
create_key execution | extend_keyfile
create_key fullnode_network | extend_keyfile
create_key validator_network | extend_keyfile

# Add the special value for `validator_network_address_keys`
(
  cat <<EOF
  {
    "validator_network_address_keys": {
      "last_update": $(date +%s),
      "value": {
        "current": 0,
        "keys": {
          "0": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
        }
      }
    }
  }
EOF
) | extend_keyfile

# Add initial empty safety data
(
  cat <<EOF
  {
    "safety_data": {
      "data": "GetResponse",
      "last_update": 1613773160,
      "value": {
        "epoch": 0,
        "last_vote": null,
        "last_voted_round": 0,
        "preferred_round": 0
      }
    }
  }
EOF
) | extend_keyfile


# # # # # # # # # # # # # # # # # # # #
#  Run config + create genesis block  #
# # # # # # # # # # # # # # # # # # # #

echo -e "\n${LBLUE}Creating Shared Config${NC} "

export NAME="testing"
export CHAIN_ID="TESTING"

export PRIVATE_KEYFILE="$DIR/private_keys.json"
export MINT_KEYFILE="$DIR/mint.key"

export GENESIS_BLOBFILE="$DIR/genesis.blob"

export WAYPOINT_FILE="$DIR/waypoint.txt"

export CONFIG_FILE="$DIR/shared_config.json"

export LAYOUT_FILE="$(mktemp)"

cat >"$LAYOUT_FILE" <<EOF
owners = ["$NAME"]
operators = ["$NAME"]
diem_root = "diem"
treasury_compliance = "diem"
EOF

# Set up the shared config
cargo run -p diem-genesis-tool -- set-layout --path "$LAYOUT_FILE" --shared-backend "backend=disk;path=$CONFIG_FILE;namespace=common"
cargo run -p diem-genesis-tool -- diem-root-key --validator-backend "backend=disk;path=$PRIVATE_KEYFILE" --shared-backend "backend=disk;path=$CONFIG_FILE;namespace=diem"
cargo run -p diem-genesis-tool -- treasury-compliance-key --validator-backend "backend=disk;path=$PRIVATE_KEYFILE" --shared-backend "backend=disk;path=$CONFIG_FILE;namespace=diem"
cargo run -p diem-genesis-tool -- operator-key --validator-backend "backend=disk;path=$PRIVATE_KEYFILE" --shared-backend "backend=disk;path=$CONFIG_FILE;namespace=$NAME"
cargo run -p diem-genesis-tool -- owner-key --validator-backend "backend=disk;path=$PRIVATE_KEYFILE" --shared-backend "backend=disk;path=$CONFIG_FILE;namespace=$NAME"
cargo run -p diem-genesis-tool -- set-operator --shared-backend "backend=disk;path=$CONFIG_FILE;namespace=$NAME" --operator-name "$NAME"
# Make sure the IP addresses here match those in the docker-compose
cargo run -p diem-genesis-tool -- validator-config --validator-backend "backend=disk;path=$PRIVATE_KEYFILE" --shared-backend "backend=disk;path=$CONFIG_FILE;namespace=$NAME" --validator-address "/ip4/172.16.1.2/tcp/6180" --fullnode-address "/ip4/172.16.1.4/tcp/6182" --owner-name "$NAME" --chain-id "$CHAIN_ID"

# Run genesis to create the first block!
echo -e "\n${LBLUE}Running Genesis${NC} "
cargo run -p diem-genesis-tool -- genesis --shared-backend "backend=disk;path=$CONFIG_FILE;namespace=$NAME" --path "$GENESIS_BLOBFILE" --chain-id "$CHAIN_ID"

echo -e "\n${LBLUE}Creating Waypoint${NC} "
cargo run -p diem-genesis-tool -- create-waypoint --shared-backend "backend=disk;path=$CONFIG_FILE" --chain-id "$CHAIN_ID" | cut -d' ' -f2 >"$WAYPOINT_FILE"
cargo run -p diem-genesis-tool -- insert-waypoint --validator-backend "backend=disk;path=$PRIVATE_KEYFILE" --set-genesis --waypoint "$(cat "$WAYPOINT_FILE")"

echo -e "\n${LBLUE}Extracting Mint Key${NC} "
cargo run -p diem-operational-tool -- extract-private-key --key-name 'treasury_compliance' --key-file "$MINT_KEYFILE" --validator-backend "backend=disk;path=$PRIVATE_KEYFILE"
