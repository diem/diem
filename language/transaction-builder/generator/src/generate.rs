// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! # Code generator for Move script builders
//!
//! '''bash
//! cargo run -p transaction-builder-generator -- --help
//! '''

use serde_generate as serdegen;
use serde_reflection::Registry;
use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};
use transaction_builder_generator as buildgen;

arg_enum! {
#[derive(Debug, StructOpt)]
enum Language {
    Python3,
    Rust,
    Cpp,
    Java,
}
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Transaction builder generator",
    about = "Generate code for Move script builders"
)]
struct Options {
    /// Path to the directory containing ABI files in LCS encoding.
    abi_directory: PathBuf,

    /// Language for code generation.
    #[structopt(long, possible_values = &Language::variants(), case_insensitive = true, default_value = "Python3")]
    language: Language,

    /// Directory where to write generated modules (otherwise print code on stdout).
    #[structopt(long)]
    target_source_dir: Option<PathBuf>,

    /// Also install the libra types described by the given YAML file, along with the LCS runtime.
    #[structopt(long)]
    with_libra_types: Option<PathBuf>,

    /// Module name for the transaction builders installed in the `target_source_dir`.
    /// Rust crates may contain a version number, e.g. "test:1.2.0".
    /// In Java, this is expected to be a class name, e.g. "com.test.Test" to create `com/test/Test.java`.
    #[structopt(long)]
    module_name: Option<String>,

    /// Optional package name where to find the `serde_types` module (useful in Python).
    #[structopt(long)]
    serde_package_name: Option<String>,

    /// Optional version number for the `libra_types` module (useful in Rust).
    /// If `--with-libra-types` is passed, this will be the version of the generated `libra_types` module.
    #[structopt(long, default_value = "0.1.0")]
    libra_version_number: String,

    /// Optional package name where to find the `libra_types` module (useful in Python).
    #[structopt(long)]
    libra_package_name: Option<String>,
}

fn main() {
    let options = Options::from_args();
    let abis = buildgen::read_abis(options.abi_directory).expect("Failed to read ABI in directory");

    let install_dir = match options.target_source_dir {
        None => {
            // Nothing to install. Just print to stdout.
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            match options.language {
                Language::Python3 => buildgen::python3::output(&mut out, &abis).unwrap(),
                Language::Rust => {
                    buildgen::rust::output(&mut out, &abis, /* local types */ false).unwrap()
                }
                Language::Cpp => {
                    buildgen::cpp::output(&mut out, &abis, options.module_name.as_deref()).unwrap()
                }
                Language::Java => {
                    let module_name = options.module_name.as_deref().unwrap_or("Helpers");
                    let parts = module_name.rsplitn(2, '.').collect::<Vec<_>>();
                    let (package_name, class_name) = if parts.len() > 1 {
                        (Some(parts[1]), parts[0])
                    } else {
                        (None, parts[0])
                    };
                    buildgen::java::output(&mut out, &abis, package_name, class_name).unwrap()
                }
            }
            return;
        }
        Some(dir) => dir,
    };

    // Libra types
    if let Some(registry_file) = options.with_libra_types {
        let installer: Box<dyn serdegen::SourceInstaller<Error = Box<dyn std::error::Error>>> =
            match options.language {
                Language::Python3 => Box::new(serdegen::python3::Installer::new(
                    install_dir.clone(),
                    options.serde_package_name.clone(),
                )),
                Language::Rust => Box::new(serdegen::rust::Installer::new(install_dir.clone())),
                Language::Cpp => Box::new(serdegen::cpp::Installer::new(install_dir.clone())),
                Language::Java => Box::new(serdegen::java::Installer::new(install_dir.clone())),
            };

        match options.language {
            Language::Rust => (), // In Rust, runtimes are deployed as crates.
            _ => {
                installer.install_serde_runtime().unwrap();
                installer.install_lcs_runtime().unwrap();
            }
        }
        let content =
            std::fs::read_to_string(registry_file).expect("registry file must be readable");
        let registry = serde_yaml::from_str::<Registry>(content.as_str()).unwrap();
        let name = match options.language {
            Language::Rust => {
                if options.libra_version_number == "0.1.0" {
                    "libra-types".to_string()
                } else {
                    format!("libra-types:{}", options.libra_version_number)
                }
            }
            Language::Java => "org.libra.types".to_string(),
            _ => "libra_types".to_string(),
        };
        installer.install_module(&name, &registry).unwrap();
    }

    // Transaction builders
    let installer: Box<dyn buildgen::SourceInstaller<Error = Box<dyn std::error::Error>>> =
        match options.language {
            Language::Python3 => Box::new(buildgen::python3::Installer::new(
                install_dir,
                options.serde_package_name,
                options.libra_package_name,
            )),
            Language::Rust => Box::new(buildgen::rust::Installer::new(
                install_dir,
                options.libra_version_number,
            )),
            Language::Cpp => Box::new(buildgen::cpp::Installer::new(install_dir)),
            Language::Java => Box::new(buildgen::java::Installer::new(install_dir)),
        };

    if let Some(name) = options.module_name {
        installer
            .install_transaction_builders(&name, &abis)
            .unwrap();
    }
}
