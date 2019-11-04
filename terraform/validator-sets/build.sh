#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

OUT_DIR="${1?[Specify relative output directory]}"
shift

LIBRA_DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" && cd ../.. && pwd)"
TF_WORK_DIR="$LIBRA_DIR/terraform/validator-sets"
OUTPUT_DIR="$TF_WORK_DIR/$OUT_DIR"
mkdir -p "$OUTPUT_DIR"


if [ ! -e "$OUTPUT_DIR/mint.key" ]; then
	cargo run -p generate-keypair --bin generate-keypair -- -o "$OUTPUT_DIR/mint.key"
fi

cd $LIBRA_DIR && cargo run -p config-builder --bin libra-config -- -b $LIBRA_DIR/config/data/configs/node.config.toml -m "$OUTPUT_DIR/mint.key" -o "$OUTPUT_DIR/val" -d -r validator "$@"
cd $LIBRA_DIR && cargo run -p config-builder --bin libra-config -- -b $LIBRA_DIR/config/data/configs/node.config.toml -m "$OUTPUT_DIR/mint.key" -o "$OUTPUT_DIR/fn" -u "$OUTPUT_DIR/val/0" -r full_node -n 10


cd "$OUTPUT_DIR/val"
mv */*.keys.toml .
mv 1/*.network_peers.config.toml network_peers.config.toml
mv 1/consensus_peers.config.toml ../consensus_peers.config.toml
mv 1/genesis.blob ../
rm */*.toml */*.blob
find . -mindepth 1 -type d -print0 | xargs -0 rmdir

cd "$OUTPUT_DIR/fn"
mv */*.keys.toml .
mv 0/*.network_peers.config.toml network_peers.config.toml
rm */*.toml */*.blob
find . -mindepth 1 -type d -print0 | xargs -0 rmdir
