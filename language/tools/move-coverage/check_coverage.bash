#!/bin/bash

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

TRACE_PATH=$HOME/trace

[ ! -e  "$TRACE_PATH" ] || rm -f "$TRACE_PATH"

export MOVE_VM_TRACE=$TRACE_PATH

echo "Rebuilding stdlib..."
pushd ../../stdlib || exit 1
cargo run
popd || exit 1

#cargo test -p ir-testsuite -p language-e2e-tests -p move-lang-functional-tests

echo "Running IR testsuite..."
pushd ../../ir-testsuite || exit 1
cargo test
popd || exit 1

echo "Running e2e testsuite..."
pushd ../../e2e-tests || exit 1
cargo test -- --skip account_universe --skip fuzz_scripts
popd || exit 1

echo "Running Move testsuite..."
pushd ../../move-lang/functional-tests/tests || exit 1
cargo test
popd || exit 1

echo "Building Move modules and source maps.."
pushd ../../move-lang || exit 1
rm -rf move_build_output
cargo run --bin move-build -- ../stdlib/modules -m
popd || exit 1

echo "Converting trace file..."
cargo run --bin move-trace-conversion -- -f "$TRACE_PATH" -o trace.mvcov

echo "Producing coverage summaries..."
cargo run --bin coverage-summaries -- -t trace.mvcov -s ../../stdlib/compiled/stdlib

echo "==========================================================================="
echo "You can check source coverage for a module by running:"
echo "> cargo run --bin source-coverage -- -t trace.mvcov -b ../../move-lang/move_build_output/modules/<LOOK_FOR_MODULE_HERE>.mv -s ../../stdlib/modules/<SOURCE_MODULE>.move"
echo "---------------------------------------------------------------------------"
echo "You can can also get a finer-grained coverage summary for each function by running:"
echo "> cargo run --bin coverage-summaries -- -t trace.mvcov -s ../../stdlib/compiled/stdlib.mv"
echo "==========================================================================="

unset MOVE_VM_TRACE

echo "DONE"
