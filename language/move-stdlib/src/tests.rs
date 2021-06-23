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

#[test]
fn check_that_the_errmap_is_updated() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();

    crate::build_error_code_map(&temp_file.path().to_string_lossy());

    assert!(
        file_diff::diff(
            &temp_file.path().to_string_lossy(),
            &crate::move_stdlib_errmap_full_path()
        ),
        "Generated errmap differ from the one checked in"
    );
}
