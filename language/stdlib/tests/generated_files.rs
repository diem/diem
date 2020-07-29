// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

fn assert_that_version_control_has_no_unstaged_changes() {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .unwrap();
    assert!(
        output.stdout.is_empty(),
        "Git repository should be in a clean state"
    );
    assert!(output.status.success());
}

#[test]
fn test_that_generated_file_are_up_to_date_in_git() {
    // Better not run the `stdlib` tool when the repository is not in a clean state.
    assert_that_version_control_has_no_unstaged_changes();

    assert!(Command::new("cargo")
        .current_dir("../..")
        .arg("run")
        .arg("--release")
        .arg("-p")
        .arg("stdlib")
        .status()
        .unwrap()
        .success());

    // Running the stdlib tool should not create unstaged changes.
    assert_that_version_control_has_no_unstaged_changes();
}
