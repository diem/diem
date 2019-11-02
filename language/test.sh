#!/bin/bash -e

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# This script runs `cargo test` for each crate in the subdir

base_cmd="cargo test --all-features"
dir="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
while read line; do
    echo $line;
    dirline=$(realpath $(dirname $line));
    cmd="cd $dirline && $base_cmd"
    echo $cmd;
    # Run the cmd in a subshell since it switches directories.
    (eval $cmd)
done < <(find "$dir" -name 'Cargo.toml')
