#!/bin/sh
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

echo "This lab cannot be run at head because the poly backend has been removed!"
echo "To run this lab, one has to time travel back to"
echo "https://github.com/diem/diem/commit/2b248773729ef75c805e94982cce7c941b11cbfb"

exit 1

DIEM="$(git rev-parse --show-toplevel)"
FRAMEWORK="$DIEM/language/diem-framework/modules"
STDLIB="$DIEM/language/move-stdlib/modules"

for config in *.toml ; do
  # Benchmark per function
  cargo run -q --release -p prover-lab -- \
    bench -f -c $config -d $STDLIB -d $FRAMEWORK $FRAMEWORK/*.move
  # Benchmark per module
  cargo run -q --release -p prover-lab -- \
    bench -c $config -d $STDLIB -d $FRAMEWORK $FRAMEWORK/*.move
done
