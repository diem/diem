// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Context;
use bytecode_verifier::{dependencies, verify_module, verify_script};
use compiler::util;
use ir_to_bytecode::parser::{parse_module, parse_script};
use move_binary_format::{
    errors::VMError,
    file_format::{CompiledModule, CompiledScript},
};
use move_command_line_common::files::{
    MOVE_COMPILED_EXTENSION, MOVE_IR_EXTENSION, SOURCE_MAP_EXTENSION,
};
use move_core_types::account_address::AccountAddress;
use move_symbol_pool::Symbol;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "IR Compiler", about = "Move IR to bytecode compiler.")]
struct Args {
    /// Treat input file as a module (default is to treat file as a script)
    #[structopt(short = "m", long = "module")]
    pub module_input: bool,
    /// Account address used for publishing
    #[structopt(short = "a", long = "address")]
    pub address: String,
    /// Do not automatically run the bytecode verifier
    #[structopt(long = "no-verify")]
    pub no_verify: bool,
    /// Path to the Move IR source to compile
    #[structopt(parse(from_os_str))]
    pub source_path: PathBuf,
    /// Instead of compiling the source, emit a dependency list of the compiled source
    #[structopt(short = "l", long = "list-dependencies")]
    pub list_dependencies: bool,
    /// Path to the list of modules that we want to link with
    #[structopt(short = "d", long = "deps")]
    pub deps_path: Option<String>,

    #[structopt(long = "src-map")]
    pub output_source_maps: bool,
}

fn print_error_and_exit(verification_error: &VMError) -> ! {
    println!("Verification failed:");
    println!("{:?}", verification_error);
    std::process::exit(1);
}

fn do_verify_module(module: &CompiledModule, dependencies: &[CompiledModule]) {
    verify_module(module).unwrap_or_else(|err| print_error_and_exit(&err));
    if let Err(err) = dependencies::verify_module(module, dependencies) {
        print_error_and_exit(&err);
    }
}

fn do_verify_script(script: &CompiledScript, dependencies: &[CompiledModule]) {
    verify_script(script).unwrap_or_else(|err| print_error_and_exit(&err));
    if let Err(err) = dependencies::verify_script(script, dependencies) {
        print_error_and_exit(&err);
    }
}

fn write_output(path: &Path, buf: &[u8]) {
    let mut f = fs::File::create(path)
        .with_context(|| format!("Unable to open output file {:?}", path))
        .unwrap();
    f.write_all(buf)
        .with_context(|| format!("Unable to write to output file {:?}", path))
        .unwrap();
}

fn main() {
    let args = Args::from_args();

    let address = match AccountAddress::from_hex_literal(&args.address) {
        Ok(address) => address,
        Err(_) => {
            println!("Bad address: {}", args.address);
            std::process::exit(1);
        }
    };
    let source_path = Path::new(&args.source_path);
    let mvir_extension = MOVE_IR_EXTENSION;
    let mv_extension = MOVE_COMPILED_EXTENSION;
    let source_map_extension = SOURCE_MAP_EXTENSION;
    let extension = source_path
        .extension()
        .expect("Missing file extension for input source file");
    if extension != mvir_extension {
        println!(
            "Bad source file extension {:?}; expected {}",
            extension, mvir_extension
        );
        std::process::exit(1);
    }

    let file_name = Symbol::from(args.source_path.as_path().as_os_str().to_str().unwrap());

    if args.list_dependencies {
        let source = fs::read_to_string(args.source_path.clone()).expect("Unable to read file");
        let dependency_list = if args.module_input {
            let module = parse_module(file_name, &source).expect("Unable to parse module");
            module.get_external_deps()
        } else {
            let script = parse_script(file_name, &source).expect("Unable to parse module");
            script.get_external_deps()
        };
        println!(
            "{}",
            serde_json::to_string(&dependency_list).expect("Unable to serialize dependencies")
        );
        return;
    }

    let deps_owned = {
        if let Some(path) = args.deps_path {
            let deps = fs::read_to_string(path).expect("Unable to read dependency file");
            let deps_list: Vec<Vec<u8>> =
                serde_json::from_str(deps.as_str()).expect("Unable to parse dependency file");
            deps_list
                .into_iter()
                .map(|module_bytes| {
                    let module = CompiledModule::deserialize(module_bytes.as_slice())
                        .expect("Downloaded module blob can't be deserialized");
                    verify_module(&module).expect("Downloaded module blob failed verifier");
                    module
                })
                .collect()
        } else {
            vec![]
        }
    };

    if args.module_input {
        let (compiled_module, source_map) =
            util::do_compile_module(&args.source_path, address, &deps_owned);
        if !args.no_verify {
            do_verify_module(&compiled_module, &deps_owned);
        }

        if args.output_source_maps {
            let source_map_bytes =
                bcs::to_bytes(&source_map).expect("Unable to serialize source maps for module");
            write_output(
                &source_path.with_extension(source_map_extension),
                &source_map_bytes,
            );
        }

        let mut module = vec![];
        compiled_module
            .serialize(&mut module)
            .expect("Unable to serialize module");
        write_output(&source_path.with_extension(mv_extension), &module);
    } else {
        let (compiled_script, source_map) =
            util::do_compile_script(&args.source_path, address, &deps_owned);
        if !args.no_verify {
            do_verify_script(&compiled_script, &deps_owned);
        }

        if args.output_source_maps {
            let source_map_bytes =
                bcs::to_bytes(&source_map).expect("Unable to serialize source maps for script");
            write_output(
                &source_path.with_extension(source_map_extension),
                &source_map_bytes,
            );
        }

        let mut script = vec![];
        compiled_script
            .serialize(&mut script)
            .expect("Unable to serialize script");
        write_output(&source_path.with_extension(mv_extension), &script);
    }
}
