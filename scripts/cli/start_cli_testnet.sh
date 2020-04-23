#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

print_help()
{
    echo "Build client binary and connect to testnet."
    echo "\`$0 -r|--release\` to use release build or"
    echo "\`$0\` to use debug build."
}

source "$HOME/.cargo/env"

SCRIPT_PATH="$(dirname $0)"

RUN_PARAMS="--url https://client.testnet.libra.org --waypoint_url https://developers.libra.org/testnet_waypoint.txt"
RELEASE=""

while [[ ! -z "$1" ]]; do
	case "$1" in
		-h | --help)
			print_help;exit 0;;
		-r | --release)
			RELEASE="--release"
			;;
		--)
			shift
			break
			;;
		*) echo "Invalid option"; print_help; exit 0;
	esac
	shift
done

if [ -z "$RELEASE" ]; then
	echo "Building and running client in debug mode."
else
	echo "Building and running client in release mode."
fi

cargo run -p cli $RELEASE -- $RUN_PARAMS "$@"
