#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# This is a common compilation scripts across different docker file
# It unifies RUSFLAGS, compilation flags (like --release) and set of binary crates to compile in common docker layer

export RUSTFLAGS="-Ctarget-cpu=skylake -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3"

export RUST_NIGHTLY=$(cat cargo-toolchain)
export CARGO=$(rustup which --toolchain $RUST_NIGHTLY cargo)
export CARGOFLAGS=$(cat cargo-flags)

${CARGO} ${CARGOFLAGS} build --release -p libra-genesis-tool -p libra-node -p cli -p config-builder -p libra-key-manager -p safety-rules -p db-bootstrapper -p backup-cli -p cluster-test "$@"

rm -rf target/release/{build,deps,incremental}

STRIP_DIR=${STRIP_DIR:-/libra/target}
find "$STRIP_DIR/release" -maxdepth 1 -executable -type f | xargs strip
