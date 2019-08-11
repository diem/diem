#!/bin/bash
set -ex
## Download Fuzzit CLI
cd ./testsuite/libra_fuzzer
wget -q -O fuzzit https://github.com/fuzzitdev/fuzzit/releases/download/v2.4.39/fuzzit_Linux_x86_64
chmod a+x fuzzit

# Build targets
export FUZZIT_API_KEY=e27b764735b43b2f5d63a2285a03e109b3aca39096fc90f454c0bd96f65df859e659776a0da4028a774722b0d02e9f6a
export PORTABLE=1
export RUSTFLAGS="-Ctarget-feature=-avx512f"
cargo install cargo-fuzz
TARGETS=$(cargo run list --no-desc 2> /dev/null)
for i in $TARGETS
do
	cargo run fuzz "${i}" -- -- -runs=1
	# run fuzzing on push to master
	TARGET_NAME=${i//_/-}
	./fuzzit create target --skip-if-exists --public-corpus "${TARGET_NAME}"
	./fuzzit create job -e FUZZ_TARGET="${i}" libra/${TARGET_NAME} ./fuzz/target/x86_64-unknown-linux-gnu/debug/fuzz_runner
done
