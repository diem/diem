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
cd $OUTDIR

# Remove all generated node.config.toml files.
find . -mindepth 2 -iname node.config.toml | xargs rm
# Move all keys files to single top-level directory.
find . -mindepth 2 -iname '*.keys.toml' | xargs -I '{}' mv '{}' ./
# Move all seed peers files to top-level directory.
find . -mindepth 2 -iname '*.seed_peers.config.toml' | xargs rm
# Move all network peers files to top-level directory.
find . -mindepth 2 -iname '*.network_peers.config.toml' -exec mv -f {} ./network_peers.config.toml \;
# Move all consensus peers files to top-level directory.
find . -mindepth 2 -iname 'consensus_peers.config.toml' -exec mv -f {} ./consensus_peers.config.toml \;
# Move all genesis.blob files to top-level directory.
find . -mindepth 2 -iname 'genesis.blob' -exec mv -f {} ./genesis.blob \;
# Delete all directories.
find . -mindepth 1 -type d | xargs rmdir

cd -
