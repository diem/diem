use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestUnitError {
    #[error("tests failed for some crate (not crypto or testsuite)")]
    NonCrypto,
    #[error("tests failed for libra-crypto")]
    Crypto,
    #[error("crate failed")]
    Specific(String),
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

pub fn run(package: Option<String>) -> Result<(), Box<TestUnitError>> {
    if let Some(package) = package {
        if run_cargo_test_on_package(&package, package != "libra-crypto") {
            Ok(())
        } else {
            Err(Box::new(TestUnitError::Specific(package.to_string())))
        }
    } else {
        if !run_cargo_test_on_most_things() {
            return Err(Box::new(TestUnitError::NonCrypto));
        }
        if !run_cargo_test_on_package("libra-crypto", false) {
            return Err(Box::new(TestUnitError::Crypto));
        }
        Ok(())
    }
}

fn run_cargo_test_on_package(package: &str, with_all_features: bool) -> bool {
    let mut args = if with_all_features {
        vec!["test", "--all-features"]
    } else {
        vec!["test"]
    };
    args.push("-p");
    args.push(package);
    let output = Command::new("cargo")
        .current_dir(project_root())
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    output.status.success()
}

fn run_cargo_test_on_most_things() -> bool {
    let output = Command::new("cargo")
        .current_dir(project_root())
        .args(&[
            "test",
            "--all",
            "--all-features",
            "--exclude",
            "libra-crypto",
            "--exclude",
            "testsuite",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    output.status.success()
}
