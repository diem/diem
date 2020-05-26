// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use move_core_types::fs::AFS;
use move_lang::{
    command_line::{self as cli},
    shared::*,
};
use structopt::*;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Move Check",
    about = "Check Move source code, without compiling to bytecode."
)]
pub struct Options {
    /// The source files to check
    #[structopt(name = "PATH_TO_SOURCE_FILE")]
    pub source_files: Vec<String>,

    /// The library files needed as dependencies
    #[structopt(
        name = "PATH_TO_DEPENDENCY_FILE",
        short = cli::DEPENDENCY_SHORT,
        long = cli::DEPENDENCY,
    )]
    pub dependencies: Vec<String>,

    /// The sender address for modules and scripts
    #[structopt(
        name = "ADDRESS",
        short = cli::SENDER_SHORT,
        long = cli::SENDER,
        parse(try_from_str = cli::parse_address)
    )]
    pub sender: Option<Address>,
}

pub fn main() -> anyhow::Result<()> {
    let Options {
        source_files,
        dependencies,
        sender,
    } = Options::from_args();
    let fs = AFS::new();
    move_lang::move_check(&source_files, &dependencies, sender, &fs)
}
