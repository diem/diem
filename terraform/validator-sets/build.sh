#!/bin/sh
set -e

OUTDIR="${1?[Specify relative output directory]}"
shift

mkdir -p "$OUTDIR"

cd ../..

if [ ! -e "terraform/validator-sets/$OUTDIR/mint.key" ]; then
	cargo run --bin generate-keypair -- -o "terraform/validator-sets/$OUTDIR/mint.key"
fi

cargo run --bin libra-config -- -b config/data/configs/node.config.toml -m "terraform/validator-sets/$OUTDIR/mint.key" -o "terraform/validator-sets/$OUTDIR/val" -d -r validator "$@"
cargo run --bin libra-config -- -b config/data/configs/node.config.toml -m "terraform/validator-sets/$OUTDIR/mint.key" -o "terraform/validator-sets/$OUTDIR/fn" -u "terraform/validator-sets/$OUTDIR/val/0" -r full_node -n 10

cd -

cd $OUTDIR/val
mv */*.keys.toml .
mv 1/*.network_peers.config.toml network_peers.config.toml
mv 1/consensus_peers.config.toml ../consensus_peers.config.toml
mv 1/genesis.blob ../
rm */*.toml */*.blob
find . -mindepth 1 -type d -print0 | xargs -0 rmdir

cd ../fn
mv */*.keys.toml .
mv 0/*.network_peers.config.toml network_peers.config.toml
rm */*.toml */*.blob
find . -mindepth 1 -type d -print0 | xargs -0 rmdir
