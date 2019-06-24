// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for function definitions and handles.

use crate::loaded_data::loaded_module::LoadedModule;
use vm::{
    access::ModuleAccess,
    errors::*,
    file_format::{Bytecode, CodeUnit, FunctionDefinitionIndex},
    internals::ModuleIndex,
    CompiledModule,
};

/// Trait that defines the internal representation of a move function.
pub trait FunctionReference<'txn>: Sized + Clone {
    /// Create a new function reference to a module
    fn new(
        module: &'txn LoadedModule,
        idx: FunctionDefinitionIndex,
    ) -> Result<Self, VMInvariantViolation>;

    /// Fetch the reference to the module where the function is defined.
    fn module(&self) -> &'txn LoadedModule;

    /// Fetch the code of the function definition.
    fn code_definition(&self) -> &'txn [Bytecode];

    /// Return the signature vector for the function's local value
    fn local_count(&self) -> usize;

    /// Return function's argument type
    fn arg_count(&self) -> usize;

    /// Return function's return type.
    fn return_count(&self) -> usize;

    /// Return whether the function is native or not
    fn is_native(&self) -> bool;

    /// Return the name of the function
    fn name(&self) -> &'txn str;
}

/// Resolved form of a function handle
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionRef<'txn> {
    module: &'txn LoadedModule,
    def: &'txn FunctionDef,
    name: &'txn str,
}

impl<'txn> FunctionReference<'txn> for FunctionRef<'txn> {
    fn new(
        module: &'txn LoadedModule,
        idx: FunctionDefinitionIndex,
    ) -> Result<Self, VMInvariantViolation> {
        let def = &module.function_defs[idx.into_index()];
        let fn_definition = &module.function_def_at(idx);
        let name_idx = module.function_handle_at(fn_definition.function).name;
        Ok(FunctionRef {
            module,
            def,
            name: module.string_at(name_idx),
        })
    }

    fn module(&self) -> &'txn LoadedModule {
        &self.module
    }

    fn code_definition(&self) -> &'txn [Bytecode] {
        &self.def.code
    }

    fn local_count(&self) -> usize {
        self.def.local_count
    }

    fn arg_count(&self) -> usize {
        self.def.arg_count
    }

    fn return_count(&self) -> usize {
        self.def.local_count
    }

    fn is_native(&self) -> bool {
        (self.def.flags & CodeUnit::NATIVE) == CodeUnit::NATIVE
    }

    fn name(&self) -> &'txn str {
        self.name
    }
}

/// Resolved form of a function definition
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionDef {
    pub local_count: usize,
    pub arg_count: usize,
    pub return_count: usize,
    pub code: Vec<Bytecode>,
    pub flags: u8,
}

impl FunctionDef {
    pub fn new(module: &CompiledModule, idx: FunctionDefinitionIndex) -> Self {
        let definition = module.function_def_at(idx);
        let code = definition.code.code.clone();
        let handle = module.function_handle_at(definition.function);
        let function_sig = module.function_signature_at(handle.signature);
        let flags = definition.flags;

        FunctionDef {
            code,
            flags,
            arg_count: function_sig.arg_types.len(),
            return_count: function_sig.return_types.len(),
            // Local count for native function is omitted
            local_count: if (flags & CodeUnit::NATIVE) == CodeUnit::NATIVE {
                0
            } else {
                module.locals_signature_at(definition.code.locals).0.len()
            },
        }
    }
}
