// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::transaction::ScriptABI;
use serde_generate as serdegen;
use serde_generate::SourceInstaller as _;
use serde_reflection::Registry;
use std::{io::Write, process::Command};
use tempfile::tempdir;
use transaction_builder_generator as buildgen;
use transaction_builder_generator::SourceInstaller as _;

fn get_libra_registry() -> Registry {
    let path = "../../../testsuite/generate-format/tests/staged/libra.yaml";
    let content = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str::<Registry>(content.as_str()).unwrap()
}

fn get_stdlib_script_abis() -> Vec<ScriptABI> {
    let path = "../../stdlib/compiled/transaction_scripts/abi";
    buildgen::read_abis(path).expect("reading ABI files should not fail")
}

const EXPECTED_OUTPUT : &str = "225 1 161 28 235 11 1 0 0 0 7 1 0 2 2 2 4 3 6 16 4 22 2 5 24 29 7 53 97 8 150 1 16 0 0 0 1 1 0 0 2 0 1 0 0 3 2 3 1 1 0 4 1 3 0 1 5 1 6 12 1 8 0 5 6 8 0 5 3 10 2 10 2 0 5 6 12 5 3 10 2 10 2 1 9 0 12 76 105 98 114 97 65 99 99 111 117 110 116 18 87 105 116 104 100 114 97 119 67 97 112 97 98 105 108 105 116 121 27 101 120 116 114 97 99 116 95 119 105 116 104 100 114 97 119 95 99 97 112 97 98 105 108 105 116 121 8 112 97 121 95 102 114 111 109 27 114 101 115 116 111 114 101 95 119 105 116 104 100 114 97 119 95 99 97 112 97 98 105 108 105 116 121 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 1 1 4 1 12 11 0 17 0 12 5 14 5 10 1 10 2 11 3 11 4 56 0 11 5 17 2 2 1 7 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 3 76 66 82 3 76 66 82 0 4 3 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 1 135 214 18 0 0 0 0 0 4 0 4 0 \n";

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
    installer.install_lcs_runtime().unwrap();

    let stdlib_dir_path = src_dir_path.join("libra_stdlib");
    std::fs::create_dir_all(stdlib_dir_path.clone()).unwrap();
    let source_path = stdlib_dir_path.join("__init__.py");

    let mut source = std::fs::File::create(&source_path).unwrap();
    buildgen::python3::output(&mut source, &abis).unwrap();

    std::fs::copy(
        "examples/python3/stdlib_demo.py",
        dir.path().join("src/stdlib_demo.py"),
    )
    .unwrap();

    let python_path = format!(
        "{}:{}",
        std::env::var("PYTHONPATH").unwrap_or_default(),
        src_dir_path.to_string_lossy(),
    );
    let output = Command::new("python3")
        .env("PYTHONPATH", python_path)
        .arg(dir.path().join("src/stdlib_demo.py"))
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        EXPECTED_OUTPUT
    );

    // This temporarily requires a checkout of serde-reflection.git next to libra.git
    // Hopefully, numpy's next release will include typeshed (.pyi) files and we will only
    // require a local install of numpy (on top of python3 and pyre).
    let status = Command::new("cp")
        .arg("-r")
        .arg("../../../../serde-reflection/serde-generate/runtime/python/typeshed")
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

    let stdlib_dir_path = dir.path().join("libra-stdlib");
    std::fs::create_dir_all(stdlib_dir_path.clone()).unwrap();

    let mut cargo = std::fs::File::create(&stdlib_dir_path.join("Cargo.toml")).unwrap();
    write!(
        cargo,
        r#"[package]
name = "libra-stdlib"
version = "0.1.0"
edition = "2018"

[dependencies]
libra-types = {{ path = "../libra-types", version = "0.1.0" }}
serde_bytes = "0.11"
serde = {{ version = "1.0.114", features = ["derive"] }}
libra-canonical-serialization = {{ path = "{}", version = "0.1.0" }}
once_cell = "1.4.0"

[[bin]]
name = "stdlib_demo"
path = "src/stdlib_demo.rs"
test = false
"#,
        std::env::current_dir()
            .unwrap()
            .join("../../../common/lcs")
            .to_string_lossy()
    )
    .unwrap();
    std::fs::create_dir(stdlib_dir_path.join("src")).unwrap();
    let source_path = stdlib_dir_path.join("src/lib.rs");
    let mut source = std::fs::File::create(&source_path).unwrap();
    buildgen::rust::output(&mut source, &abis, /* local types */ false).unwrap();

    std::fs::copy(
        "examples/rust/stdlib_demo.rs",
        stdlib_dir_path.join("src/stdlib_demo.rs"),
    )
    .unwrap();

    // Use a stable `target` dir to avoid downloading and recompiling crates everytime.
    let target_dir = std::env::current_dir().unwrap().join("../../target");
    let status = Command::new("cargo")
        .current_dir(dir.path().join("libra-stdlib"))
        .arg("build")
        .arg("--target-dir")
        .arg(target_dir.clone())
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new(target_dir.join("debug/stdlib_demo"))
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        EXPECTED_OUTPUT
    );
}

#[test]
#[ignore]
fn test_that_cpp_code_compiles_and_demo_runs() {
    let registry = get_libra_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let lcs_installer = serdegen::cpp::Installer::new(dir.path().to_path_buf());
    lcs_installer
        .install_module("libra_types", &registry)
        .unwrap();
    lcs_installer.install_serde_runtime().unwrap();
    lcs_installer.install_lcs_runtime().unwrap();

    let abi_installer = buildgen::cpp::Installer::new(dir.path().to_path_buf());
    abi_installer
        .install_transaction_builders("libra_stdlib", &abis)
        .unwrap();

    std::fs::copy(
        "examples/cpp/stdlib_demo.cpp",
        dir.path().join("stdlib_demo.cpp"),
    )
    .unwrap();

    let status = Command::new("clang++")
        .arg("--std=c++17")
        .arg("-g")
        .arg(dir.path().join("libra_stdlib.cpp"))
        .arg(dir.path().join("stdlib_demo.cpp"))
        .arg("-o")
        .arg(dir.path().join("stdlib_demo"))
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new(dir.path().join("stdlib_demo"))
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        EXPECTED_OUTPUT
    );
}

#[test]
#[ignore]
fn test_that_java_code_compiles_and_demo_runs() {
    let registry = get_libra_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let lcs_installer = serdegen::java::Installer::new(dir.path().to_path_buf());
    lcs_installer
        .install_module("org.libra.types", &registry)
        .unwrap();
    lcs_installer.install_serde_runtime().unwrap();
    lcs_installer.install_lcs_runtime().unwrap();

    let abi_installer = buildgen::java::Installer::new(dir.path().to_path_buf());
    abi_installer
        .install_transaction_builders("org.libra.stdlib.Stdlib", &abis)
        .unwrap();

    std::fs::copy(
        "examples/java/StdlibDemo.java",
        dir.path().join("StdlibDemo.java"),
    )
    .unwrap();

    let paths = std::iter::empty()
        .chain(std::fs::read_dir(dir.path().join("com/facebook/serde")).unwrap())
        .chain(std::fs::read_dir(dir.path().join("com/facebook/lcs")).unwrap())
        .chain(std::fs::read_dir(dir.path().join("org/libra/types")).unwrap())
        .chain(std::fs::read_dir(dir.path().join("org/libra/stdlib")).unwrap())
        .map(|e| e.unwrap().path())
        .chain(std::iter::once(dir.path().join("StdlibDemo.java")));

    let status = Command::new("javac")
        .arg("-cp")
        .arg(dir.path())
        .arg("-d")
        .arg(dir.path())
        .args(paths)
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new("java")
        .arg("-enableassertions")
        .arg("-cp")
        .arg(dir.path())
        .arg("StdlibDemo")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        EXPECTED_OUTPUT
    );
}
