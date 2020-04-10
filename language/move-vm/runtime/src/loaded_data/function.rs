// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for function definitions and handles.

use crate::loaded_data::loaded_module::LoadedModule;
use bytecode_verifier::VerifiedModule;
use move_core_types::identifier::IdentStr;
use vm::{
    access::ModuleAccess,
    file_format::{
        Bytecode, CodeUnit, FunctionDefinitionIndex, FunctionHandle, Kind, Signature,
        SignatureIndex,
    },
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

    /// Return the locals for the function
    fn locals(&self) -> Option<&'txn Signature>;

    /// Return the number of locals for the function
    fn local_count(&self) -> usize {
        match self.locals() {
            Some(locals) => locals.len(),
            None => 0,
        }
    }

    /// Return the number of input parameters for the function
    fn arg_count(&self) -> usize;

    /// Return the number of output parameters for the function
    fn return_count(&self) -> usize;

    /// Return whether the function is native or not
    fn is_native(&self) -> bool;

    /// Return the name of the function
    fn name(&self) -> &'txn IdentStr;

    /// Returns the parameters of the function
    fn parameters(&self) -> &'txn Signature;

    /// Returns the "return parameters" of the function
    fn return_(&self) -> &'txn Signature;

    /// Returns the type parameters of the function
    fn type_parameters(&self) -> &'txn [Kind];
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
        assume!(self.handle.name.into_index() < self.module.identifiers().len()); // invariant
        self.module.identifier_at(self.handle.name)
    }

    fn parameters(&self) -> &'txn Signature {
        assume!(self.handle.parameters.into_index() < self.module.signatures().len()); // invariant
        self.module.signature_at(self.handle.parameters)
    }

    fn return_(&self) -> &'txn Signature {
        assume!(self.handle.return_.into_index() < self.module.signatures().len()); // invariant
        self.module.signature_at(self.handle.return_)
    }

    fn type_parameters(&self) -> &'txn [Kind] {
        &self.handle.type_parameters
    }

    fn locals(&self) -> Option<&'txn Signature> {
        self.def.locals_idx.map(|idx| self.module.signature_at(idx))
    }
}

impl<'txn> FunctionRef<'txn> {
    pub fn pretty_string(&self) -> String {
        let parameters = self.parameters();
        let return_ = self.return_();
        format!(
            "{}::{}({:?}){:?}",
            self.module().name(),
            self.name().as_str(),
            parameters,
            return_
        )
    }
}

/// Resolved form of a function definition
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionDef {
    pub locals_idx: Option<SignatureIndex>,
    pub arg_count: usize,
    pub return_count: usize,
    pub code: Vec<Bytecode>,
    pub flags: u8,
}

impl FunctionDef {
    pub fn new(module: &VerifiedModule, idx: FunctionDefinitionIndex) -> Self {
        precondition!(idx.into_index() < module.function_defs().len());
        let definition = module.function_def_at(idx);
        let code = definition.code.code.clone();
        let handle = module.function_handle_at(definition.function);
        let parameters = module.signature_at(handle.parameters);
        let return_ = module.signature_at(handle.return_);
        let flags = definition.flags;
        let locals_idx = if (flags & CodeUnit::NATIVE) == CodeUnit::NATIVE {
            None
        } else {
            Some(definition.code.locals)
        };

        FunctionDef {
            code,
            flags,
            arg_count: parameters.len(),
            return_count: return_.len(),
            locals_idx,
        }
    }
}
