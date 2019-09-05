// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

use bytecode_to_boogie::translator::BoogieTranslator;
use bytecode_verifier::verifier::VerifiedProgram;
use functional_tests::{checker::check, errors::*, evaluator::eval, utils::parse_input};
use ir_to_bytecode::{compiler::compile_program, parser::parse_program};
use std::{fs, path::Path};
use stdlib::stdlib_modules;

fn is_ignore(path: &Path) -> bool {
    path.parent()
        .unwrap()
        .display()
        .to_string()
        .ends_with("generics")
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
        if parsed_program_res.is_err() {
            continue;
        }
        let parsed_program = parsed_program_res.unwrap();

        // stage 2: compile the program
        let compiled_program_res = compile_program(addr, &parsed_program, &deps);
        if compiled_program_res.is_err() {
            continue;
        }
        let compiled_program = compiled_program_res.unwrap();

        // stage 3: verify the program
        let verified_program_res = VerifiedProgram::new(compiled_program, &deps);
        if verified_program_res.is_err() {
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

        let output_folder = Path::new("output");
        let folder_name = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let output_path = output_folder.join(&format!("{}__{}_{}.bpl", folder_name, file_name, i));
        println!("{}", output_path.to_str().unwrap());
        let _output_path_str = output_path.to_str().unwrap();

        // uncomment the code to write to Boogie files
        // let mut f = File::create(_output_path_str).expect("Unable to create file");
        // write!(f, "{}", res).expect("unable to write file");

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
