// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Note: the path of the entry point function is included as part of the test name.
// Importing it and giving it an alias will make it more readable.
use functional_tests::harness::run_mvir_functional_test as ir_testsuite;
datatest_stable::harness!(ir_testsuite, "tests", r".*\.mvir");
