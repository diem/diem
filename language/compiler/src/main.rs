// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::verifier::{
    verify_module, verify_module_dependencies, verify_script, verify_script_dependencies,
};
use compiler::{
    compiler::compile_program,
    parser::parse_program,
    util::{build_stdlib, do_compile_module},
};
use std::{fs, io::Write, path::PathBuf};
use structopt::StructOpt;
use types::account_address::AccountAddress;
use vm::{
    errors::VerificationError,
    file_format::{CompiledModule, CompiledScript},
};

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

fn check_verification_results(verification_errors: &[VerificationError]) {
    if !verification_errors.is_empty() {
        println!("Verification failed. Errors below:");
        for e in verification_errors {
            println!("{:?}", e);
        }
        std::process::exit(1);
    }
}

fn do_verify_module(module: &CompiledModule, dependencies: &[CompiledModule]) {
    let (verified_module, verification_errors) = verify_module(module.clone());
    check_verification_results(&verification_errors);
    let (_verified_module, verification_errors) =
        verify_module_dependencies(verified_module, dependencies);
    check_verification_results(&verification_errors);
}

fn do_verify_script(script: &CompiledScript, dependencies: &[CompiledModule]) {
    let (verified_script, verification_errors) = verify_script(script.clone());
    check_verification_results(&verification_errors);
    let (_verified_script, verification_errors) =
        verify_script_dependencies(verified_script, dependencies);
    check_verification_results(&verification_errors);
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
    let mut dependencies = if args.no_stdlib {
        vec![]
    } else {
        build_stdlib()
    };

    if !args.module_input {
        let source = fs::read_to_string(args.source_path).expect("Unable to read file");
        let parsed_program = parse_program(&source).unwrap();

        let compiled_program = compile_program(&address, &parsed_program, &dependencies).unwrap();

        // TODO: Make this a do_verify_program helper function.
        if !args.no_verify {
            for m in &compiled_program.modules {
                do_verify_module(m, &dependencies);
                dependencies.push(m.clone());
            }
            do_verify_script(&compiled_program.script, &dependencies);
        }

        match args.output_path {
            Some(path) => {
                // TODO: Only the script is serialized. Shall we also serialize the modules?
                let mut out = vec![];
                compiled_program
                    .script
                    .serialize(&mut out)
                    .expect("Unable to serialize script");
                write_output(&path, &out);
            }
            None => {
                println!("{}", compiled_program);
            }
        }
    } else {
        let compiled_module = do_compile_module(&args.source_path, &address, &dependencies);
        if !args.no_verify {
            do_verify_module(&compiled_module, &dependencies);
        }
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
