// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use tempfile::tempdir;

#[test]
fn check_that_docs_are_updated() {
    let temp_dir = tempdir().unwrap();

    crate::build_stdlib_doc(&temp_dir.path().to_string_lossy());

    assert!(
        !dir_diff::is_different(&temp_dir, &crate::move_stdlib_docs_full_path()).unwrap(),
        "Generated docs differ from the ones checked in"
    )
}
