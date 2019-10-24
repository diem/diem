// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_source_map::source_map::SourceMap;
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::{compiler::compile_module, parser::ast::Loc, parser::parse_module};
use libra_types::account_address::AccountAddress;
use std::{
    env,
    fs::{self, File},
    io::prelude::*,
};
use stdlib::{stdlib_modules, stdlib_source_map};
use tree_heap::translator::BoogieTranslator;

// mod translator;
fn compile_files(file_names: Vec<String>) -> (Vec<VerifiedModule>, SourceMap<Loc>) {
    // only include vector module for ease of testing vector implementation
    let vector_module = stdlib_modules().to_vec()[6].clone();
    let mut verified_modules = vec![vector_module];
    // let mut verified_modules = stdlib_modules().to_vec();
    let mut source_maps = stdlib_source_map().to_vec();
    let files_len = file_names.len();
    let dep_files = &file_names[0..files_len];
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_names = &args[1..];
    // read files and compile into compiled modules
    let (modules, source_maps) = compile_files(file_names.to_vec());
    let mut ts = BoogieTranslator::new(&modules, &source_maps);
    let mut res = String::new();

    // handwritten boogie code
    let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
    res.push_str(&written_code);
    res.push_str(&ts.translate());
    let mut f = File::create("output.bpl").expect("Unable to create file");

    // write resulting code into output.bpl
    write!(f, "{}", res).expect("unable to write file");
}
