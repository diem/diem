// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! # Code generator for Move script builders
//!
//! '''bash
//! cargo run -p transaction-builder-generator -- --help
//! '''

use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};
use transaction_builder_generator::{cpp, python3, read_abis, rust, SourceInstaller};

arg_enum! {
#[derive(Debug, StructOpt)]
enum Language {
    Python3,
    Rust,
    Cpp,
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

    /// Module name for the transaction builders installed in the `target_source_dir`.
    #[structopt(long)]
    module_name: Option<String>,

    /// Optional package name where to find the `serde_types` module (useful in Python).
    #[structopt(long)]
    serde_package_name: Option<String>,

    /// Optional version number for the `serde_types` module (useful in Rust).
    #[structopt(long, default_value = "0.1.0")]
    serde_version_number: String,

    /// Optional package name where to find the `libra_types` module (useful in Python).
    #[structopt(long)]
    libra_package_name: Option<String>,
}

fn main() {
    let options = Options::from_args();
    let abis = read_abis(options.abi_directory).expect("Failed to read ABI in directory");

    match options.target_source_dir {
        None => {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            match options.language {
                Language::Python3 => python3::output(&mut out, &abis).unwrap(),
                Language::Rust => rust::output(&mut out, &abis, /* local types */ false).unwrap(),
                Language::Cpp => {
                    cpp::output(&mut out, &abis, options.module_name.as_deref()).unwrap()
                }
            }
        }
        Some(install_dir) => {
            let installer: Box<dyn SourceInstaller<Error = Box<dyn std::error::Error>>> =
                match options.language {
                    Language::Python3 => Box::new(python3::Installer::new(
                        install_dir,
                        options.serde_package_name,
                        options.libra_package_name,
                    )),
                    Language::Rust => Box::new(rust::Installer::new(
                        install_dir,
                        options.serde_version_number,
                    )),
                    Language::Cpp => Box::new(cpp::Installer::new(install_dir)),
                };

            if let Some(name) = options.module_name {
                installer
                    .install_transaction_builders(&name, &abis)
                    .unwrap();
            }
        }
    }
}
