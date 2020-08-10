// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_that_installed_rust_code_compiles() {
    let dir = tempdir().unwrap();

    let status = Command::new("cargo")
        .current_dir("../../..")
        .arg("run")
        .arg("-p")
        .arg("transaction-builder-generator")
        .arg("--")
        .arg("--language")
        .arg("rust")
        .arg("--module-name")
        .arg("libra-stdlib:0.1.1")
        .arg("--with-libra-types")
        .arg("testsuite/generate-format/tests/staged/libra.yaml")
        .arg("--target-source-dir")
        .arg(dir.path())
        .arg("language/stdlib/compiled/transaction_scripts/abi")
        .status()
        .unwrap();
    assert!(status.success());

    // Use a stable `target` dir to avoid downloading and recompiling crates everytime.
    let target_dir = std::env::current_dir().unwrap().join("../../target");
    let status = Command::new("cargo")
        .current_dir(dir.path().join("libra-stdlib"))
        .arg("build")
        .arg("--target-dir")
        .arg(target_dir)
        .status()
        .unwrap();
    assert!(status.success());
}
