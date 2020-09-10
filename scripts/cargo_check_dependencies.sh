#!/bin/bash

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# This script assumes it runs in the same directory as a Cargo.toml file
# and sees if this Cargo.toml file can operate without some of its
# dependencies using repeated `cargo check --all-targets` attempts.
#
# In order to run this in a directory containing multiple Cargo.toml files,
# we could suggest:
# find ./ -name Cargo.toml -execdir /path/to/$(basename $0) \;

# Requirements:
# https://github.com/killercup/cargo-edit << `cargo install cargo-edit`
# awk, bash, git
# This will make one local commit per removable dependency. It is advised to
# squash those dependency-removing commits into a single one.

if [ ! -f Cargo.toml ]; then
    echo "Cargo.toml not found! Are you running this script in the right directory?"
fi

# Here, awk attempts to capture the dependency names among the [(build-|dev-)dependencies]
# blocks of the Cargo.toml
dependencies=$(awk  'x==1 {print } /\[(build-|dev-)?dependencies\]/ {x=1} /^$/ {x=0}' Cargo.toml| grep -Ev '(^$|^\[.+\]$)'| grep -Eo '^\w+')
echo "$dependencies"
for i in $dependencies; do
    echo "testing removal of $i"
    cargo rm "$i";
    cargo check --all-targets --all-features
    if (( $? == 0 )); then
        echo "removal succeeded, committing"
        git commit --no-gpg-sign -m "Removing $i from $(basename `pwd`)" --all;
    else
        echo "removal failed, rolling back"
        git reset --hard
    fi
done
