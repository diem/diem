// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use move_lang::{
    command_line::{self as cli},
    shared::*,
};
use structopt::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "Move Build", about = "Compile Move source to Move bytecode.")]
pub struct Options {
    /// The source files to check and compile
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

    /// The directory for outputing move bytecode
    #[structopt(
        name = "PATH_TO_OUTPUT_DIRECTORY",
        short = cli::OUT_DIR_SHORT,
        long = cli::OUT_DIR,
        default_value = cli::DEFAULT_OUTPUT_DIR,
    )]
    pub out_dir: String,

    /// Save bytecode source map to disk
    #[structopt(
        name = "",
        short = cli::SOURCE_MAP_SHORT,
        long = cli::SOURCE_MAP,
    )]
    pub emit_source_map: bool,
}

pub fn main() {
    std::process::exit(match main_impl() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("error: {}", e);
            1
        }
    });
}

fn main_impl() -> std::io::Result<()> {
    let Options {
        source_files,
        dependencies,
        sender,
        out_dir,
        emit_source_map,
    } = Options::from_args();
    let (files, compiled_units) = move_lang::move_compile(&source_files, &dependencies, sender)?;
    move_lang::output_compiled_units(emit_source_map, files, compiled_units, &out_dir)
}
