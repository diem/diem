#!/bin/bash -eux
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# This build script is used with Google OSS-Fuzz to build fuzzer with their docker container.

# recipe:
# -------
# 1. we list all the fuzzers and save to a file fuzzer_list
# 2. we build the corpus for each fuzzer
# 3. we build all the fuzzers

# fetch all dependencies (needed for patching rocksdb)
cargo fetch

# for rocksdb we need to link libc++ at the end
export RUSTFLAGS="-C link-arg=-L/usr/local/lib -C link-arg=-lc++"

# 1. list fuzzers
cargo run --bin libra-fuzzer list --no-desc > fuzzer_list

# 2. build corpus and move to $OUT
cat fuzzer_list | while read -r line
do
    cargo run --bin libra-fuzzer generate -n 128 $line
    zip -r $OUT/"$line"_seed_corpus.zip fuzz/corpus/$line
    rm -r fuzz/corpus/$line
done

# RUSTC_BOOTSTRAP: to get some nightly features like ASAN
export RUSTC_BOOTSTRAP=1

# export fuzzing flags
RUSTFLAGS="$RUSTFLAGS --cfg fuzzing"          # used to change code logic
RUSTFLAGS="$RUSTFLAGS -Cdebug-assertions"     # to get debug_assert in rust
RUSTFLAGS="$RUSTFLAGS -Zsanitizer=address"    # address sanitizer (ASAN)

RUSTFLAGS="$RUSTFLAGS -Cpasses=sancov"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-level=4"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-trace-compares"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-inline-8bit-counters"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-trace-geps"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-prune-blocks=0"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-pc-table"
RUSTFLAGS="$RUSTFLAGS -Clink-dead-code"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-stack-depth"
RUSTFLAGS="$RUSTFLAGS -Ccodegen-units=1"

export RUSTFLAGS

# 3. build all the fuzzers!
cat fuzzer_list | while read -r line
do
    # build
    export SINGLE_FUZZ_TARGET="$line"
    cargo build --manifest-path fuzz/Cargo.toml --bin fuzzer_builder --target x86_64-unknown-linux-gnu
    # move fuzzer to $OUT
    mv $SRC/libra/target/x86_64-unknown-linux-gnu/debug/fuzzer_builder $OUT/$SINGLE_FUZZ_TARGET
done
