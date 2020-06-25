#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0
set -e

# This is a common compilation scripts across different docker file
# It unifies RUSFLAGS, compilation flags (like --release) and set of binary crates to compile in common docker layer

export RUSTFLAGS="-Ctarget-cpu=skylake -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3"
cargo build --release -p libra-management -p libra-node -p cli -p config-builder -p libra-key-manager -p safety-rules -p db-bootstrapper -p backup-cli "$@"

rm -r target/release/{build,deps,incremental}

strip /libra/target/release/config-builder
strip /libra/target/release/cli
strip /libra/target/release/db-backup
strip /libra/target/release/db-bootstrapper
strip /libra/target/release/db-restore
strip /libra/target/release/libra-management
