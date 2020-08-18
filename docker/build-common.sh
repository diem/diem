#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# This is a common compilation scripts across different docker file
# It unifies RUSFLAGS, compilation flags (like --release) and set of binary crates to compile in common docker layer

export RUSTFLAGS="-Ctarget-cpu=skylake -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3"

# We are using a pinned nightly cargo until feature resolver v2 is released (around 10/2020), but compiling with stable rustc
export RUST_NIGHTLY=$(cat cargo-toolchain)
export CARGO=$(rustup which --toolchain $RUST_NIGHTLY cargo)
export CARGOFLAGS=$(cat cargo-flags)
export CARGO_PROFILE_RELEASE_LTO=thin # override lto setting to turn on thin-LTO for release builds

# skip building non-release binaries to shave off time
BUILD_TEST=$1

# Build release binaries
${CARGO} ${CARGOFLAGS} build --release \
         -p libra-genesis-tool \
         -p libra-operational-tool \
         -p libra-node \
         -p config-builder \
         -p libra-key-manager \
         -p safety-rules \
         -p db-bootstrapper \
         -p backup-cli \
         "$@"

if [ "$1" = "test" ]; then
    # These non-release binaries are built separately to avoid feature unification issues
    ${CARGO} ${CARGOFLAGS} build --release \
            -p cluster-test \
            -p cli \
            "$@"
fi

rm -rf target/release/{build,deps,incremental}

STRIP_DIR=${STRIP_DIR:-/libra/target}
find "$STRIP_DIR/release" -maxdepth 1 -executable -type f | grep -Ev 'libra-node|safety-rules' | xargs strip
