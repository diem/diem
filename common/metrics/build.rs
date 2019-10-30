// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

/// Save revision info to environment variable
fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .unwrap();
    let git_rev = &String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_REVISION={}", git_rev);
}
