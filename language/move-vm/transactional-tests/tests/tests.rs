// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub const TEST_DIR: &str = "tests";
use move_transactional_test_runner::vm_test_harness::run_test;

datatest_stable::harness!(run_test, TEST_DIR, r".*\.(mvir|move)$");
