// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

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

    /// The output directory for saved artifacts, namely any 'move' interface files generated from
    /// 'mv' files
    #[structopt(
        name = "PATH_TO_OUTPUT_DIRECTORY",
        short = cli::OUT_DIR_SHORT,
        long = cli::OUT_DIR,
    )]
    pub out_dir: Option<String>,
}

pub fn main() -> anyhow::Result<()> {
    let Options {
        source_files,
        dependencies,
        sender,
        out_dir,
    } = Options::from_args();

    let _files = move_lang::move_check_and_report(&source_files, &dependencies, sender, out_dir)?;
    Ok(())
}
