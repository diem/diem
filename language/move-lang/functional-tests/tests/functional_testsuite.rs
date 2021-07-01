// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Note: the path of the entry point function is included as part of the test name.
// Importing it and giving it an alias will make it more readable.
use functional_tests::harness::run_move_functional_test as move_lang_functional_tests;
datatest_stable::harness!(move_lang_functional_tests, "tests", r".*\.move");
