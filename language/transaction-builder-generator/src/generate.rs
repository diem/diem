// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! # Code generator for Move script builders
//!
//! '''bash
//! cargo run -p transaction-builder-generator -- --help
//! '''

use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};
use transaction_builder_generator::{python3, read_abis, rust};

arg_enum! {
#[derive(Debug, StructOpt)]
enum Language {
    Python3,
    Rust,
}
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Transaction builder generator",
    about = "Generate code for Move script builders"
)]
struct Options {
    #[structopt(long, possible_values = &Language::variants(), case_insensitive = true, default_value = "Python3")]
    language: Language,

    abi_directory: PathBuf,
}

fn main() {
    let options = Options::from_args();
    let abis = read_abis(options.abi_directory).expect("Failed to read ABI in directory");

    let mut out = std::io::stdout();

    match options.language {
        Language::Python3 => python3::output(&mut out, &abis).unwrap(),
        Language::Rust => rust::output(&mut out, &abis, /* local types */ false).unwrap(),
    }
}
