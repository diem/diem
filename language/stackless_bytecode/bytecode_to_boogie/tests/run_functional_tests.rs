// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

use functional_tests::{checker::check, errors::*, evaluator::eval, utils::parse_input};
use bytecode_to_boogie::translator::BoogieTranslator;
use bytecode_verifier::verifier::{
    verify_module_dependencies, verify_script_dependencies, VerifiedModule, VerifiedProgram,
};
use ir_to_bytecode::{
    compiler::{compile_module, compile_program},
    parser::{parse_module, parse_program},
};
use stdlib::stdlib_modules;
use std::{
    env,
    fs::{self, File},
    io::prelude::*,
};
use std::path::Path;
use std::process::Command;

fn is_ignore(path: &Path) -> bool {
    path.parent().unwrap().display().to_string().ends_with("generics")
}

// Runs all tests under the functional_tests/tests/testsuite directory.
#[datatest::files("../../functional_tests/tests/testsuite/",
{ input in r"(.*)\.mvir" if !is_ignore,
  path = r"${1}" })]
fn functional_tests(input: &str, path: &Path) -> Result<()> {
    let (config, directives, transactions) = parse_input(input)?;
    let mut deps = stdlib_modules().to_vec();
    for (i, transaction) in transactions.iter().enumerate() {
        // get the account data of the sender
        let data = config.accounts.get(&transaction.config.sender).unwrap();
        let addr = data.address();

        // stage 1: parse the program
        let parsed_program_res = parse_program(&transaction.input);
        if let Err(e) = parsed_program_res {
            continue;
        }
        let parsed_program = parsed_program_res.unwrap();

        // stage 2: compile the program
        let compiled_program_res = compile_program(addr, &parsed_program, &deps);
        if let Err(e) = compiled_program_res {
            continue;
        }
        let compiled_program = compiled_program_res.unwrap();

        // stage 3: verify the program
        let verified_program_res = VerifiedProgram::new(compiled_program, &deps);
        if let Err(e) = verified_program_res {
            // Only translate programs that pass the verifier
            continue;
        }

        let verified_program = verified_program_res.unwrap();

        let new_modules = verified_program.modules().to_vec();
        let mut modules = deps.to_vec();
        modules.extend(verified_program.modules().to_vec());
        modules.push(verified_program.script().clone().into_module());

        let mut ts = BoogieTranslator::new(&modules);
        let mut res = String::new();

        // handwritten boogie code
        let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
        res.push_str(&written_code);
        res.push_str(&ts.translate());

        // let input_path_str = input.display().to_string();
        let op = Path::new("output");
        let folder_name = path.parent().unwrap().file_name().unwrap().to_str().unwrap();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let output_path = op.join(&format!("{}__{}_{}.bpl", folder_name, file_name, i));
        // output.display().to_string();
        println!("{}", output_path.to_str().unwrap());
        let output_path_str = output_path.to_str().unwrap();
        let mut f = File::create(output_path_str).expect("Unable to create file");

        // write resulting code into output.bpl
        write!(f, "{}", res).expect("unable to write file");

        deps.extend(new_modules);
    }

    let res = eval(&config, &transactions)?;
    if let Err(e) = check(&res, &directives) {
        println!("{:#?}", res);
        return Err(e);
    }
    println!("test");
    Ok(())
}
