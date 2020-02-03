// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use move_lang::command_line::{self as cli};
use move_lang::shared::*;
use structopt::*;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Move Check",
    about = "Check Move source code, without compiling to bytecode."
)]
pub struct Options {
    /// The source files to check
    #[structopt(
        name = "PATH_TO_SOURCE_FILE",
        short = cli::SOURCE_FILES_SHORT,
        long = cli::SOURCE_FILES,
    )]
    pub source_files: Vec<String>,

    /// The library files needed as dependencies
    #[structopt(
        name = "PATH_TO_DEPENDENCY_FILE",
        short = cli::DEPENDENCIES_SHORT,
        long = cli::DEPENDENCIES,
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

pub fn main() -> std::io::Result<()> {
    let Options {
        source_files,
        dependencies,
        sender,
    } = Options::from_args();
    move_lang::move_check(&source_files, &dependencies, sender)
}
