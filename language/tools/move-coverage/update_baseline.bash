#! /bin/bash

echo "Updating baseline...You must have already run check_coverage.bash in order for this to work"
cargo run --bin coverage-summaries -- -t trace.mvcov -s ../../stdlib/staged/stdlib.mv -o ./baseline/coverage_report
echo "DONE"
