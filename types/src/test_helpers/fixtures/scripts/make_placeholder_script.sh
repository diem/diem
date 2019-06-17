#!/bin/bash

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Run this script if the serialization format changes.
cwd=$(dirname "$0")
INPUT="$cwd/../../../../../language/stdlib/transaction_scripts/placeholder_script.mvir"
OUTPUT="$cwd/placeholder_script.mvbin"
cargo run --package compiler -- --script --no-stdlib "$INPUT" --output "$OUTPUT"
