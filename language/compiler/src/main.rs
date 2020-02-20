// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Context;
use bytecode_verifier::{
    verifier::{verify_module_dependencies, VerifiedProgram},
    VerifiedModule,
};
use compiler::{util, Compiler};
use ir_to_bytecode::parser::{parse_module, parse_script};
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    transaction::{Module, Script},
    vm_error::VMStatus,
};
use std::{
    convert::TryFrom,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use stdlib::stdlib_modules;
use structopt::StructOpt;
use vm::file_format::CompiledModule;

#[derive(Debug, StructOpt)]
#[structopt(name = "IR Compiler", about = "Move IR to bytecode compiler.")]
struct Args {
    /// Treat input file as a module (default is to treat file as a program)
    #[structopt(short = "m", long = "module")]
    pub module_input: bool,
    /// Account address used for publishing
    #[structopt(short = "a", long = "address")]
    pub address: Option<String>,
    /// Do not automatically compile stdlib dependencies
    #[structopt(long = "no-stdlib")]
    pub no_stdlib: bool,
    /// Do not automatically run the bytecode verifier
    #[structopt(long = "no-verify")]
    pub no_verify: bool,
    /// Path to the Move IR source to compile
    #[structopt(parse(from_os_str))]
    pub source_path: PathBuf,
    /// Instead of compiling the source, emit a dependency list of the compiled source
    #[structopt(short = "-l", long = "list-dependencies")]
    pub list_dependencies: bool,
    /// Path to the list of modules that we want to link with
    #[structopt(long = "deps")]
    pub deps_path: Option<String>,

    #[structopt(long = "src-map")]
    pub output_source_maps: bool,
}

fn print_errors_and_exit(verification_errors: &[VMStatus]) -> ! {
    println!("Verification failed. Errors below:");
    for e in verification_errors {
        println!("{:?}", e);
    }
    std::process::exit(1);
}

fn do_verify_module(module: CompiledModule, dependencies: &[VerifiedModule]) -> VerifiedModule {
    let verified_module =
        VerifiedModule::new(module).unwrap_or_else(|(_, errors)| print_errors_and_exit(&errors));
    let errors = verify_module_dependencies(&verified_module, dependencies);
    if !errors.is_empty() {
        print_errors_and_exit(&errors);
    }
    verified_module
}

fn write_output(path: &PathBuf, buf: &[u8]) {
    let mut f = fs::File::create(path)
        .with_context(|| format!("Unable to open output file {:?}", path))
        .unwrap();
    f.write_all(&buf)
        .with_context(|| format!("Unable to write to output file {:?}", path))
        .unwrap();
}

fn main() {
    let args = Args::from_args();

    let address = args
        .address
        .map(|a| AccountAddress::try_from(a).unwrap())
        .unwrap_or_else(AccountAddress::default);
    let source_path = Path::new(&args.source_path);
    let mvir_extension = "mvir";
    let mv_extension = "mv";
    let source_map_extension = "mvsm";
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

    if args.list_dependencies {
        let source = fs::read_to_string(args.source_path.clone()).expect("Unable to read file");
        let dependency_list: Vec<AccessPath> = if args.module_input {
            let module = parse_module(&source).expect("Unable to parse module");
            module.get_external_deps()
        } else {
            let script = parse_script(&source).expect("Unable to parse module");
            script.get_external_deps()
        }
        .into_iter()
        .map(|m| AccessPath::code_access_path(&m))
        .collect();
        println!(
            "{}",
            serde_json::to_string(&dependency_list).expect("Unable to serialize dependencies")
        );
        return;
    }

    let deps = {
        if let Some(path) = args.deps_path {
            let deps = fs::read_to_string(path).expect("Unable to read dependency file");
            let deps_list: Vec<Vec<u8>> =
                serde_json::from_str(deps.as_str()).expect("Unable to parse dependency file");
            deps_list
                .into_iter()
                .map(|module_bytes| {
                    VerifiedModule::new(
                        CompiledModule::deserialize(module_bytes.as_slice())
                            .expect("Downloaded module blob can't be deserialized"),
                    )
                    .expect("Downloaded module blob failed verifier")
                })
                .collect()
        } else if args.no_stdlib {
            vec![]
        } else {
            stdlib_modules().to_vec()
        }
    };

    if !args.module_input {
        let source = fs::read_to_string(args.source_path.clone()).expect("Unable to read file");
        let compiler = Compiler {
            address,
            skip_stdlib_deps: args.no_stdlib,
            extra_deps: deps,
            ..Compiler::default()
        };
        let (compiled_program, source_map, dependencies) = compiler
            .into_compiled_program_and_source_maps_deps(&source)
            .expect("Failed to compile program");

        let compiled_program = if !args.no_verify {
            let verified_program = VerifiedProgram::new(compiled_program, &dependencies)
                .expect("Failed to verify program");
            verified_program.into_inner()
        } else {
            compiled_program
        };

        if args.output_source_maps {
            let source_map_bytes = serde_json::to_vec(&source_map)
                .expect("Unable to serialize source maps for program");
            write_output(
                &source_path.with_extension(source_map_extension),
                &source_map_bytes,
            );
        }

        let mut script = vec![];
        compiled_program
            .script
            .serialize(&mut script)
            .expect("Unable to serialize script");
        let payload = Script::new(script, vec![]);
        let payload_bytes = serde_json::to_vec(&payload).expect("Unable to serialize program");
        write_output(&source_path.with_extension(mv_extension), &payload_bytes);
    } else {
        let (compiled_module, source_map) =
            util::do_compile_module(&args.source_path, address, &deps);
        let compiled_module = if !args.no_verify {
            let verified_module = do_verify_module(compiled_module, &deps);
            verified_module.into_inner()
        } else {
            compiled_module
        };

        if args.output_source_maps {
            let source_map_bytes = serde_json::to_vec(&source_map)
                .expect("Unable to serialize source maps for program");
            write_output(
                &source_path.with_extension(source_map_extension),
                &source_map_bytes,
            );
        }

        let mut module = vec![];
        compiled_module
            .serialize(&mut module)
            .expect("Unable to serialize module");
        let payload = Module::new(module);
        let payload_bytes = serde_json::to_vec(&payload).expect("Unable to serialize program");
        write_output(&source_path.with_extension(mv_extension), &payload_bytes);
    }
}
