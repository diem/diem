#!/bin/bash -eux
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

# This build script is used with Google OSS-Fuzz to build fuzzer with their docker container.
# It must be called from diem/testsuite/diem-fuzzer
# usage: OUT=out_dir ./build.sh

# recipe:
# -------
# 1. we list all the fuzzers and save to a file fuzzer_list
# 2. we build the corpus for each fuzzer
# 3. we build all the fuzzers

# fetch all dependencies (needed for patching rocksdb)
cargo fetch

# 1. list fuzzers
cargo run --bin diem-fuzzer list --no-desc > fuzzer_list

# 2. build corpus and move to $OUT
cat fuzzer_list | while read -r line
do
    fuzz/google-oss-fuzz/build_corpus.sh $line $OUT
done

# 3. build all the fuzzers!
cat fuzzer_list | while read -r line
do
    # build & move fuzzer to $OUT
    fuzz/google-oss-fuzz/build_fuzzer.sh $line $OUT
done
