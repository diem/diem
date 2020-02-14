#!/bin/bash

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# This script modifies local cargo files to reflect compatibility (semver)
# upgrades to direct dependencies. It does not allow non-compatible
# updates as those should be done manually (reading the output of cargo outdated)

# It requires cargo-edit and cargo-outdated
# Example usage:
# libra$ ./scripts/cargo_update_outdated.sh
# libra$ git commit --all -m "Update dependencies"
set -e

# check install for outdated & edit
if ! $(cargo install --list | grep -qe 'cargo-outdated')
then
   cargo install cargo-outdated
fi

if ! $(cargo install --list | grep -qe 'cargo-edit')
then
   cargo install cargo-edit
fi

for upgrade in $(cargo outdated | awk 'NF >2 && $2 ~ /[0-9\.]+/ && $4 ~ /[0-9\.]+/ {print $1"@"$4}' | uniq |tr '\n' " ")
do
    echo $upgrade
    cargo -q upgrade $upgrade --all > /dev/null
done
