## Overview

This crate implements a unified testing infrastructure that allows developers
to write tests as individual Move IR programs, send them through the entire
pipeline, and check the output of each stage using inline directives.

## How to run functional tests

Run `cargo test` inside this crate, or `cargo test -p functional_tests` anywhere
in the repo. `cargo test` also accepts a filter: `cargo test foo` runs only
the tests with `foo` in the name.

## Adding a new test

To add a new test, simply create a new .mvir file in `tests/testsuite`.
The test harness will recursively search for all move ir sources in
the directory and register each of them as a test case.

## Checking the test output using directives

Directives are essentially comments with special meanings to the testing infra.
They can be used to define patterns that should appear in the test output.
If the test output does not match the pattern specified, the test is
considered a failure.

The test output is a log-like structure that consists of the debug print
of the data structure each pipeline stage outputs. In case there is an
error, it is the debug print of the error. Good tests should match only
crucial details such as the name of the error and omit unimportant details
such as formatting, spaces, brackets etc.

When no directives are specified, the testing infra requires a test program
to pass all stages of the pipeline. Any error will result in a test failure.

See `tests/testsuite/examples` for more examples.
