// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytecode_source_map::disassembler::{Disassembler, DisassemblerOptions};
use bytecode_source_map::mapping::SourceMapping;
use bytecode_source_map::source_map::ModuleSourceMap;
use bytecode_source_map::utils::module_source_map_from_file;
use ir_to_bytecode_syntax::ast::Loc;
use libra_types::transaction::Module;
use serde_json;
use std::fs;
use std::path::Path;
use structopt::StructOpt;
use vm::file_format::{CompiledModule, CompiledScript};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Move Bytecode Disassembler",
    about = "Move IR bytecode disassembler."
)]
struct Args {
    /// Shows only public functions.
    #[structopt(short = "p", long = "public")]
    pub only_public: bool,

    /// Prints out the disassembled bytecodes for each function.
    #[structopt(short = "c", long = "code")]
    pub print_code: bool,

    /// The path to the bytecode file to disassemble; let's call it file.mv. We assume that two
    /// other files reside under the same directory: a source map file.mvsm (possibly) and the Move
    /// IR source code file.mvir.
    #[structopt(short = "s", long = "script")]
    pub is_script: bool,

    /// The path to the bytecode file.
    #[structopt(short = "b", long = "bytecode")]
    pub bytecode_file_path: String,

    /// Print basic blocks.
    #[structopt(long = "basic-blocks")]
    pub print_basic_blocks: bool,

    /// Print locals within each function.
    #[structopt(long = "locals")]
    pub print_locals: bool,
}

fn main() {
    let args = Args::from_args();

    let mvir_extension = "mvir";
    let mv_bytecode_extension = "mv";
    let source_map_extension = "mvsm";

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

    let bytecode_source =
        fs::read_to_string(args.bytecode_file_path.clone()).expect("Unable to read bytecode file");
    let module_bytes: Module = serde_json::from_str(bytecode_source.as_str())
        .expect("Unable to deserialize bytecode file");

    let ir_source_path = Path::new(&args.bytecode_file_path).with_extension(mvir_extension);
    let ir_source = fs::read_to_string(&ir_source_path).ok();
    let source_map = module_source_map_from_file::<Loc>(
        &Path::new(&args.bytecode_file_path).with_extension(source_map_extension),
    );

    let mut disassembler_options = DisassemblerOptions::new();
    disassembler_options.print_code = args.print_code | args.print_basic_blocks;
    disassembler_options.only_public = args.only_public;
    disassembler_options.print_basic_blocks = args.print_basic_blocks;
    disassembler_options.print_locals = args.print_locals;

    let mut source_mapping = if args.is_script {
        let compiled_script = CompiledScript::deserialize(module_bytes.code())
            .expect("Script blob can't be deserialized");
        source_map
            .or_else(|_| ModuleSourceMap::dummy_from_script(&compiled_script))
            .and_then(|source_map| Ok(SourceMapping::new_from_script(source_map, compiled_script)))
            .expect("Unable to build source mapping for compiled script")
    } else {
        let compiled_module = CompiledModule::deserialize(module_bytes.code())
            .expect("Module blob can't be deserialized");
        source_map
            .or_else(|_| ModuleSourceMap::dummy_from_module(&compiled_module))
            .and_then(|source_map| Ok(SourceMapping::new(source_map, compiled_module)))
            .expect("Unable to build source mapping for compiled module")
    };

    if let Some(source_code) = ir_source {
        source_mapping
            .with_source_code((ir_source_path.to_str().unwrap().to_string(), source_code));
    }

    let disassembler = Disassembler::new(source_mapping, disassembler_options);

    let dissassemble_string = disassembler.disassemble().expect("Unable to dissassemble");

    println!("{}", dissassemble_string);
}
