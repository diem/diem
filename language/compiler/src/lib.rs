// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod util;

#[cfg(test)]
mod unit_tests;

use bytecode_verifier::VerifiedModule;
use failure::prelude::*;
use ir_to_bytecode::{
    compiler::{compile_module, compile_program},
    parser::parse_program,
};
use std::mem;
use stdlib::stdlib_modules;
use types::{
    account_address::AccountAddress,
    transaction::{Program, TransactionArgument},
};
use vm::file_format::{CompiledModule, CompiledProgram, CompiledScript};

/// An API for the compiler. Supports setting custom options.
#[derive(Clone, Debug, Default)]
pub struct Compiler<'a> {
    /// The address used as the sender for the compiler.
    pub address: AccountAddress,
    /// The Move IR code to compile.
    pub code: &'a str,
    /// Skip stdlib dependencies if true.
    pub skip_stdlib_deps: bool,
    /// The address to use for stdlib.
    pub stdlib_address: AccountAddress,
    /// Extra dependencies to compile with.
    pub extra_deps: Vec<VerifiedModule>,

    // The typical way this should be used is with functional record update syntax:
    //
    // let compiler = Compiler { address, code, ..Compiler::new() };
    //
    // Until the #[non_exhaustive] attribute is available (see
    // https://github.com/rust-lang/rust/issues/44109), this workaround is required to make the
    // syntax be mandatory.
    #[allow(missing_docs)]
    #[doc(hidden)]
    pub _non_exhaustive: (),
}

impl<'a> Compiler<'a> {
    /// Compiles into a `CompiledProgram` where the bytecode hasn't been serialized.
    pub fn into_compiled_program(mut self) -> Result<CompiledProgram> {
        Ok(self.compile_impl()?.0)
    }

    /// Compiles into a `CompiledProgram` and also returns the dependencies.
    pub fn into_compiled_program_and_deps(
        mut self,
    ) -> Result<(CompiledProgram, Vec<VerifiedModule>)> {
        self.compile_impl()
    }

    /// Compiles into a `CompiledScript`.
    pub fn into_script(mut self) -> Result<CompiledScript> {
        let compiled_program = self.compile_impl()?.0;
        Ok(compiled_program.script)
    }

    /// Compiles the script into a serialized form.
    pub fn into_script_blob(mut self) -> Result<Vec<u8>> {
        let compiled_program = self.compile_impl()?.0;

        let mut serialized_script = Vec::<u8>::new();
        compiled_program.script.serialize(&mut serialized_script)?;
        Ok(serialized_script)
    }

    /// Compiles the module.
    pub fn into_compiled_module(mut self) -> Result<CompiledModule> {
        Ok(self.compile_mod()?.0)
    }

    /// Compiles the module into a serialized form.
    pub fn into_module_blob(mut self) -> Result<Vec<u8>> {
        let compiled_module = self.compile_mod()?.0;

        let mut serialized_module = Vec::<u8>::new();
        compiled_module.serialize(&mut serialized_module)?;
        Ok(serialized_module)
    }

    /// Compiles the code and arguments into a `Program` -- the bytecode is serialized.
    pub fn into_program(mut self, args: Vec<TransactionArgument>) -> Result<Program> {
        let compiled_program = self.compile_impl()?.0;

        let mut serialized_script = Vec::<u8>::new();
        compiled_program.script.serialize(&mut serialized_script)?;
        let mut serialized_modules = vec![];
        for m in compiled_program.modules {
            let mut module = vec![];
            m.serialize(&mut module).expect("module must serialize");
            serialized_modules.push(module);
        }
        Ok(Program::new(serialized_script, serialized_modules, args))
    }

    fn compile_impl(&mut self) -> Result<(CompiledProgram, Vec<VerifiedModule>)> {
        let parsed_program = parse_program(self.code)?;
        let deps = self.deps();
        let compiled_program = compile_program(self.address, parsed_program, &deps)?;
        Ok((compiled_program, deps))
    }

    fn compile_mod(&mut self) -> Result<(CompiledModule, Vec<VerifiedModule>)> {
        let parsed_program = parse_program(self.code)?;
        let deps = self.deps();
        let mut modules = parsed_program.modules;
        assert_eq!(modules.len(), 1, "Must have single module");
        let module = modules.pop().expect("Module must exist");
        let compiled_module = compile_module(self.address, module, &deps)?;
        Ok((compiled_module, deps))
    }

    fn deps(&mut self) -> Vec<VerifiedModule> {
        let extra_deps = mem::replace(&mut self.extra_deps, vec![]);
        if self.skip_stdlib_deps {
            extra_deps
        } else {
            let mut deps = stdlib_modules().to_vec();
            deps.extend(extra_deps);
            deps
        }
    }
}
