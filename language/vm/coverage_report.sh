#!/bin/bash


# Check that the report destination argument is provided
if [ $# -eq 0 ]
then
    echo "Usage: coverage_report.sh path/to/report/destination"
    exit 1
fi

# Set the directory to which the report will be saved
COVERAGE_DIR=$1

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
    echo "Assuming macOS and homebrew"
    read -p "Install lcov? [yY/*] " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]
    then
        [[ "$0" = "$BASH_SOURCE" ]] && exit 1 || return 1
    fi
    brew install lcov
fi

# Warn that cargo clean will happen
read -p "Generate coverage report? This will run cargo clean. [yY/*] " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]
then
    [[ "$0" = "$BASH_SOURCE" ]] && exit 1 || return 1
fi

# Remove existing coverage output
echo "Cleaning existing coverage info..."
find ../../target -type f -name "*.gcda" -delete
find ../../target -type f -name "*.gcno" -delete

# Clean the project
echo "Cleaning project..."
cargo clean

# Build with flags necessary for coverage output
echo "Building with coverage instrumentation..."
CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Coverflow-checks=off -Zno-landing-pads" cargo build

# Run tests
echo "Running tests..."
while read line; do
    dirline=$(realpath $(dirname $line));
    (cd $dirline; CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Coverflow-checks=off -Zno-landing-pads" cargo test)
done < <(find ../ -name 'Cargo.toml')

# Make the coverage directory if it doesn't exist
if [ ! -d $COVERAGE_DIR ]; then
    mkdir $COVERAGE_DIR;
fi

# Generate lcov report
echo "Generating lcov report at ${COVERAGE_DIR}/lcov.info..."
grcov ../../target -t lcov --ignore-dir "/*" -o $COVERAGE_DIR/lcov.info

# Generate HTML report
echo "Generating report at ${COVERAGE_DIR}..."
# Flag "--ignore-errors source" ignores missing source files
(cd ../../; genhtml -o $COVERAGE_DIR --show-details --highlight --ignore-errors source --legend $COVERAGE_DIR/lcov.info)

echo "Done. Please view report at ${COVERAGE_DIR}/index.html"
