#!/bin/bash

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Run this script if the serialization format changes.
cwd=$(dirname "$0")
INPUT="$cwd/../../../../../language/stdlib/transaction_scripts/placeholder_script.mvir"
OUTPUT="$cwd/placeholder_script.mvbin"

cargo run -p compiler -- --no-stdlib "$INPUT" --output tmp
bytes=$(cat tmp  | sed 's/[^[]*\[\([^[]*\)\].*/\1/')
IFS=', ' read -r -a bytes <<< "$bytes"
for i in "${!bytes[@]}"
do
    bytes[$i]=$(printf "%02x" "${bytes[$i]}")
done
echo "${bytes[@]}" | xxd -r -p > "$OUTPUT"
rm tmp
