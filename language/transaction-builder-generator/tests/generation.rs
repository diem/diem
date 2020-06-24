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

// Cannot run this test in the CI of Libra.
#[test]
#[ignore]
fn test_that_python_code_parses_and_passes_pyre_check() {
    let registry = get_libra_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let src_dir_path = dir.path().join("src");
    let installer =
        serdegen::python3::Installer::new(src_dir_path.clone(), /* package */ None);
    installer.install_module("libra_types", &registry).unwrap();
    installer.install_serde_runtime().unwrap();

    let builder_dir_path = src_dir_path.join("libra_builders");
    std::fs::create_dir_all(builder_dir_path.clone()).unwrap();
    let source_path = builder_dir_path.join("__init__.py");

    let mut source = std::fs::File::create(&source_path).unwrap();
    buildgen::python3::output(&mut source, &abis).unwrap();

    let python_path = format!(
        "{}:{}",
        std::env::var("PYTHONPATH").unwrap_or_default(),
        src_dir_path.to_string_lossy(),
    );
    let status = Command::new("python3")
        .arg("-c")
        .arg("import serde_types; import libra_types; import libra_builders")
        .env("PYTHONPATH", python_path)
        .status()
        .unwrap();
    assert!(status.success());

    // This temporarily requires a checkout of serde-reflection.git next to libra.git
    // Hopefully, numpy's next release will include typeshed (.pyi) files and we will only
    // require a local install of numpy (on top of python3 and pyre).
    let status = Command::new("cp")
        .arg("-r")
        .arg("../../../serde-reflection/serde-generate/runtime/python/typeshed")
        .arg(dir.path())
        .status()
        .unwrap();
    assert!(status.success());

    let mut pyre_config = std::fs::File::create(dir.path().join(".pyre_configuration")).unwrap();
    writeln!(
        &mut pyre_config,
        r#"{{
  "source_directories": [
    "src"
  ],
  "search_path": [
    "typeshed"
  ]
}}"#,
    )
    .unwrap();

    let status = Command::new("pyre")
        .current_dir(dir.path())
        .arg("check")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn test_that_rust_code_compiles() {
    let registry = get_libra_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let installer = serdegen::rust::Installer::new(dir.path().to_path_buf());
    installer.install_module("libra-types", &registry).unwrap();

    let builder_dir_path = dir.path().join("libra-builders");
    std::fs::create_dir_all(builder_dir_path.clone()).unwrap();

    let mut cargo = std::fs::File::create(&builder_dir_path.join("Cargo.toml")).unwrap();
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
    std::fs::create_dir(builder_dir_path.join("src")).unwrap();
    let source_path = builder_dir_path.join("src/lib.rs");
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
