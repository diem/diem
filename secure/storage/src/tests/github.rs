// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{tests::suite, GitHubStorage, Storage};

const OWNER: &str = "OWNER";
const REPOSITORY: &str = "REPOSITORY";
const TOKEN: &str = "TOKEN";

// These tests must be run in series via: `cargo xtest -- --ignored --test-threads=1`
// Also the constants above must be defined with proper values -- never commit these values to the
// repository.
#[ignore]
#[test]
fn github_storage() {
    let mut storage = Storage::from(GitHubStorage::new(
        OWNER.into(),
        REPOSITORY.into(),
        TOKEN.into(),
    ));
    suite::execute_all_storage_tests(&mut storage);
}
