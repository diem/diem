#!/bin/bash
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

# A script to check stability of a verification outcome. This runs the same prover
# task N times with different random seeds and checks whether the produced output
# (diagnosis or success) stays stable. Use as in
#
#     check_stability.sh <move-source> N
#
# TODO: we should add here some logic to measure and compare verification timing.

set -e

PROVER="cargo run -q -p move-prover --release -- -v=error --stable-test-output"
BASELINE="check_stability.out"

$PROVER $1 > $BASELINE 2>&1 || true
for n in $(seq 1 $2); do
  rand=$RANDOM
  echo "experiment #$n with seed $rand"
  mv $BASELINE $BASELINE.last
  $PROVER --seed=$rand $1 > $BASELINE 2>&1 || true
  if [[ $(diff $BASELINE.last $BASELINE) ]]; then
    echo "not stable"
    exit 1
  fi
done
