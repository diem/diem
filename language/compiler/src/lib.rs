// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod util;

#[cfg(test)]
mod unit_tests;

use anyhow::Result;
use bytecode_source_map::source_map::{ModuleSourceMap, SourceMap};
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::{
    compiler::{compile_module, compile_program},
    parser::{ast::Loc, parse_program},
};
use libra_types::{
    account_address::AccountAddress,
    transaction::{Script, TransactionArgument},
};
use std::mem;
use stdlib::stdlib_modules;
use vm::file_format::{CompiledModule, CompiledProgram, CompiledScript};

/// An API for the compiler. Supports setting custom options.
#[derive(Clone, Debug, Default)]
pub struct Compiler {
    /// The address used as the sender for the compiler.
    pub address: AccountAddress,
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

impl Compiler {
    /// Compiles into a `CompiledProgram` where the bytecode hasn't been serialized.
    pub fn into_compiled_program(mut self, code: &str) -> Result<CompiledProgram> {
        Ok(self.compile_impl(code)?.0)
    }

    pub fn into_compiled_program_and_source_maps(
        mut self,
        code: &str,
    ) -> Result<(CompiledProgram, SourceMap<Loc>)> {
        let (compiled_program, source_maps, _) = self.compile_impl(code)?;
        Ok((compiled_program, source_maps))
    }

    pub fn into_compiled_program_and_source_maps_deps(
        mut self,
        code: &str,
    ) -> Result<(CompiledProgram, SourceMap<Loc>, Vec<VerifiedModule>)> {
        Ok(self.compile_impl(code)?)
    }

    /// Compiles into a `CompiledProgram` and also returns the dependencies.
    pub fn into_compiled_program_and_deps(
        mut self,
        code: &str,
    ) -> Result<(CompiledProgram, Vec<VerifiedModule>)> {
        let (compiled_program, _, deps) = self.compile_impl(code)?;
        Ok((compiled_program, deps))
    }

    /// Compiles into a `CompiledScript`.
    pub fn into_script(mut self, code: &str) -> Result<CompiledScript> {
        let compiled_program = self.compile_impl(code)?.0;
        Ok(compiled_program.script)
    }

    /// Compiles the script into a serialized form.
    pub fn into_script_blob(mut self, code: &str) -> Result<Vec<u8>> {
        let compiled_program = self.compile_impl(code)?.0;

        let mut serialized_script = Vec::<u8>::new();
        compiled_program.script.serialize(&mut serialized_script)?;
        Ok(serialized_script)
    }

    /// Compiles the module.
    pub fn into_compiled_module(mut self, code: &str) -> Result<CompiledModule> {
        Ok(self.compile_mod(code)?.0)
    }

    /// Compiles the module into a serialized form.
    pub fn into_module_blob(mut self, code: &str) -> Result<Vec<u8>> {
        let compiled_module = self.compile_mod(code)?.0;

        let mut serialized_module = Vec::<u8>::new();
        compiled_module.serialize(&mut serialized_module)?;
        Ok(serialized_module)
    }

    /// Compiles the code and arguments into a `Script` -- the bytecode is serialized.
    pub fn into_program(self, code: &str, args: Vec<TransactionArgument>) -> Result<Script> {
        Ok(Script::new(self.into_script_blob(code)?, args))
    }

    fn compile_impl(
        &mut self,
        code: &str,
    ) -> Result<(CompiledProgram, SourceMap<Loc>, Vec<VerifiedModule>)> {
        let parsed_program = parse_program(code)?;
        let deps = self.deps();
        let (compiled_program, source_maps) = compile_program(self.address, parsed_program, &deps)?;
        Ok((compiled_program, source_maps, deps))
    }

    fn compile_mod(
        &mut self,
        code: &str,
    ) -> Result<(CompiledModule, ModuleSourceMap<Loc>, Vec<VerifiedModule>)> {
        let parsed_program = parse_program(code)?;
        let deps = self.deps();
        let mut modules = parsed_program.modules;
        assert_eq!(modules.len(), 1, "Must have single module");
        let module = modules.pop().expect("Module must exist");
        let (compiled_module, source_map) = compile_module(self.address, module, &deps)?;
        Ok((compiled_module, source_map, deps))
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
