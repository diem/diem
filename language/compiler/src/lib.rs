// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod util;

#[cfg(test)]
mod unit_tests;

use anyhow::Result;
use bytecode_source_map::source_map::SourceMap;
use ir_to_bytecode::{
    compiler::{compile_module, compile_script},
    parser::{parse_module, parse_script},
};
use move_core_types::account_address::AccountAddress;
use move_ir_types::location::Loc;
use std::mem;
use vm::file_format::{CompiledModule, CompiledScript};

/// An API for the compiler. Supports setting custom options.
#[derive(Clone, Debug)]
pub struct Compiler {
    /// The address used as the sender for the compiler.
    pub address: AccountAddress,
    /// Skip stdlib dependencies if true.
    pub skip_stdlib_deps: bool,
    /// Extra dependencies to compile with.
    pub extra_deps: Vec<CompiledModule>,
}

impl Compiler {
    pub fn new(
        address: AccountAddress,
        skip_stdlib_deps: bool,
        extra_deps: Vec<CompiledModule>,
    ) -> Self {
        Self {
            address,
            skip_stdlib_deps,
            extra_deps,
        }
    }

    /// Compiles into a `CompiledScript` where the bytecode hasn't been serialized.
    pub fn into_compiled_script_and_source_map(
        mut self,
        file_name: &str,
        code: &str,
    ) -> Result<(CompiledScript, SourceMap<Loc>)> {
        let (compiled_script, source_map, _) = self.compile_script(file_name, code)?;
        Ok((compiled_script, source_map))
    }

    /// Compiles the script into a serialized form.
    pub fn into_script_blob(mut self, file_name: &str, code: &str) -> Result<Vec<u8>> {
        let compiled_script = self.compile_script(file_name, code)?.0;

        let mut serialized_script = Vec::<u8>::new();
        compiled_script.serialize(&mut serialized_script)?;
        Ok(serialized_script)
    }

    /// Compiles the module.
    pub fn into_compiled_module(mut self, file_name: &str, code: &str) -> Result<CompiledModule> {
        Ok(self.compile_mod(file_name, code)?.0)
    }

    /// Compiles the module into a serialized form.
    pub fn into_module_blob(mut self, file_name: &str, code: &str) -> Result<Vec<u8>> {
        let compiled_module = self.compile_mod(file_name, code)?.0;

        let mut serialized_module = Vec::<u8>::new();
        compiled_module.serialize(&mut serialized_module)?;
        Ok(serialized_module)
    }

    fn compile_script(
        &mut self,
        file_name: &str,
        code: &str,
    ) -> Result<(CompiledScript, SourceMap<Loc>, Vec<CompiledModule>)> {
        let parsed_script = parse_script(file_name, code)?;
        let deps = self.deps();
        let (compiled_script, source_map) = compile_script(None, parsed_script, &deps)?;
        Ok((compiled_script, source_map, deps))
    }

    fn compile_mod(
        &mut self,
        file_name: &str,
        code: &str,
    ) -> Result<(CompiledModule, SourceMap<Loc>, Vec<CompiledModule>)> {
        let parsed_module = parse_module(file_name, code)?;
        let deps = self.deps();
        let (compiled_module, source_map) = compile_module(self.address, parsed_module, &deps)?;
        Ok((compiled_module, source_map, deps))
    }

    fn deps(&mut self) -> Vec<CompiledModule> {
        let extra_deps = mem::replace(&mut self.extra_deps, vec![]);
        if self.skip_stdlib_deps {
            extra_deps
        } else {
            let mut deps: Vec<CompiledModule> = diem_framework_releases::current_modules().to_vec();
            deps.extend(extra_deps);
            deps
        }
    }
}
