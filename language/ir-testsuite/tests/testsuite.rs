// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use bytecode_verifier::verifier::VerifiedModule;
use functional_tests::{
    compiler::{Compiler, ScriptOrModule},
    testsuite,
};
use ir_to_bytecode::{
    compiler::{compile_module, compile_script},
    parser::parse_script_or_module,
};
use libra_types::account_address::AccountAddress;
use move_ir_types::ast;
use std::path::Path;
use stdlib::{env_stdlib_modules, stdlib_modules, use_staged, StdLibOptions};

struct IRCompiler {
    deps: Vec<VerifiedModule>,
}

impl IRCompiler {
    fn new(stdlib_modules: Vec<VerifiedModule>) -> Self {
        IRCompiler {
            deps: stdlib_modules,
        }
    }
}

impl Compiler for IRCompiler {
    /// Compile a transaction script or module.
    fn compile<Logger: FnMut(String) -> ()>(
        &mut self,
        mut log: Logger,
        address: AccountAddress,
        input: &str,
    ) -> Result<ScriptOrModule> {
        Ok(match parse_script_or_module("unused_file_name", input)? {
            ast::ScriptOrModule::Script(parsed_script) => {
                log(format!("{}", &parsed_script));
                ScriptOrModule::Script(compile_script(address, parsed_script, &self.deps)?.0)
            }
            ast::ScriptOrModule::Module(parsed_module) => {
                log(format!("{}", &parsed_module));
                let module = compile_module(address, parsed_module, &self.deps)?.0;
                let verified =
                    VerifiedModule::bypass_verifier_DANGEROUS_FOR_TESTING_ONLY(module.clone());
                self.deps.push(verified);
                ScriptOrModule::Module(module)
            }
        })
    }

    // Use the staged genesis/stdlib unless the the
    // MOVE_NO_USE_STAGED environment variable is set.
    fn stdlib() -> Option<Vec<VerifiedModule>> {
        if !use_staged() {
            Some(env_stdlib_modules().to_vec())
        } else {
            None
        }
    }
}

fn run_test(path: &Path) -> datatest_stable::Result<()> {
    // The IR tests always run with the staged stdlib
    let stdlib = stdlib_modules(StdLibOptions::Staged);
    let compiler = IRCompiler::new(stdlib.to_vec());
    testsuite::functional_tests(compiler, path)
}

datatest_stable::harness!(run_test, "tests", r".*\.mvir");
