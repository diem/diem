#!/bin/bash

# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

TRACE_PATH=$HOME/trace

[ ! -e  "$TRACE_PATH" ] || rm -f "$TRACE_PATH"

export MOVE_VM_TRACE=$TRACE_PATH

echo "Rebuilding stdlib..."
pushd ../../stdlib || exit 1
cargo run
popd || exit 1

#cargo test -p ir-testsuite -p language-e2e-tests -p move-lang-functional-tests

echo "---------------------------------------------------------------------------"
echo "Running IR testsuite..."
echo "---------------------------------------------------------------------------"
pushd ../../ir-testsuite || exit 1
cargo test
popd || exit 1

echo "---------------------------------------------------------------------------"
echo "Running e2e testsuite..."
echo "---------------------------------------------------------------------------"
pushd ../../e2e-testsuite || exit 1
cargo test -- --skip account_universe --skip fuzz_scripts
popd || exit 1

echo "---------------------------------------------------------------------------"
echo "Running Move testsuite..."
echo "---------------------------------------------------------------------------"
pushd ../../move-lang/functional-tests/tests || exit 1
cargo test
popd || exit 1

echo "---------------------------------------------------------------------------"
echo "Building Move modules and source maps.."
echo "---------------------------------------------------------------------------"
pushd ../../move-lang || exit 1
rm -rf build
cargo run --bin move-build -- ../stdlib/modules -m
popd || exit 1

echo "---------------------------------------------------------------------------"
echo "Converting trace file..."
echo "---------------------------------------------------------------------------"
cargo run --bin move-trace-conversion -- -f "$TRACE_PATH" -o trace.mvcov

echo "---------------------------------------------------------------------------"
echo "Producing coverage summaries..."
echo "---------------------------------------------------------------------------"
cargo run --bin coverage-summaries -- -t trace.mvcov -s ../../stdlib/compiled/stdlib

echo "==========================================================================="
echo "You can check source coverage for a module by running:"
echo "> cargo run --bin source-coverage -- -t trace.mvcov -b ../../move-lang/build/modules/<LOOK_FOR_MODULE_HERE>.mv -s ../../stdlib/modules/<SOURCE_MODULE>.move"
echo "---------------------------------------------------------------------------"
echo "You can can also get a finer-grained coverage summary for each function by running:"
echo "> cargo run --bin coverage-summaries -- -t trace.mvcov -s ../../stdlib/compiled/stdlib.mv"
echo "==========================================================================="

unset MOVE_VM_TRACE

echo "DONE"
