#!/bin/bash

print_help()
{
    echo "Build client binary and connect to testnet."
    echo "\`$0 -r|--release\` to use release build or"
    echo "\`$0\` to use debug build."
}

source "$HOME/.cargo/env"

SCRIPT_PATH="$(dirname $0)"

RUN_PARAMS="--host ac.testnet.libra.org --port 8000 -s $SCRIPT_PATH/trusted_peers.config.toml"

case $1 in
    -h | --help)
        print_help;exit 0;;
    -r | --release)
        echo "Building and running client in release mode."
        cargo run -p client --release -- $RUN_PARAMS
        ;;
    '')
        echo "Building and running client in debug mode."
        cargo run -p client -- $RUN_PARAMS
        ;;
    *) echo "Invalid option"; print_help; exit 0;
esac
