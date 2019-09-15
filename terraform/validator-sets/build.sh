#!/bin/sh
set -e

OUTDIR="${1?[Specify relative output directory]}"
shift

mkdir -p "$OUTDIR"

cd ../..

if [ ! -e "terraform/validator-sets/$OUTDIR/mint.key" ]; then
	cargo run --bin generate_keypair -- -o "terraform/validator-sets/$OUTDIR/mint.key"
fi

cargo run --bin libra-config -- -b config/data/configs/node.config.toml -m "terraform/validator-sets/$OUTDIR/mint.key" -o "terraform/validator-sets/$OUTDIR" -d "$@"

cd -
rm $OUTDIR/*.node.config.toml
