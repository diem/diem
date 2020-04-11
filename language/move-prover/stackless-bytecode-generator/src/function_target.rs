// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    annotations::Annotations,
    stackless_bytecode::{AttrId, Bytecode},
};
use spec_lang::{
    ast::Condition,
    env::{FunId, FunctionEnv, Loc, TypeParameter},
    symbol::{Symbol, SymbolPool},
    ty::Type,
};
use std::collections::BTreeMap;
use vm::file_format::CodeOffset;

/// A FunctionTarget is a drop-in replacement for a FunctionEnv which allows to rewrite
/// and analyze bytecode and parameter/local types. It encapsulates a FunctionEnv and information
/// which can be rewritten using the `FunctionTargetsHolder` data structure.
#[derive(Debug)]
pub struct FunctionTarget<'env> {
    pub func_env: &'env FunctionEnv<'env>,
    pub data: &'env FunctionTargetData,
}

/// Holds the owned data belonging to a FunctionTarget, which can be rewritten using
/// the `FunctionTargetsHolder::rewrite` method.
#[derive(Debug)]
pub struct FunctionTargetData {
    pub code: Vec<Bytecode>,
    pub local_types: Vec<Type>,
    pub return_types: Vec<Type>,
    pub locations: BTreeMap<AttrId, Loc>,
    pub annotations: Annotations,
}

impl<'env> FunctionTarget<'env> {
    /// Returns the name of this function.
    pub fn get_name(&self) -> Symbol {
        self.func_env.get_name()
    }

    /// Gets the id of this function.
    pub fn get_id(&self) -> FunId {
        self.func_env.get_id()
    }

    /// Shortcut for accessing the symbol pool.
    pub fn symbol_pool(&self) -> &SymbolPool {
        self.func_env.module_env.symbol_pool()
    }

    /// Returns the location of this function.
    pub fn get_loc(&self) -> Loc {
        self.func_env.get_loc()
    }

    /// Returns the location of the bytecode at the given offset.
    pub fn get_bytecode_loc(&self, attr_id: AttrId) -> Loc {
        if let Some(loc) = self.data.locations.get(&attr_id) {
            loc.clone()
        } else {
            self.get_loc()
        }
    }

    /// Returns true if this function is native.
    pub fn is_native(&self) -> bool {
        self.func_env.is_native()
    }

    /// Returns true if this function is public.
    pub fn is_public(&self) -> bool {
        self.func_env.is_public()
    }

    /// Returns true if this function mutates any references (i.e. has &mut parameters).
    pub fn is_mutating(&self) -> bool {
        self.func_env.is_mutating()
    }

    /// Returns the type parameters associated with this function.
    pub fn get_type_parameters(&self) -> Vec<TypeParameter> {
        self.func_env.get_type_parameters()
    }

    /// Returns return type at given index.
    pub fn get_return_type(&self, idx: usize) -> &Type {
        &self.data.return_types[idx]
    }

    /// Returns return types of this function.
    pub fn get_return_types(&self) -> &[Type] {
        &self.data.return_types
    }

    /// Returns the number of return values of this function.
    pub fn get_return_count(&self) -> usize {
        self.data.return_types.len()
    }

    pub fn get_parameter_count(&self) -> usize {
        self.func_env.get_parameter_count()
    }

    /// Get the name to be used for a local. If the local is an argument, use that for naming,
    /// otherwise generate a unique name.
    pub fn get_local_name(&self, idx: usize) -> Symbol {
        self.func_env.get_local_name(idx)
    }

    /// Gets the number of locals of this function, including parameters.
    pub fn get_local_count(&self) -> usize {
        self.data.local_types.len()
    }

    /// Gets the number of user declared locals of this function, excluding locals which have
    /// been introduced by transformations.
    pub fn get_user_local_count(&self) -> usize {
        self.func_env.get_local_count()
    }

    /// Gets the type of the local at index. This must use an index in the range as determined by
    /// `get_local_count`.
    pub fn get_local_type(&self, idx: usize) -> &Type {
        &self.data.local_types[idx]
    }

    /// Returns specification conditions associated with this function.
    pub fn get_specification_on_decl(&'env self) -> &'env [Condition] {
        self.func_env.get_specification_on_decl()
    }

    /// Returns specification conditions associated with this function at bytecode offset.
    pub fn get_specification_on_impl(&'env self, offset: CodeOffset) -> Option<&'env [Condition]> {
        self.func_env.get_specification_on_impl(offset)
    }

    /// Gets the bytecode.
    pub fn get_code(&self) -> &[Bytecode] {
        &self.data.code
    }

    /// Gets annotations.
    pub fn get_annotations(&self) -> &Annotations {
        &self.data.annotations
    }
}
