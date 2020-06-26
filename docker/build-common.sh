#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# This is a common compilation scripts across different docker file
# It unifies RUSFLAGS, compilation flags (like --release) and set of binary crates to compile in common docker layer

export RUSTFLAGS="-Ctarget-cpu=skylake -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3"
cargo build --release -p libra-management -p libra-node -p cli -p config-builder -p libra-key-manager -p safety-rules -p db-bootstrapper -p backup-cli "$@"

rm -rf target/release/{build,deps,incremental}

STRIP_DIR=${STRIP_DIR:-/libra/target}
strip "$STRIP_DIR/release/config-builder"
strip "$STRIP_DIR/release/cli"
strip "$STRIP_DIR/release/db-backup"
strip "$STRIP_DIR/release/db-bootstrapper"
strip "$STRIP_DIR/release/db-restore"
strip "$STRIP_DIR/release/libra-management"
