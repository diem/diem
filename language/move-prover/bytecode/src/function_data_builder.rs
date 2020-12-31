// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides a builder for `FunctionData`, including building expressions and rewriting
//! bytecode.

use crate::{
    function_target::FunctionData,
    stackless_bytecode::{AttrId, Bytecode, TempIndex},
};
use move_model::{
    ast,
    ast::Exp,
    model::{FunctionEnv, GlobalEnv, Loc, NodeId, QualifiedId, StructId},
    ty::{Type, BOOL_TYPE},
};

/// A builder for `FunctionData`.
pub struct FunctionDataBuilder<'env> {
    pub fun_env: &'env FunctionEnv<'env>,
    pub data: FunctionData,
    next_free_attr_index: usize,
    current_loc: Loc,
}

impl<'env> FunctionDataBuilder<'env> {
    /// Creates a new builder.
    pub fn new(fun_env: &'env FunctionEnv<'env>, data: FunctionData) -> Self {
        let next_free_attr_index = data.next_free_attr_index();
        FunctionDataBuilder {
            fun_env,
            data,
            next_free_attr_index,
            current_loc: fun_env.get_loc(),
        }
    }

    /// Gets the global env associated with this builder.
    pub fn global_env(&self) -> &GlobalEnv {
        self.fun_env.module_env.env
    }

    /// Allocates a new temporary.
    pub fn new_temp(&mut self, ty: Type) -> TempIndex {
        let idx = self.data.local_types.len();
        self.data.local_types.push(ty);
        idx
    }

    /// Sets the default location.
    pub fn set_loc(&mut self, loc: Loc) {
        self.current_loc = loc;
    }

    /// Creates a new bytecode attribute id with default location.
    pub fn new_attr(&mut self) -> AttrId {
        let id = AttrId::new(self.next_free_attr_index);
        self.next_free_attr_index += 1;
        self.data.locations.insert(id, self.current_loc.clone());
        id
    }

    /// Makes a Call expression.
    pub fn mk_call(&self, ty: &Type, oper: ast::Operation, args: Vec<Exp>) -> Exp {
        let node_id = self
            .global_env()
            .new_node(self.current_loc.clone(), ty.clone());
        Exp::Call(node_id, oper, args)
    }

    /// Makes a Call expression with boolean result type.
    pub fn mk_bool_call(&self, oper: ast::Operation, args: Vec<Exp>) -> Exp {
        self.mk_call(&BOOL_TYPE, oper, args)
    }

    /// Makes a local for a temporary.
    pub fn mk_local(&self, temp: TempIndex) -> Exp {
        let ty = self.data.local_types[temp].clone();
        let node_id = self.global_env().new_node(self.current_loc.clone(), ty);
        let sym = self.fun_env.symbol_pool().make(&format!("$t{}", temp));
        Exp::LocalVar(node_id, sym)
    }

    /// Get's the memory associated with a Call(Global,..) or Call(Exists, ..) node. Crashes
    /// if the the node is not typed as expected.
    pub fn get_memory_of_node(&self, node_id: NodeId) -> QualifiedId<StructId> {
        let rty = &self.global_env().get_node_instantiation(node_id)[0];
        let (mid, sid, _) = rty.require_struct();
        mid.qualified(sid)
    }

    /// Emits a bytecode.
    pub fn emit(&mut self, bc: Bytecode) {
        self.data.code.push(bc);
    }

    /// Emits a bytecode via a function which takes a freshly generated attribute id.
    pub fn emit_with<F>(&mut self, f: F)
    where
        F: FnOnce(AttrId) -> Bytecode,
    {
        let attr_id = self.new_attr();
        self.data.code.push(f(attr_id))
    }
}
