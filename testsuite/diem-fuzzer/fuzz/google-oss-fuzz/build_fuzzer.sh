#!/bin/bash -eux
# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

# this file builds a single fuzzer target
# usage: ./build_fuzzer.sh ConsensusProposal outdir

export SINGLE_FUZZ_TARGET="$1"
export OUT_DIR="$2"

# RUSTC_BOOTSTRAP: to get some nightly features like ASAN
export RUSTC_BOOTSTRAP=1

# for rocksdb we need to link libc++ at the end
if [ "$(uname)" == "Darwin" ]; then
  export RUSTFLAGS=""
  export TARGET="x86_64-apple-darwin"
else
  export RUSTFLAGS="-C link-arg=-L/usr/local/lib -C link-arg=-lc++"
  export TARGET="x86_64-unknown-linux-gnu"
fi

# export fuzzing flags
RUSTFLAGS="$RUSTFLAGS --cfg fuzzing"          # used to change code logic
RUSTFLAGS="$RUSTFLAGS -Zsanitizer=address"    # address sanitizer (ASAN)

# export SanitizerCoverage options (used by libfuzzer)
RUSTFLAGS="$RUSTFLAGS -Cpasses=sancov"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-level=4"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-trace-compares"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-inline-8bit-counters"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-trace-geps"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-prune-blocks=0"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-pc-table"
RUSTFLAGS="$RUSTFLAGS -Cllvm-args=-sanitizer-coverage-stack-depth"

# export codegen flags (https://doc.rust-lang.org/rustc/codegen-options/index.html)
RUSTFLAGS="$RUSTFLAGS -Ccodegen-units=1"
RUSTFLAGS="$RUSTFLAGS -Cdebug-assertions"     # to get debug_assert in rust

export RUSTFLAGS

# build
cargo build --manifest-path fuzz/Cargo.toml --bin fuzzer_builder --target $TARGET
# move fuzzer to $OUT
mv $SRC/diem/target/$TARGET/debug/fuzzer_builder $OUT_DIR/$SINGLE_FUZZ_TARGET
