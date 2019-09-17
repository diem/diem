// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for function definitions and handles.

use crate::loaded_data::loaded_module::LoadedModule;
use bytecode_verifier::VerifiedModule;
use libra_types::identifier::IdentStr;
use vm::{
    access::ModuleAccess,
    file_format::{Bytecode, CodeUnit, FunctionDefinitionIndex, FunctionHandle, FunctionSignature},
    internals::ModuleIndex,
};

/// Trait that defines the internal representation of a move function.
pub trait FunctionReference<'txn>: Sized + Clone {
    /// Create a new function reference to a module
    fn new(module: &'txn LoadedModule, idx: FunctionDefinitionIndex) -> Self;

    /// Fetch the reference to the module where the function is defined
    fn module(&self) -> &'txn LoadedModule;

    /// Fetch the code of the function definition
    fn code_definition(&self) -> &'txn [Bytecode];

    /// Return the number of locals for the function
    fn local_count(&self) -> usize;

    /// Return the number of input parameters for the function
    fn arg_count(&self) -> usize;

    /// Return the number of output parameters for the function
    fn return_count(&self) -> usize;

    /// Return whether the function is native or not
    fn is_native(&self) -> bool;

    /// Return the name of the function
    fn name(&self) -> &'txn IdentStr;

    /// Returns the signature of the function
    fn signature(&self) -> &'txn FunctionSignature;
}

/// Resolved form of a function handle
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionRef<'txn> {
    module: &'txn LoadedModule,
    def: &'txn FunctionDef,
    handle: &'txn FunctionHandle,
}

impl<'txn> FunctionReference<'txn> for FunctionRef<'txn> {
    fn new(module: &'txn LoadedModule, idx: FunctionDefinitionIndex) -> Self {
        let def = &module.function_defs[idx.into_index()];
        let fn_definition = module.function_def_at(idx);
        let handle = module.function_handle_at(fn_definition.function);
        FunctionRef {
            module,
            def,
            handle,
        }
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
        self.def.return_count
    }

    fn is_native(&self) -> bool {
        (self.def.flags & CodeUnit::NATIVE) == CodeUnit::NATIVE
    }

    fn name(&self) -> &'txn IdentStr {
        self.module.identifier_at(self.handle.name)
    }

    fn signature(&self) -> &'txn FunctionSignature {
        self.module.function_signature_at(self.handle.signature)
    }
}

impl<'txn> FunctionRef<'txn> {
    pub fn pretty_string(&self) -> String {
        let signature = self.signature();
        format!(
            "{}::{}({:?}){:?}",
            self.module().name(),
            self.name().as_str(),
            signature.arg_types,
            signature.return_types
        )
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
    pub fn new(module: &VerifiedModule, idx: FunctionDefinitionIndex) -> Self {
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
