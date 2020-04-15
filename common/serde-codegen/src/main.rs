// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_reflection::RegistryOwned;
use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};

mod cpp;
mod python3;
mod rust;

arg_enum! {
#[derive(Debug, StructOpt)]
enum Language {
    Python3,
    Cpp,
    Rust,
}
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Serde code generator",
    about = "Generate code for Serde containers"
)]
struct Options {
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    #[structopt(long, possible_values = &Language::variants(), default_value = "Python3")]
    language: Language,
}

fn main() {
    let options = Options::from_args();
    let content =
        std::fs::read_to_string(options.input.as_os_str()).expect("input file must be readable");
    let registry = serde_yaml::from_str::<RegistryOwned>(content.as_str()).unwrap();

    match options.language {
        Language::Python3 => python3::output(&registry),
        Language::Cpp => cpp::output(&registry),
        Language::Rust => rust::output(&registry),
    }
}
