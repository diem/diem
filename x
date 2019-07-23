#!/bin/sh
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# This script is meant to be a light-weight bootstrapping mechanism around
# Anchor, a toolchain shipped with Libra intended to improve the developer
# workflow.

set -e

if [ -z "$ANCHOR" ]; then
	cat <<-EOF
	Anchor and x are currently under heavy developmnet and in a pre-Alpha state.
	If you would still like to try it out you can bypass this check by setting
	the 'ANCHOR' environment variable, e.g. 'ANCHOR=1 ./x'
	EOF

	exit 0
fi

SCRIPT_PATH="$(cd "$(dirname "$0")" >/dev/null 2>&1 && pwd)"
cd "$SCRIPT_PATH"

if [ ! -f rust-toolchain ]; then
	echo "Unknown location. Please run this from the libra repository. Abort."
	exit 1
fi

# Install Rust if it hasn't already been installed
if ! rustup --version >/dev/null 2>&1; then
    echo "Installing Rust......"

	curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain none
	CARGO_ENV="$HOME/.cargo/env"
	source "$CARGO_ENV"

    cat <<-EOF

	Successfully installed Rust!

	Please follow the instructions provided above by the Rust installer and add
	Cargo's bin directory to your path.
	EOF
fi

# Check that the active toolchain matches the one pinned by version control.
# Performing this check should download the required toolchain if necessary.
if ! rustup show active-toolchain 2>/dev/null | grep -q "$(cat rust-toolchain)"; then
	echo "x: unable to sync pinned Rust toolchain"
	exit 1
fi

#TODO move this to the lint step in rust code
# Add all the components that we need
rustup component add rustfmt >/dev/null 2>&1
rustup component add clippy >/dev/null 2>&1

if ! cargo build --manifest-path=anchor/Cargo.toml --bin anchor >/dev/null 2>&1; then
	echo "x: unable to build anchor binary"
	exit 1
fi

exec anchor/target/debug/anchor "$@"
