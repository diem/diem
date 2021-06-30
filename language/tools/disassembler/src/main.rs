// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytecode_source_map::{
    mapping::SourceMapping,
    utils::{remap_owned_loc_to_loc, source_map_from_file, OwnedLoc},
};
use disassembler::disassembler::{Disassembler, DisassemblerOptions};
use move_binary_format::{
    binary_views::BinaryIndexedView,
    file_format::{CompiledModule, CompiledScript},
};
use move_command_line_common::files::{
    MOVE_COMPILED_EXTENSION, MOVE_EXTENSION, SOURCE_MAP_EXTENSION,
};
use move_coverage::coverage_map::CoverageMap;
use move_ir_types::location::Spanned;
use std::{fs, path::Path};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Move Bytecode Disassembler",
    about = "Print a human-readable version of Move bytecode (.mv files)"
)]
struct Args {
    /// Skip printing of private functions.
    #[structopt(long = "skip-private")]
    pub skip_private: bool,

    /// Do not print the disassembled bytecodes of each function.
    #[structopt(long = "skip-code")]
    pub skip_code: bool,

    /// Do not print locals of each function.
    #[structopt(long = "skip-locals")]
    pub skip_locals: bool,

    /// Do not print the basic blocks of each function.
    #[structopt(long = "skip-basic-blocks")]
    pub skip_basic_blocks: bool,

    /// Treat input file as a script (default is to treat file as a module)
    #[structopt(short = "s", long = "script")]
    pub is_script: bool,

    /// The path to the bytecode file to disassemble; let's call it file.mv. We assume that two
    /// other files reside under the same directory: a source map file.mvsm (possibly) and the Move
    /// source code file.move.
    #[structopt(short = "b", long = "bytecode")]
    pub bytecode_file_path: String,

    /// (Optional) Path to a coverage file for the VM in order to print trace information in the
    /// disassembled output.
    #[structopt(short = "c", long = "move-coverage-path")]
    pub code_coverage_path: Option<String>,
}

fn main() {
    let args = Args::from_args();

    let move_extension = MOVE_EXTENSION;
    let mv_bytecode_extension = MOVE_COMPILED_EXTENSION;
    let source_map_extension = SOURCE_MAP_EXTENSION;

    let source_path = Path::new(&args.bytecode_file_path);
    let extension = source_path
        .extension()
        .expect("Missing file extension for bytecode file");
    if extension != mv_bytecode_extension {
        println!(
            "Bad source file extension {:?}; expected {}",
            extension, mv_bytecode_extension
        );
        std::process::exit(1);
    }

    let bytecode_bytes = fs::read(&args.bytecode_file_path).expect("Unable to read bytecode file");

    let source_path = Path::new(&args.bytecode_file_path).with_extension(move_extension);
    let source = fs::read_to_string(&source_path).ok();
    let source_map = source_map_from_file::<OwnedLoc>(
        &Path::new(&args.bytecode_file_path).with_extension(source_map_extension),
    )
    .map(remap_owned_loc_to_loc);

    let mut disassembler_options = DisassemblerOptions::new();
    disassembler_options.print_code = !args.skip_code;
    disassembler_options.only_externally_visible = args.skip_private;
    disassembler_options.print_basic_blocks = !args.skip_basic_blocks;
    disassembler_options.print_locals = !args.skip_locals;

    // TODO: make source mapping work with the Move source language
    let no_loc = Spanned::unsafe_no_loc(()).loc;
    let module: CompiledModule;
    let script: CompiledScript;
    let bytecode = if args.is_script {
        script = CompiledScript::deserialize(&bytecode_bytes)
            .expect("Script blob can't be deserialized");
        BinaryIndexedView::Script(&script)
    } else {
        module = CompiledModule::deserialize(&bytecode_bytes)
            .expect("Module blob can't be deserialized");
        BinaryIndexedView::Module(&module)
    };

    let mut source_mapping = {
        if let Ok(s) = source_map {
            SourceMapping::new(s, bytecode)
        } else {
            SourceMapping::new_from_view(bytecode, no_loc)
                .expect("Unable to build dummy source mapping")
        }
    };

    if let Some(source_code) = source {
        source_mapping.with_source_code((source_path.to_str().unwrap().to_string(), source_code));
    }

    let mut disassembler = Disassembler::new(source_mapping, disassembler_options);

    if let Some(file_path) = &args.code_coverage_path {
        disassembler
            .add_coverage_map(CoverageMap::from_binary_file(file_path).to_unified_exec_map());
    }

    let dissassemble_string = disassembler.disassemble().expect("Unable to dissassemble");

    println!("{}", dissassemble_string);
}
