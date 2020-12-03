// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use compiled_stdlib::{stdlib_modules, StdLibOptions};
use diem_types::account_address::AccountAddress;
use functional_tests::{
    compiler::{Compiler, ScriptOrModule},
    testsuite,
};
use ir_to_bytecode::{
    compiler::{compile_module, compile_script},
    parser::parse_script_or_module,
};
use move_ir_types::ast;
use std::path::Path;
use vm::CompiledModule;

struct IRCompiler {
    deps: Vec<CompiledModule>,
}

impl IRCompiler {
    fn new(stdlib_modules: Vec<CompiledModule>) -> Self {
        IRCompiler {
            deps: stdlib_modules,
        }
    }
}

impl Compiler for IRCompiler {
    /// Compile a transaction script or module.
    fn compile<Logger: FnMut(String)>(
        &mut self,
        mut log: Logger,
        address: AccountAddress,
        input: &str,
    ) -> Result<ScriptOrModule> {
        Ok(match parse_script_or_module("unused_file_name", input)? {
            ast::ScriptOrModule::Script(parsed_script) => {
                log(format!("{}", &parsed_script));
                ScriptOrModule::Script(compile_script(Some(address), parsed_script, &self.deps)?.0)
            }
            ast::ScriptOrModule::Module(parsed_module) => {
                log(format!("{}", &parsed_module));
                let module = compile_module(address, parsed_module, &self.deps)?.0;
                self.deps.push(module.clone());
                ScriptOrModule::Module(module)
            }
        })
    }

    fn use_compiled_genesis(&self) -> bool {
        true
    }
}

fn run_test(path: &Path) -> datatest_stable::Result<()> {
    testsuite::functional_tests(
        IRCompiler::new(stdlib_modules(StdLibOptions::Compiled).to_vec()),
        path,
    )
}

datatest_stable::harness!(run_test, "tests", r".*\.mvir");
