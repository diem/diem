// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use move_lang::command_line::{self as cli};
use move_lang::shared::*;
use structopt::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "Move Build", about = "Compile Move source to Move bytecode.")]
pub struct Options {
    /// The source files to check and compile
    #[structopt(
        name = "PATH_TO_SOURCE_FILE",
        short = cli::SOURCE_FILES_SHORT,
        long = cli::SOURCE_FILES,
        parse(from_str = cli::leak_str)
    )]
    pub source_files: Vec<&'static str>,

    /// The library files needed as dependencies
    #[structopt(
        name = "PATH_TO_DEPENDENCY_FILE",
        short = cli::DEPENDENCIES_SHORT,
        long = cli::DEPENDENCIES,
        parse(from_str = cli::leak_str)
    )]
    pub dependencies: Vec<&'static str>,

    /// The sender address for modules and scripts
    #[structopt(
        name = "ADDRESS",
        short = cli::SENDER_SHORT,
        long = cli::SENDER,
        parse(try_from_str = cli::parse_address)
    )]
    pub sender: Option<Address>,

    /// The directory for outputing move bytecode
    #[structopt(
        name = "PATH_TO_OUTPUT_DIRECTORY",
        short = cli::OUT_DIR_SHORT,
        long = cli::OUT_DIR,
        default_value = cli::DEFAULT_OUTPUT_DIR,
    )]
    pub out_dir: String,
}

pub fn main() -> std::io::Result<()> {
    let Options {
        source_files,
        dependencies,
        sender,
        out_dir,
    } = Options::from_args();
    let (files, compiled_units) = move_lang::move_compile(&source_files, &dependencies, sender)?;
    move_lang::output_compiled_units(files, compiled_units, &out_dir)
}
