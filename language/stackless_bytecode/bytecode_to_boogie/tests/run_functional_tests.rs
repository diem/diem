// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![feature(custom_test_frameworks)]
#![test_runner(datatest::runner)]

use functional_tests::{checker::check, errors::*, evaluator::eval, utils::parse_input};

// Runs all tests under the functional_tests/tests/testsuite directory.
#[datatest::files("../../functional_tests/tests/testsuite/dereference_tests", { input in r".*\.mvir" })]
fn functional_tests(input: &str) -> Result<()> {
    let (config, directives, transactions) = parse_input(input)?;
    let mut deps = stdlib_modules().to_vec();
    for transaction in transactions {
        // get the account data of the sender
        let data = config.accounts.get(&transaction.config.sender).unwrap();
        let addr = data.address();

        // stage 1: parse the program
        let parsed_program_res = parse_program(&transaction.program);
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
            continue;
        }
        let verified_program = verified_program_res.unwrap();

        let new_modules = verified_program.modules().to_vec();
        let modules = deps.to_vec();
        modules.extend(verified_program.modules().to_vec());
        modules.push(verified_program.script().clone().into_module());

        let mut ts = BoogieTranslator::new(&modules);
        let mut res = String::new();

        // handwritten boogie code
        let written_code = fs::read_to_string("src/bytecode_instrs.bpl").unwrap();
        res.push_str(&written_code);
        res.push_str(&ts.translate());
        let mut f = File::create("output.bpl").expect("Unable to create file");

        // write resulting code into output.bpl
        write!(f, "{}", res).expect("unable to write file");

        // This has to be before deps.extend since verified_program holds a reference to the
        // deps.
        // let compiled_program = verified_program.into_inner();
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
