// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use move_lang::{
    command_line::{self as cli},
    shared::{self, verify_and_create_named_address_mapping, AddressBytes, Flags},
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

    /// Named address mapping
    #[structopt(
        name = "NAMED_ADDRESSES",
        short = "a",
        long = "addresses",
        parse(try_from_str = shared::parse_named_address)
    )]
    pub named_addresses: Vec<(String, AddressBytes)>,

    #[structopt(flatten)]
    pub flags: Flags,
}

pub fn main() -> anyhow::Result<()> {
    let Options {
        source_files,
        dependencies,
        out_dir,
        emit_source_map,
        flags,
        named_addresses,
    } = Options::from_args();

    let interface_files_dir = format!("{}/generated_interface_files", out_dir);
    let (files, compiled_units) = move_lang::Compiler::new(&source_files, &dependencies)
        .set_interface_files_dir(interface_files_dir)
        .set_flags(flags)
        .set_named_address_values(verify_and_create_named_address_mapping(named_addresses)?)
        .build_and_report()?;
    move_lang::output_compiled_units(emit_source_map, files, compiled_units, &out_dir)
}
