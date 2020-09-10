#!/bin/bash
# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

# Check that the test directory and report path arguments are provided
if [ $# -lt 1 ]
then
	echo "Usage: $0 <outdir> [--batch]"
	echo "The resulting coverage report will be stored in <outdir>."
	echo "--batch will skip all prompts."
	exit 1
fi

# User prompts will be skipped if '--batch' is given as the third argument
SKIP_PROMPTS=0
if [ $# -eq 2 -a "$2" == "--batch" ]
then
	SKIP_PROMPTS=1
fi

# Set the directory to which the report will be saved
COVERAGE_DIR=$1

# This needs to run in test-generation
TOOL_DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd)"
if [ $(pwd) != $TOOL_DIR  ]
then
	echo "Error: This needs to run from test-generation/, not in $(pwd)" >&2
	exit 1
fi

set -e

# Check that grcov is installed
if ! [ -x "$(command -v grcov)" ]; then
	echo "Error: grcov is not installed." >&2
	if [ $SKIP_PROMPTS -eq 0 ]
	then
		read -p "Install grcov? [yY/*] " -n 1 -r
		echo ""
		if [[ ! $REPLY =~ ^[Yy]$ ]]
		then
			[[ "$0" = "$BASH_SOURCE" ]] && exit 1 || return 1
		fi
		cargo install grcov
	else
		exit 1
	fi
fi

# Check that lcov is installed
if ! [ -x "$(command -v lcov)" ]; then
	echo "Error: lcov is not installed." >&2
	echo "Documentation for lcov can be found at http://ltp.sourceforge.net/coverage/lcov.php"
	echo "If on macOS and using homebrew, run 'brew install lcov'"
	exit 1
fi

# Warn that cargo clean will happen
if [ $SKIP_PROMPTS -eq 0 ]
then
	read -p "Generate coverage report? This will run cargo clean. [yY/*] " -n 1 -r
	echo ""
	if [[ ! $REPLY =~ ^[Yy]$ ]]
	then
		[[ "$0" = "$BASH_SOURCE" ]] && exit 1 || return 1
	fi
fi

# Set flags for coverage generation
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Coverflow-checks=off -Zno-landing-pads"
export CARGO_INCREMENTAL=0

# Clean the project
echo "Cleaning project..."
cargo clean

# Run bytecode test generator
echo "Running bytecode test generator..."
RUST_LOG=info cargo run -- --iterations 1000

# Make the coverage directory if it doesn't exist
if [ ! -d $COVERAGE_DIR ]; then
	mkdir $COVERAGE_DIR;
fi

# Generate lcov report
echo "Generating lcov report at ${COVERAGE_DIR}/lcov.info..."
grcov $TOOL_DIR/../../../target/ -t lcov --llvm --branch --ignore-dir "/*" -o $COVERAGE_DIR/lcov.info

# Generate HTML report
echo "Generating report at ${COVERAGE_DIR}..."
# Flag "--ignore-errors source" ignores missing source files
(cd $TOOL_DIR/../../../; genhtml -o $COVERAGE_DIR --show-details --highlight --ignore-errors source --legend --branch-coverage $COVERAGE_DIR/lcov.info)

echo "Done. Please view report at ${COVERAGE_DIR}/index.html"
