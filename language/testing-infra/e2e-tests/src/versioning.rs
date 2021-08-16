// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use crate::{account::Account, executor::FakeExecutor, utils};
use diem_writeset_generator::old_releases::*;

/// The current version numbers that e2e tests should be run against.
pub const CURRENT_RELEASE_VERSIONS: &[u64] = &[1, 2, 3];

#[derive(Debug)]
pub struct VersionedTestEnv {
    pub executor: FakeExecutor,
    pub dr_account: Account,
    pub tc_account: Account,
    pub dd_account: Account,
    pub dr_sequence_number: u64,
    pub tc_sequence_number: u64,
    pub dd_sequence_number: u64,
    pub version_number: u64,
}

impl VersionedTestEnv {
    // At each release, this function will need to be updated to handle the release logic
    pub fn new(version_number: u64) -> Self {
        let (mut executor, dr_account, tc_account, dd_account) = utils::start_with_released_df();
        let mut dr_sequence_number = 1;
        let tc_sequence_number = 0;
        let dd_sequence_number = 0;

        if version_number > 1 {
            executor.execute_and_apply(
                dr_account
                    .transaction()
                    .sequence_number(1)
                    .payload(release_1_2_0_writeset())
                    .sign(),
            );
            dr_sequence_number += 1;
        }

        if version_number > 2 {
            executor.execute_and_apply(
                dr_account
                    .transaction()
                    .sequence_number(2)
                    .payload(release_1_4_0_writeset())
                    .sign(),
            );
            dr_sequence_number += 1;
        }

        // only support up to version 3 for now
        if version_number > 3 {
            panic!("Unsupported version number {}", version_number)
        }

        // Add other future version cases and upgrade paths here.

        Self {
            executor,
            dr_account,
            tc_account,
            dd_account,
            dr_sequence_number,
            tc_sequence_number,
            dd_sequence_number,
            version_number,
        }
    }
}

/// This is takes a test body parametrized by a `VersionedTestEnv`, and the `versions` to test
/// against The starting state of the `VersionedTestEnv` for each version number is determined by
/// the `starting_state` function.
pub fn run_with_versions<ParamExec, F>(
    test_golden_prefix: &str,
    versions: &[u64],
    starting_state: ParamExec,
    test_func: F,
) where
    F: Fn(VersionedTestEnv),
    ParamExec: Fn(u64) -> VersionedTestEnv,
{
    for version in versions {
        let mut testing_env = starting_state(*version);
        // Tag each golden file with the version that it's being run with, and should be compared against
        testing_env
            .executor
            .set_golden_file(&format!("{}_version_{}", test_golden_prefix, version));
        test_func(testing_env)
    }
}

// This needs to be a macro so that `current_function_name` behaves correctly.
#[macro_export]
macro_rules! test_with_different_versions {
    ($versions:expr, $expr:expr) => {
        language_e2e_tests::versioning::run_with_versions(
            language_e2e_tests::current_function_name!(),
            $versions,
            language_e2e_tests::versioning::VersionedTestEnv::new,
            $expr,
        )
    };
}
