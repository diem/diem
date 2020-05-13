#! /bin/bash

TRACE_PATH=$HOME/trace

[ ! -e  "$TRACE_PATH" ] || rm -f "$TRACE_PATH"

export MOVE_VM_TRACE=$TRACE_PATH

echo "Rebuilding stdlib..."
pushd ../../stdlib || exit 1
cargo run
popd || exit 1

echo "Running IR testsuite..."
pushd ../../ir-testsuite || exit 1
cargo test
popd || exit 1

echo "Running e2e testsuite..."
pushd ../../e2e-tests || exit 1
cargo test
popd || exit 1

echo "Running Move testsuite..."
pushd ../../move-lang || exit 1
cargo test
cargo run --bin move-build -- -f ../stdlib/modules/* -m
popd || exit 1

echo "Converting trace file..."
cargo run --bin move-trace-conversion -- -f "$TRACE_PATH" -o trace.mvcov
echo "Producing coverage summaries..."
cargo run --bin coverage-summaries -- -t trace.mvcov -s ../../stdlib/staged/stdlib.mv -c -o new_coverage_summary.csv

unset MOVE_VM_TRACE

echo "DONE"
