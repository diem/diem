#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Simple script to record benchmarks for verification. Usage:
#
#     benchmark.sh [ -o out-file ] { -f prover-flag } src ...
#

set -e

OUT=benchmark.out
FLAGS=""

if [[ $1 == "-o" ]]; then
  shift
  OUT="$1"
  shift
fi
while [[ $1 == "-f" ]]; do
  shift
  FLAGS="$FLAGS $1"
  shift
done

echo "Building prover in release mode ..."
cargo build -p move-prover -q --release

echo -n "Run with FLAGS=\"$FLAGS\" from " > $OUT
date >> $OUT
for f in "$@"; do
    echo "Verifying $f ...";
    echo -n "$f: " >> $OUT
    ( "time" -o $OUT -a -f "%Us" cargo run -q -p move-prover --release -- --verbose=warn -T=100 $FLAGS $f ) || echo "failed!"
done

echo "Results written to $OUT"
