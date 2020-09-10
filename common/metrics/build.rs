// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{env, process::Command};

/// Save revision info to environment variable
fn main() {
    if env::var("GIT_REV").is_err() {
        let output = Command::new("git")
            .args(&["rev-parse", "--short", "HEAD"])
            .output()
            .unwrap();
        let git_rev = String::from_utf8(output.stdout).unwrap();
        println!("cargo:rustc-env=GIT_REV={}", git_rev);
    }
}
