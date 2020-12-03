#!/bin/bash -eux
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

# this file generates 128 corpus files for a target, zips it all, and move zipfile to an outdir
# usage: ./build_corpus.sh ConsensusProposal outdir

export SINGLE_FUZZ_TARGET="$1"
export OUT_DIR="$2"

cargo run --bin diem-fuzzer generate -n 128 $SINGLE_FUZZ_TARGET
zip -r $OUT_DIR/"$SINGLE_FUZZ_TARGET"_seed_corpus.zip fuzz/corpus/$SINGLE_FUZZ_TARGET
rm -r fuzz/corpus/$SINGLE_FUZZ_TARGET
