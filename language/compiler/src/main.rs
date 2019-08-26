// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::{
    verifier::{verify_module_dependencies, VerifiedProgram},
    VerifiedModule,
};
use compiler::{util, Compiler};
use ir_to_bytecode::parser::{parse_module, parse_script};
use serde_json;
use std::{convert::TryFrom, fs, io::Write, path::PathBuf};
use stdlib::stdlib_modules;
use structopt::StructOpt;
use types::{
    access_path::AccessPath, account_address::AccountAddress, transaction::Program,
    vm_error::VMStatus,
};
use vm::file_format::CompiledModule;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "IR Compiler",
    author = "Libra",
    about = "Move IR to bytecode compiler."
)]
struct Args {
    /// Serialize and write the compiled output to this file
    #[structopt(short = "o", long = "output")]
    pub output_path: Option<String>,
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
    #[structopt(short = "-l", long = "list_dependencies")]
    pub list_dependencies: bool,
    /// Path to the list of modules that we want to link with
    #[structopt(long = "deps")]
    pub deps_path: Option<String>,
}

fn print_errors_and_exit(verification_errors: &[VMStatus]) -> ! {
    println!("Verification failed. Errors below:");
    for e in verification_errors {
        println!("{:?}", e);
    }
    std::process::exit(1);
}

fn do_verify_module(module: CompiledModule, dependencies: &[VerifiedModule]) -> VerifiedModule {
    let verified_module = match VerifiedModule::new(module) {
        Ok(module) => module,
        Err((_, errors)) => print_errors_and_exit(&errors),
    };
    let errors = verify_module_dependencies(&verified_module, dependencies);
    if !errors.is_empty() {
        print_errors_and_exit(&errors);
    }
    verified_module
}

fn write_output(path: &str, buf: &[u8]) {
    let mut f = fs::File::create(path)
        .unwrap_or_else(|err| panic!("Unable to open output file {}: {}", path, err));
    f.write_all(&buf)
        .unwrap_or_else(|err| panic!("Unable to write to output file {}: {}", path, err));
}

fn main() {
    let args = Args::from_args();

    let address = args
        .address
        .map(|a| AccountAddress::try_from(a).unwrap())
        .unwrap_or_else(AccountAddress::default);

    if args.list_dependencies {
        let source = fs::read_to_string(args.source_path).expect("Unable to read file");
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
        match args.output_path {
            Some(path) => {
                let deps_bytes =
                    serde_json::to_vec(&dependency_list).expect("Unable to serialize dependencies");
                write_output(&path, &deps_bytes);
            }
            None => println!(
                "{}",
                serde_json::to_string(&dependency_list).expect("Unable to serialize dependencies")
            ),
        }
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
        let source = fs::read_to_string(args.source_path).expect("Unable to read file");
        let compiler = Compiler {
            address,
            code: &source,
            skip_stdlib_deps: args.no_stdlib,
            extra_deps: deps,
            ..Compiler::default()
        };
        let (compiled_program, dependencies) = compiler
            .into_compiled_program_and_deps()
            .expect("Failed to compile program");

        let compiled_program = if !args.no_verify {
            let verified_program = VerifiedProgram::new(compiled_program, &dependencies)
                .expect("Failed to verify program");
            verified_program.into_inner()
        } else {
            compiled_program
        };

        match args.output_path {
            Some(path) => {
                let mut script = vec![];
                compiled_program
                    .script
                    .serialize(&mut script)
                    .expect("Unable to serialize script");
                let mut modules = vec![];
                for m in compiled_program.modules.iter() {
                    let mut buf = vec![];
                    m.serialize(&mut buf).expect("Unable to serialize module");
                    modules.push(buf);
                }
                let program = Program::new(script, modules, vec![]);
                let program_bytes =
                    serde_json::to_vec(&program).expect("Unable to serialize program");
                write_output(&path, &program_bytes);
            }
            None => {
                println!("{}", compiled_program);
            }
        }
    } else {
        let compiled_module = util::do_compile_module(&args.source_path, address, &deps);
        let compiled_module = if !args.no_verify {
            let verified_module = do_verify_module(compiled_module, &deps);
            verified_module.into_inner()
        } else {
            compiled_module
        };
        match args.output_path {
            Some(path) => {
                let mut out = vec![];
                compiled_module
                    .serialize(&mut out)
                    .expect("Unable to serialize module");
                write_output(&path, &out);
            }
            None => {
                println!("{}", compiled_module);
            }
        }
    }
}
