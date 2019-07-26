#!/bin/bash

# Check that the test directory and report path arguments are provided
if [ $# -ne 2 ]
then
	echo "Usage: $0 <testdir> <outdir>"
	echo "All tests in <testdir> and its subdirectories will be run to measure coverage."
	echo "The resulting coverage report will be stored in <outdir>."
	exit 1
fi

# Set the directory containing the tests to run (includes subdirectories)
TEST_DIR=$1

# Set the directory to which the report will be saved
COVERAGE_DIR=$2

# This needs to run in libra
SCRIPT_DIR="$( dirname "${BASH_SOURCE[0]}" )"
if [ ! $SCRIPT_DIR == "./scripts" ]
then
	echo "This needs to run from libra/, not in $(pwd)"
	exit 1
fi

set -e

# Check that grcov is installed
if ! [ -x "$(command -v grcov)" ]; then
	echo "Error: grcov is not installed." >&2
	read -p "Install grcov? [yY/*] " -n 1 -r
	echo ""
	if [[ ! $REPLY =~ ^[Yy]$ ]]
	then
		[[ "$0" = "$BASH_SOURCE" ]] && exit 1 || return 1
	fi
	cargo install grcov
fi

# Check that lcov is installed
if ! [ -x "$(command -v lcov)" ]; then
	echo "Error: lcov is not installed." >&2
	echo "Documentation for lcov can be found at http://ltp.sourceforge.net/coverage/lcov.php"
	echo "If on macOS and using homebrew, run 'brew install lcov'"
fi

# Warn that cargo clean will happen
read -p "Generate coverage report? This will run cargo clean. [yY/*] " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]
then
	[[ "$0" = "$BASH_SOURCE" ]] && exit 1 || return 1
fi

export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Coverflow-checks=off -Zno-landing-pads"
export CARGO_INCREMENTAL=0

# Clean the project
echo "Cleaning project..."
(cd $TEST_DIR; cargo clean)

# Build with flags necessary for coverage output
echo "Building with coverage instrumentation..."
(cd $TEST_DIR; cargo build)

# Remove existing coverage output
echo "Cleaning existing coverage info..."
if [ -d "./target" ]
then
	find target -type f -name "*.gcda" -delete
	find target -type f -name "*.gcno" -delete
else
	echo "Error: target directory does not exist. Did cargo build fail?"
fi

# Run tests
echo "Running tests..."
while read line; do
	dirline=$(realpath $(dirname $line));
	(cd $dirline; cargo test)
done < <(find $TEST_DIR -name 'Cargo.toml')

# Make the coverage directory if it doesn't exist
if [ ! -d $COVERAGE_DIR ]; then
	mkdir $COVERAGE_DIR;
fi

# Generate lcov report
echo "Generating lcov report at ${COVERAGE_DIR}/lcov.info..."
grcov target -t lcov  --llvm --branch --ignore-dir "/*" -o $COVERAGE_DIR/lcov.info

# Generate HTML report
echo "Generating report at ${COVERAGE_DIR}..."
# Flag "--ignore-errors source" ignores missing source files
genhtml -o $COVERAGE_DIR --show-details --highlight --ignore-errors source --legend $COVERAGE_DIR/lcov.info

echo "Done. Please view report at ${COVERAGE_DIR}/index.html"
