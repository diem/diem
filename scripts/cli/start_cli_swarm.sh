#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

print_help()
{
	echo "Build spawn a local swarm and open the CLI."
	echo "\`$0 -r|--release\` to use release build or"
	echo "\`$0\` to use debug build."
	echo "Use -- to pass arguments directly to diem-swarm."
}

source "$HOME/.cargo/env"

SCRIPT_PATH="$(dirname $0)"

RELEASE="debug"
FLAGS=""

while [[ ! -z "$1" ]]; do
	case "$1" in
		-h | --help)
			print_help;exit 0;;
		-r | --release)
			RELEASE="release";
			FLAGS="--release";;
		--)
			shift
			break
			;;
		*) echo "Invalid option"; print_help; exit 0;
	esac
	shift
done

echo "Building and running swarm in $RELEASE mode with the following flags: $FLAGS"

cargo build -p diem-node $FLAGS
cargo build -p cli $FLAGS
cargo run -p diem-swarm -- -s --diem-node $SCRIPT_PATH/../../target/$RELEASE/diem-node --cli-path $SCRIPT_PATH/../../target/$RELEASE/cli "$@"
