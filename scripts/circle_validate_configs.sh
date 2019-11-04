#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# This script builds the libra configs
# and then checks for git modified files in the generated configs

# Assumptions:
# - Script is run in the correct Rust environment for the project
# - Script is running inside the git repository, with git command available
# - Script is run from repo's top level folder

# Generate Configs
echo "--- Generating all the configs ---"
cd terraform/validator-sets
./build-all.sh

# Compare configs
# Notice: This is not comparing all configs yet.

# Cleanup files we do compare yet
git checkout -- '**/seed_peers.config.toml' '**/genesis.blob'
git update-index --refresh

echo "--- Compare configs ---"
changes=$(git diff-index HEAD -- '**/consensus_peers.config.toml' '**/node.consensus.keys.toml' '**/network_peers.config.toml' '**/node.network.keys.toml')
if [ -z "$changes" ];
then
	# nothing to do
	echo "Done!"
	exit 0
else
	# config differs
	echo "Config Differs!!"
	echo $changes
	echo "Your code changes require updating configuration"
	echo "You can run this script itself: ${BASH_SOURCE[0]}"
	echo "Verify that all modified files were expected to change and commit."
	exit 1
fi
