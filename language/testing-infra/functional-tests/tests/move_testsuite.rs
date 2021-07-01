// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Meta tests to check that the functional test framework itself is working as expected.

// Note: the path of the entry point function is included as part of the test name.
// Importing it and giving it an alias will make it more readable.
use functional_tests::harness::run_move_functional_test as functional_tests_meta_tests_move;

datatest_stable::harness!(
    functional_tests_meta_tests_move,
    "tests/move-testsuite",
    r".*\.move"
);
