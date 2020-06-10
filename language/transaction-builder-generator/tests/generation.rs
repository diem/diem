// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use generate_format::Corpus;
use serde_generate as serdegen;
use serde_generate::SourceInstaller;
use serde_reflection::Registry;
use std::process::Command;
use tempfile::tempdir;
use transaction_builder::get_stdlib_script_abis;
use transaction_builder_generator as buildgen;

fn get_libra_registry() -> Registry {
    let path =
        "../../testsuite/generate-format/".to_string() + Corpus::Libra.output_file().unwrap();
    let content = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str::<Registry>(content.as_str()).unwrap()
}

#[test]
#[ignore]
fn test_that_python_code_parses() {
    let registry = get_libra_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let installer = serdegen::python3::Installer::new(dir.path().to_path_buf());
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
