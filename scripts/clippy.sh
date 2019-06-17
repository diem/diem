#!/bin/bash

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

set -e

SCRIPT_PATH="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# Workaround https://github.com/rust-lang/rust-clippy/issues/2604
if [[ "$1" == *"c"* ]]; then
	cargo clean
fi

# Run 'clippy' on all targets, ensuring that all warnings trigger a
# failure.
cargo clippy --all --all-targets -- -D warnings $(source "$SCRIPT_PATH/clippy.args")
