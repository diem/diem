// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::{
    verifier::{verify_module_dependencies, VerifiedProgram},
    VerifiedModule,
};
use compiler::{util, Compiler};
use serde_json;
use std::{fs, io::Write, path::PathBuf};
use structopt::StructOpt;
use types::{account_address::AccountAddress, transaction::Program};
use vm::{errors::VerificationError, file_format::CompiledModule};

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
    /// Do not automatically compile stdlib dependencies
    #[structopt(long = "no-stdlib")]
    pub no_stdlib: bool,
    /// Do not automatically run the bytecode verifier
    #[structopt(long = "no-verify")]
    pub no_verify: bool,
    /// Path to the Move IR source to compile
    #[structopt(parse(from_os_str))]
    pub source_path: PathBuf,
}

fn print_errors_and_exit(verification_errors: &[VerificationError]) -> ! {
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
    let (verified_module, errors) = verify_module_dependencies(verified_module, dependencies);
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

    let address = AccountAddress::default();

    if !args.module_input {
        let source = fs::read_to_string(args.source_path).expect("Unable to read file");
        let compiler = Compiler {
            code: &source,
            skip_stdlib_deps: args.no_stdlib,
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
        let dependencies = if args.no_stdlib {
            vec![]
        } else {
            util::build_stdlib(&AccountAddress::default())
        };
        let compiled_module = util::do_compile_module(&args.source_path, &address, &dependencies);
        let compiled_module = if !args.no_verify {
            let verified_module = do_verify_module(compiled_module, &dependencies);
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
