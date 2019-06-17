#!/bin/sh
set -e

OUTDIR="${1?[Specify relative output directory]}"
shift

mkdir -p "$OUTDIR"

cd ../..

if [ ! -e "../setup_scripts/terraform/testnet/validator-sets/$OUTDIR/mint.key" ]; then
    cargo run --bin generate_keypair -- -o "../setup_scripts/terraform/testnet/validator-sets/$OUTDIR/mint.key"
fi

cargo run --bin libra-config -- -b config/data/configs/node.config.toml -m "../setup_scripts/terraform/testnet/validator-sets/$OUTDIR/mint.key" -o "../setup_scripts/terraform/testnet/validator-sets/$OUTDIR" -d "$@" # -r config/data/configs/overrides/testnet.node.config.override.toml

cd -
cd $OUTDIR
ls *.node.config.toml | head -n1 | xargs -I{} mv {} node.config.toml
rm *.node.config.toml
