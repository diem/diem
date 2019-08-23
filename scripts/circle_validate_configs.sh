#!/bin/bash

# This script builds the libra configs
# and then checks for git modified files in the generated configs

# Assumptions:
# - Script is run in the correct Rust environment for the project
# - Script is running inside the git repository, with git command available
# - Script is run from repo's top level folder

# Generate Configs
cd terraform/validator-sets
./build.sh dev -n 4
./build.sh 60 -n 60
./build.sh 100 -n 100


# Compare configs
# Notice: This is not comparing all configs just yet.
# We're also not cerrectly regenerating all configs

if $(git diff --exit-code HEAD -- **/genesis.blob **/trusted_peers.config.toml **/node.keys.toml) ; then
	# nothing to do
	exit 0
else
	# Config differs
	echo "Config Differs!!"
	echo "Your change requires updating configuration. Build the config and commit it."
	echo "Instructions for building config are in: terraform/validator-sets/build.sh"
	exit 1
fi
