// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::transaction::ScriptABI;
use serde_generate as serdegen;
use serde_generate::SourceInstaller;
use serde_reflection::Registry;
use std::{io::Write, process::Command};
use tempfile::tempdir;
use transaction_builder_generator as buildgen;

fn get_libra_registry() -> Registry {
    let path = "../../testsuite/generate-format/tests/staged/libra.yaml";
    let content = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str::<Registry>(content.as_str()).unwrap()
}

fn get_stdlib_script_abis() -> Vec<ScriptABI> {
    let path = "../stdlib/compiled/transaction_scripts/abi";
    buildgen::read_abis(path).expect("reading ABI files should not fail")
}

#[test]
#[ignore]
fn test_that_python_code_parses() {
    let registry = get_libra_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let installer =
        serdegen::python3::Installer::new(dir.path().to_path_buf(), /* package */ None);
    installer.install_module("libra_types", &registry).unwrap();
    installer.install_serde_runtime().unwrap();

    let dir_path = dir.path().join("libra_builders");
    std::fs::create_dir_all(dir_path.clone()).unwrap();
    let source_path = dir_path.join("__init__.py");

    let mut source = std::fs::File::create(&source_path).unwrap();
    buildgen::python3::output(&mut source, &abis).unwrap();

    let python_path = format!(
        "{}:{}",
        std::env::var("PYTHONPATH").unwrap_or_default(),
        dir.path().to_string_lossy(),
    );
    let output = Command::new("python3")
        .arg("-c")
        .arg("import serde_types; import libra_types; import libra_builders")
        .env("PYTHONPATH", python_path)
        .output()
        .unwrap();
    assert_eq!(String::new(), String::from_utf8_lossy(&output.stderr));
    assert!(output.status.success());
}

#[test]
fn test_that_rust_code_compiles() {
    let registry = get_libra_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let installer = serdegen::rust::Installer::new(dir.path().to_path_buf());
    installer.install_module("libra-types", &registry).unwrap();

    let dir_path = dir.path().join("libra-builders");
    std::fs::create_dir_all(dir_path.clone()).unwrap();

    let mut cargo = std::fs::File::create(&dir_path.join("Cargo.toml")).unwrap();
    write!(
        cargo,
        r#"[package]
name = "libra-builders"
version = "0.1.0"
edition = "2018"

[dependencies]
libra-types = {{ path = "../libra-types", version = "0.1.0" }}
"#,
    )
    .unwrap();
    std::fs::create_dir(dir_path.join("src")).unwrap();
    let source_path = dir_path.join("src/lib.rs");
    let mut source = std::fs::File::create(&source_path).unwrap();
    buildgen::rust::output(&mut source, &abis, /* local types */ false).unwrap();

    // Use a stable `target` dir to avoid downloading and recompiling crates everytime.
    let target_dir = std::env::current_dir().unwrap().join("../../target");
    let status = Command::new("cargo")
        .current_dir(dir.path().join("libra-builders"))
        .arg("build")
        .arg("--target-dir")
        .arg(target_dir)
        .status()
        .unwrap();
    assert!(status.success());
}
