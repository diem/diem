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
use move_binary_format::file_format::{CompiledModule, CompiledScript};
use move_core_types::account_address::AccountAddress;
use move_ir_types::location::Loc;

/// An API for the compiler. Supports setting custom options.
#[derive(Clone, Debug)]
pub struct Compiler<'a> {
    /// The address used as the sender for the compiler.
    pub address: AccountAddress,
    /// Extra dependencies to compile with.
    pub deps: Vec<&'a CompiledModule>,
}

impl<'a> Compiler<'a> {
    pub fn new(address: AccountAddress, deps: Vec<&'a CompiledModule>) -> Self {
        Self { address, deps }
    }

    /// Compiles into a `CompiledScript` where the bytecode hasn't been serialized.
    pub fn into_compiled_script_and_source_map(
        self,
        file_name: &str,
        code: &str,
    ) -> Result<(CompiledScript, SourceMap<Loc>)> {
        let (compiled_script, source_map) = self.compile_script(file_name, code)?;
        Ok((compiled_script, source_map))
    }

    /// Compiles the script into a serialized form.
    pub fn into_script_blob(self, file_name: &str, code: &str) -> Result<Vec<u8>> {
        let compiled_script = self.compile_script(file_name, code)?.0;

        let mut serialized_script = Vec::<u8>::new();
        compiled_script.serialize(&mut serialized_script)?;
        Ok(serialized_script)
    }

    /// Compiles the module.
    pub fn into_compiled_module(self, file_name: &str, code: &str) -> Result<CompiledModule> {
        Ok(self.compile_mod(file_name, code)?.0)
    }

    /// Compiles the module into a serialized form.
    pub fn into_module_blob(self, file_name: &str, code: &str) -> Result<Vec<u8>> {
        let compiled_module = self.compile_mod(file_name, code)?.0;

        let mut serialized_module = Vec::<u8>::new();
        compiled_module.serialize(&mut serialized_module)?;
        Ok(serialized_module)
    }

    fn compile_script(
        self,
        file_name: &str,
        code: &str,
    ) -> Result<(CompiledScript, SourceMap<Loc>)> {
        let parsed_script = parse_script(file_name, code)?;
        let (compiled_script, source_map) =
            compile_script(None, parsed_script, self.deps.iter().map(|d| &**d))?;
        Ok((compiled_script, source_map))
    }

    fn compile_mod(self, file_name: &str, code: &str) -> Result<(CompiledModule, SourceMap<Loc>)> {
        let parsed_module = parse_module(file_name, code)?;
        let (compiled_module, source_map) =
            compile_module(self.address, parsed_module, self.deps.iter().map(|d| &**d))?;
        Ok((compiled_module, source_map))
    }
}
