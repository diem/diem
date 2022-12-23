// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::transaction::ScriptABI;
use serde_generate as serdegen;
use serde_generate::SourceInstaller as _;
use serde_reflection::Registry;
use std::{io::Write, path::Path, process::Command};
use tempfile::tempdir;
use transaction_builder_generator as buildgen;
use transaction_builder_generator::SourceInstaller as _;

fn get_diem_registry() -> Registry {
    let path = "../../../testsuite/generate-format/tests/staged/diem.yaml";
    let content = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str::<Registry>(content.as_str()).unwrap()
}

fn get_tx_script_abis() -> Vec<ScriptABI> {
    // This is also a custom rule in diem/x.toml.
    let legacy_path = Path::new("../../diem-framework/releases/legacy/script_abis");
    buildgen::read_abis(&[legacy_path]).expect("reading legacy ABI files should not fail")
}

fn get_script_fun_abis() -> Vec<ScriptABI> {
    let new_abis = Path::new("../../diem-framework/releases/artifacts/current/script_abis");
    buildgen::read_abis(&[new_abis]).expect("reading new ABI files should not fail")
}

fn get_stdlib_script_abis() -> Vec<ScriptABI> {
    let mut abis = get_tx_script_abis();
    abis.extend(get_script_fun_abis().into_iter());
    abis
}

const EXPECTED_OUTPUT: &str = "224 1 161 28 235 11 1 0 0 0 7 1 0 2 2 2 4 3 6 16 4 22 2 5 24 29 7 53 96 8 149 1 16 0 0 0 1 1 0 0 2 0 1 0 0 3 2 3 1 1 0 4 1 3 0 1 5 1 6 12 1 8 0 5 6 8 0 5 3 10 2 10 2 0 5 6 12 5 3 10 2 10 2 1 9 0 11 68 105 101 109 65 99 99 111 117 110 116 18 87 105 116 104 100 114 97 119 67 97 112 97 98 105 108 105 116 121 27 101 120 116 114 97 99 116 95 119 105 116 104 100 114 97 119 95 99 97 112 97 98 105 108 105 116 121 8 112 97 121 95 102 114 111 109 27 114 101 115 116 111 114 101 95 119 105 116 104 100 114 97 119 95 99 97 112 97 98 105 108 105 116 121 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 1 1 4 1 12 11 0 17 0 12 5 14 5 10 1 10 2 11 3 11 4 56 0 11 5 17 2 2 1 7 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 3 88 68 88 3 88 68 88 0 4 3 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 1 135 214 18 0 0 0 0 0 4 0 4 0 \n3 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 14 80 97 121 109 101 110 116 83 99 114 105 112 116 115 26 112 101 101 114 95 116 111 95 112 101 101 114 95 119 105 116 104 95 109 101 116 97 100 97 116 97 1 7 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 3 88 68 88 3 88 68 88 0 4 16 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 8 135 214 18 0 0 0 0 0 1 0 1 0 \n";

const EXPECTED_TX_SCRIPT_OUTPUT: &str = "224 1 161 28 235 11 1 0 0 0 7 1 0 2 2 2 4 3 6 16 4 22 2 5 24 29 7 53 96 8 149 1 16 0 0 0 1 1 0 0 2 0 1 0 0 3 2 3 1 1 0 4 1 3 0 1 5 1 6 12 1 8 0 5 6 8 0 5 3 10 2 10 2 0 5 6 12 5 3 10 2 10 2 1 9 0 11 68 105 101 109 65 99 99 111 117 110 116 18 87 105 116 104 100 114 97 119 67 97 112 97 98 105 108 105 116 121 27 101 120 116 114 97 99 116 95 119 105 116 104 100 114 97 119 95 99 97 112 97 98 105 108 105 116 121 8 112 97 121 95 102 114 111 109 27 114 101 115 116 111 114 101 95 119 105 116 104 100 114 97 119 95 99 97 112 97 98 105 108 105 116 121 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 1 1 4 1 12 11 0 17 0 12 5 14 5 10 1 10 2 11 3 11 4 56 0 11 5 17 2 2 1 7 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 3 88 68 88 3 88 68 88 0 4 3 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 1 135 214 18 0 0 0 0 0 4 0 4 0 \n";

const EXPECTED_SCRIPT_FUN_OUTPUT: &str = "3 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 14 80 97 121 109 101 110 116 83 99 114 105 112 116 115 26 112 101 101 114 95 116 111 95 112 101 101 114 95 119 105 116 104 95 109 101 116 97 100 97 116 97 1 7 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 3 88 68 88 3 88 68 88 0 4 16 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 34 8 135 214 18 0 0 0 0 0 1 0 1 0 \n";

// Cannot run this test in the CI of Diem.
#[test]
#[ignore]
fn test_that_python_code_parses_and_passes_pyre_check() {
    let registry = get_diem_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let src_dir_path = dir.path().join("src");
    let installer =
        serdegen::python3::Installer::new(src_dir_path.clone(), /* package */ None);
    let paths = std::fs::read_dir("examples/python3/custom_diem_code")
        .unwrap()
        .map(|e| e.unwrap().path());
    let config = serdegen::CodeGeneratorConfig::new("diem_types".to_string())
        .with_encodings(vec![serdegen::Encoding::Bcs])
        .with_custom_code(buildgen::read_custom_code_from_paths(
            &["diem_types"],
            paths,
        ));
    installer.install_module(&config, &registry).unwrap();
    installer.install_serde_runtime().unwrap();
    installer.install_bcs_runtime().unwrap();

    let stdlib_dir_path = src_dir_path.join("diem_framework");
    std::fs::create_dir_all(stdlib_dir_path.clone()).unwrap();
    let source_path = stdlib_dir_path.join("__init__.py");

    let mut source = std::fs::File::create(&source_path).unwrap();
    buildgen::python3::output(&mut source, None, None, &abis).unwrap();

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
    eprintln!("{}", std::str::from_utf8(&output.stderr).unwrap());
    assert!(output.status.success());
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        EXPECTED_OUTPUT
    );

    let site_packages = Command::new("python3")
        .arg("-c")
        .arg("import os; import numpy; print(os.path.dirname(numpy.__path__[0]), end='')")
        .output()
        .unwrap()
        .stdout;

    let local_bin_path = which::which("pyre")
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let status = Command::new("pyre")
        .current_dir(dir.path())
        .arg("--source-directory")
        .arg("src")
        .arg("--noninteractive")
        .arg("--binary")
        .arg(local_bin_path.join("pyre.bin"))
        .arg("--typeshed")
        .arg(local_bin_path.join("../lib/pyre_check/typeshed"))
        .arg("--search-path")
        .arg(String::from_utf8_lossy(&site_packages).as_ref())
        .arg("check")
        .status()
        .unwrap();
    assert!(status.success());
}

fn test_rust(abis: &[ScriptABI], demo_file: &str, expected_output: &str) {
    let registry = get_diem_registry();
    let dir = tempdir().unwrap();

    let installer = serdegen::rust::Installer::new(dir.path().to_path_buf());
    let config = serdegen::CodeGeneratorConfig::new("diem-types".to_string());
    installer.install_module(&config, &registry).unwrap();

    let stdlib_dir_path = dir.path().join("diem-framework");
    std::fs::create_dir_all(stdlib_dir_path.clone()).unwrap();

    let mut cargo = std::fs::File::create(&stdlib_dir_path.join("Cargo.toml")).unwrap();
    write!(
        cargo,
        r#"[package]
name = "diem-framework"
version = "0.1.0"
edition = "2018"

[dependencies]
diem-types = {{ path = "../diem-types", version = "0.1.0" }}
serde_bytes = "0.11"
serde = {{ version = "1.0.114", features = ["derive"] }}
bcs = {{ git = "https://github.com/diem/bcs", rev = "30ce9f4ac51342d2fb4c04c4f5b40683d9652dc6" }}
once_cell = "1.4.0"

[[bin]]
name = "stdlib_demo"
path = "src/stdlib_demo.rs"
test = false
"#
    )
    .unwrap();
    std::fs::create_dir(stdlib_dir_path.join("src")).unwrap();
    let source_path = stdlib_dir_path.join("src/lib.rs");
    let mut source = std::fs::File::create(&source_path).unwrap();
    buildgen::rust::output(&mut source, abis, /* local types */ false).unwrap();

    std::fs::copy(demo_file, stdlib_dir_path.join("src/stdlib_demo.rs")).unwrap();

    // Use a stable `target` dir to avoid downloading and recompiling crates everytime.
    let target_dir = std::env::current_dir().unwrap().join("../../target");
    let status = Command::new("cargo")
        .current_dir(dir.path().join("diem-framework"))
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
        expected_output
    );
}

#[test]
fn test_that_rust_tx_script_code_compiles() {
    test_rust(
        &get_tx_script_abis(),
        "examples/rust/tx_script_demo.rs",
        EXPECTED_TX_SCRIPT_OUTPUT,
    );
}

#[test]
fn test_that_rust_script_fun_code_compiles() {
    test_rust(
        &get_script_fun_abis(),
        "examples/rust/script_fun_demo.rs",
        EXPECTED_SCRIPT_FUN_OUTPUT,
    );
}

#[test]
#[ignore]
fn test_that_cpp_code_compiles_and_demo_runs() {
    let registry = get_diem_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let config = serdegen::CodeGeneratorConfig::new("diem_types".to_string())
        .with_encodings(vec![serdegen::Encoding::Bcs]);
    let bcs_installer = serdegen::cpp::Installer::new(dir.path().to_path_buf());
    bcs_installer.install_module(&config, &registry).unwrap();
    bcs_installer.install_serde_runtime().unwrap();
    bcs_installer.install_bcs_runtime().unwrap();

    let abi_installer = buildgen::cpp::Installer::new(dir.path().to_path_buf());
    abi_installer
        .install_transaction_builders("diem_framework", &abis)
        .unwrap();

    std::fs::copy(
        "examples/cpp/stdlib_demo.cpp",
        dir.path().join("stdlib_demo.cpp"),
    )
    .unwrap();

    let status = Command::new("clang++")
        .arg("--std=c++17")
        .arg("-g")
        .arg(dir.path().join("diem_framework.cpp"))
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
    let registry = get_diem_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let paths = std::fs::read_dir("examples/java/custom_diem_code")
        .unwrap()
        .map(|e| e.unwrap().path());
    let config = serdegen::CodeGeneratorConfig::new("com.diem.types".to_string())
        .with_encodings(vec![serdegen::Encoding::Bcs])
        .with_custom_code(buildgen::read_custom_code_from_paths(
            &["com", "diem", "types"],
            paths,
        ));
    let bcs_installer = serdegen::java::Installer::new(dir.path().to_path_buf());
    bcs_installer.install_module(&config, &registry).unwrap();
    bcs_installer.install_serde_runtime().unwrap();
    bcs_installer.install_bcs_runtime().unwrap();

    let abi_installer = buildgen::java::Installer::new(dir.path().to_path_buf());
    abi_installer
        .install_transaction_builders("com.diem.stdlib", &abis)
        .unwrap();

    std::fs::copy(
        "examples/java/StdlibDemo.java",
        dir.path().join("StdlibDemo.java"),
    )
    .unwrap();

    let paths = || {
        std::iter::empty()
            .chain(std::fs::read_dir(dir.path().join("com/novi/serde")).unwrap())
            .chain(std::fs::read_dir(dir.path().join("com/novi/bcs")).unwrap())
            .chain(std::fs::read_dir(dir.path().join("com/diem/types")).unwrap())
            .chain(std::fs::read_dir(dir.path().join("com/diem/stdlib")).unwrap())
            .map(|e| e.unwrap().path())
            .chain(std::iter::once(dir.path().join("StdlibDemo.java")))
    };

    let status = Command::new("javadoc")
        .arg("-sourcepath")
        .arg(dir.path())
        .arg("-d")
        .arg(dir.path().join("html"))
        .args(paths())
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("javac")
        .arg("-cp")
        .arg(dir.path())
        .arg("-d")
        .arg(dir.path())
        .args(paths())
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
    assert_eq!(std::str::from_utf8(&output.stderr).unwrap(), String::new());
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        EXPECTED_OUTPUT
    );
    assert!(output.status.success());
}

#[test]
#[ignore]
fn test_that_csharp_code_compiles_and_demo_runs() {
    let registry = get_diem_registry();
    let abis = get_stdlib_script_abis();
    // Special case this because of what the default tempdir is on a mac
    // It looks as if the path string might be too long for the dotnet runtime
    // to execute correctly because you get funny errors that don't occur when
    // the path string is shorter. So make the temp path shorter and all is good.
    // Avoids this:
    // "Unhandled exception. System.IO.FileNotFoundException: Could not load file
    // or assembly \'Diem.Types, Version=1.0.0.0, Culture=neutral,
    // PublicKeyToken=null\'. The system cannot find the file specified.\n\n
    // File name: \'Diem.Types, Version=1.0.0.0, Culture=neutral,
    // PublicKeyToken=null\'\n\n\n"`,
    if std::env::consts::OS == "macos" {
        std::env::set_var("TMPDIR", "/private/tmp/");
    }
    let dir = tempdir().unwrap();

    let paths = std::fs::read_dir("examples/csharp/custom_diem_code")
        .unwrap()
        .map(|e| e.unwrap().path());
    let config = serdegen::CodeGeneratorConfig::new("Diem.Types".to_string())
        .with_encodings(vec![serdegen::Encoding::Bcs])
        .with_custom_code(buildgen::read_custom_code_from_paths(
            &["Diem", "Types"],
            paths,
        ));
    let bcs_installer = serdegen::csharp::Installer::new(dir.path().to_path_buf());
    bcs_installer.install_module(&config, &registry).unwrap();
    bcs_installer.install_serde_runtime().unwrap();
    bcs_installer.install_bcs_runtime().unwrap();

    let abi_installer = buildgen::csharp::Installer::new(dir.path().to_path_buf());
    abi_installer
        .install_transaction_builders("Diem.Stdlib", &abis)
        .unwrap();

    std::fs::create_dir(dir.path().join("Demo")).unwrap();
    std::fs::copy(
        "examples/csharp/StdlibDemo.cs",
        dir.path().join("Demo/StdlibDemo.cs"),
    )
    .unwrap();

    let status = Command::new("dotnet")
        .arg("new")
        .arg("classlib")
        .arg("-n")
        .arg("Diem.Stdlib")
        .arg("-o")
        .arg(dir.path().join("Diem/Stdlib"))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("rm")
        .arg(dir.path().join("Diem/Stdlib/Class1.cs"))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("dotnet")
        .arg("add")
        .arg(dir.path().join("Diem/Stdlib/Diem.Stdlib.csproj"))
        .arg("reference")
        .arg(dir.path().join("Diem/Types/Diem.Types.csproj"))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("dotnet")
        .arg("new")
        .arg("sln")
        .arg("-n")
        .arg("Demo")
        .arg("-o")
        .arg(dir.path().join("Demo"))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("dotnet")
        .arg("new")
        .arg("console")
        .arg("-n")
        .arg("Demo")
        .arg("-o")
        .arg(dir.path().join("Demo"))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("rm")
        .arg(dir.path().join("Demo/Program.cs"))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("dotnet")
        .arg("add")
        .arg(dir.path().join("Demo/Demo.csproj"))
        .arg("reference")
        .arg(dir.path().join("Diem/Stdlib/Diem.Stdlib.csproj"))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("dotnet")
        .arg("add")
        .arg(dir.path().join("Demo/Demo.csproj"))
        .arg("reference")
        .arg(dir.path().join("Diem/Types/Diem.Types.csproj"))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("dotnet")
        .arg("add")
        .arg(dir.path().join("Demo/Demo.csproj"))
        .arg("reference")
        .arg(dir.path().join("Serde/Serde.csproj"))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("dotnet")
        .arg("add")
        .arg(dir.path().join("Demo/Demo.csproj"))
        .arg("reference")
        .arg(dir.path().join("Bcs/Bcs.csproj"))
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("dotnet")
        .arg("build")
        .arg(dir.path().join("Demo/Demo.csproj"))
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new("dotnet")
        .arg("run")
        .arg("--project")
        .arg(dir.path().join("Demo/Demo.csproj"))
        .output()
        .unwrap();
    assert_eq!(std::str::from_utf8(&output.stderr).unwrap(), String::new());
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        EXPECTED_OUTPUT
    );
    assert!(output.status.success());
}

#[test]
#[ignore]
fn test_that_golang_code_compiles_and_demo_runs() {
    let registry = get_diem_registry();
    let abis = get_stdlib_script_abis();
    let dir = tempdir().unwrap();

    let config = serdegen::CodeGeneratorConfig::new("diemtypes".to_string())
        .with_encodings(vec![serdegen::Encoding::Bcs]);
    let bcs_installer = serdegen::golang::Installer::new(
        dir.path().to_path_buf(),
        /* default Serde module */ None,
    );
    bcs_installer.install_module(&config, &registry).unwrap();

    let abi_installer = buildgen::golang::Installer::new(
        dir.path().to_path_buf(),
        /* default Serde module */ None,
        Some("testing".to_string()),
    );
    abi_installer
        .install_transaction_builders("diemstdlib", &abis)
        .unwrap();

    std::fs::copy(
        "examples/golang/stdlib_demo.go",
        dir.path().join("stdlib_demo.go"),
    )
    .unwrap();

    let status = Command::new("go")
        .current_dir(dir.path())
        .arg("mod")
        .arg("init")
        .arg("testing")
        .status()
        .unwrap();
    assert!(status.success());

    let status = Command::new("go")
        .current_dir(dir.path())
        .arg("mod")
        .arg("edit")
        .arg("-replace")
        .arg(format!("testing={}", dir.path().to_string_lossy(),))
        .status()
        .unwrap();
    assert!(status.success());

    let output = Command::new("go")
        .current_dir(dir.path())
        .arg("run")
        .arg(dir.path().join("stdlib_demo.go"))
        .output()
        .unwrap();
    eprintln!("{}", std::str::from_utf8(&output.stderr).unwrap());
    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        EXPECTED_OUTPUT
    );
    assert!(output.status.success());
}
