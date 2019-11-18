// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_source_map::source_map::SourceMap;
use bytecode_to_boogie::translator::BoogieTranslator;
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::{compiler::compile_module, parser::ast::Loc, parser::parse_module};
use libra_tools::tempdir::TempPath;
use libra_types::account_address::AccountAddress;
use std::{env, fs, process::Command};
use stdlib::{stdlib_modules, stdlib_source_map};

// mod translator;
fn compile_files(file_names: Vec<&str>) -> (Vec<VerifiedModule>, SourceMap<Loc>) {
    let mut verified_modules = stdlib_modules().to_vec();
    let mut source_maps = stdlib_source_map().to_vec();
    let files_len = file_names.len();
    let dep_files = &file_names[0..files_len];

    // assuming the last file is a program that might contain a script
    let address = AccountAddress::default();
    for file_name in dep_files {
        let code = fs::read_to_string(file_name).unwrap();
        let module = parse_module(&code).unwrap();
        let (compiled_module, source_map) =
            compile_module(address, module, &verified_modules).expect("module failed to compile");
        let verified_module_res = VerifiedModule::new(compiled_module);

        match verified_module_res {
            Err(e) => {
                panic!("{:?}", e);
            }
            Ok(verified_module) => {
                verified_modules.push(verified_module);
                source_maps.push(source_map);
            }
        }
    }
    (verified_modules, source_maps)
}

fn generate_boogie(file_name: &str) -> String {
    let mut file_names = vec![];
    file_names.push(file_name);
    let (modules, source_maps) = compile_files(file_names.to_vec());

    let mut ts = BoogieTranslator::new(&modules, &source_maps);
    let mut res = String::new();

    // handwritten boogie code
    let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
    res.push_str(&written_code);
    res.push_str(&ts.translate());
    res
}

fn run_boogie(boogie_str: &str) {
    let temp_path = TempPath::new();
    temp_path.create_as_dir().unwrap();
    let boogie_file_path = temp_path.path().join("output.bpl");
    fs::write(&boogie_file_path, boogie_str).unwrap();
    if let Ok(boogie_path) = env::var("BOOGIE_EXE") {
        if let Ok(z3_path) = env::var("Z3_EXE") {
            let status = Command::new(boogie_path)
                .args(&[
                    &format!("{}{}", "-z3exe:", z3_path).as_str(),
                    "-doModSetAnalysis",
                    "-noinfer",
                    "-noVerify",
                    boogie_file_path.to_str().unwrap(),
                ])
                .status()
                .expect("failed to execute Boogie");
            assert!(status.success());
        }
    }
}

#[test]
fn test3() {
    run_boogie(&generate_boogie("test_mvir/test3.mvir"));
}

#[test]
fn test_arithmetic() {
    run_boogie(&generate_boogie("test_mvir/test-arithmetic.mvir"));
}

#[test]
fn test_control_flow() {
    run_boogie(&generate_boogie("test_mvir/test-control-flow.mvir"));
}

#[test]
fn test_func_call() {
    run_boogie(&generate_boogie("test_mvir/test-func-call.mvir"));
}

#[test]
fn test_reference() {
    run_boogie(&generate_boogie("test_mvir/test-reference.mvir"));
}

#[test]
fn test_struct() {
    run_boogie(&generate_boogie("test_mvir/test-struct.mvir"));
}
