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
#[structopt(name = "Move Build", about = "Compile Move source to Move bytecode.")]
pub struct Options {
    /// The source files to check and compile
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

    /// The Move bytecode output directory
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

pub fn main() -> anyhow::Result<()> {
    let Options {
        source_files,
        dependencies,
        sender,
        out_dir,
        emit_source_map,
    } = Options::from_args();
    let fs = AFS::new();
    let (files, compiled_units) =
        move_lang::move_compile(&source_files, &dependencies, sender, &fs)?;
    move_lang::output_compiled_units(emit_source_map, files, compiled_units, &out_dir, &fs)
}
