# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

#!/bin/sh
set -ex

GENESIS="${1:?[Specify path to diem-genesis-tool]}"
BACKEND="${2:?[Specify backend configuration]}"
VALIDATOR="${3:?[Specify validator endpoint]}"
FULLNODE="${4:?[Specify fullnode endpoint]}"

NAME="aws"

TEMPFILE="$(mktemp)"
LAYOUT="$(mktemp)"

cat > "$LAYOUT" <<EOF
owners = ["$NAME"]
operators = ["$NAME"]
diem_root = "diem"
treasury_compliance = "diem"
EOF

$GENESIS set-layout --path "$LAYOUT" --shared-backend "backend=disk;path=$TEMPFILE;namespace=common"
$GENESIS diem-root-key --validator-backend "$BACKEND" --shared-backend "backend=disk;path=$TEMPFILE;namespace=diem"
$GENESIS treasury-compliance-key --validator-backend "$BACKEND" --shared-backend "backend=disk;path=$TEMPFILE;namespace=diem"
$GENESIS operator-key --validator-backend "$BACKEND" --shared-backend "backend=disk;path=$TEMPFILE;namespace=$NAME"
$GENESIS owner-key --validator-backend "$BACKEND" --shared-backend "backend=disk;path=$TEMPFILE;namespace=$NAME"
$GENESIS set-operator --shared-backend "backend=disk;path=$TEMPFILE;namespace=$NAME" --operator-name "$NAME"
$GENESIS validator-config --validator-backend "$BACKEND" --shared-backend "backend=disk;path=$TEMPFILE;namespace=$NAME" --validator-address "$VALIDATOR" --fullnode-address "$FULLNODE" --owner-name "$NAME" --chain-id DEVNET
$GENESIS genesis --path genesis.blob --shared-backend "backend=disk;path=$TEMPFILE" --chain-id DEVNET
WAYPOINT="$($GENESIS create-waypoint --shared-backend "backend=disk;path=$TEMPFILE" --chain-id DEVNET | cut -d' ' -f2)"
$GENESIS insert-waypoint --validator-backend "$BACKEND" --set-genesis --waypoint "$WAYPOINT"

rm "$LAYOUT" "$TEMPFILE"
