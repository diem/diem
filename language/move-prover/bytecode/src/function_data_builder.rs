// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides a builder for `FunctionData`, including building expressions and rewriting
//! bytecode.

use crate::{
    function_target::FunctionData,
    stackless_bytecode::{AttrId, Bytecode, Label, TempIndex},
};
use itertools::Itertools;
use move_model::{
    ast,
    ast::Exp,
    model::{FunctionEnv, GlobalEnv, Loc, NodeId, QualifiedId, StructId},
    ty::{Type, BOOL_TYPE, NUM_TYPE},
};

/// A builder for `FunctionData`.
pub struct FunctionDataBuilder<'env> {
    pub fun_env: &'env FunctionEnv<'env>,
    pub data: FunctionData,
    next_free_attr_index: usize,
    next_free_label_index: usize,
    current_loc: Loc,
}

impl<'env> FunctionDataBuilder<'env> {
    /// Creates a new builder.
    pub fn new(fun_env: &'env FunctionEnv<'env>, data: FunctionData) -> Self {
        let next_free_attr_index = data.next_free_attr_index();
        let next_free_label_index = data.next_free_label_index();
        FunctionDataBuilder {
            fun_env,
            data,
            next_free_attr_index,
            next_free_label_index,
            current_loc: fun_env.get_loc(),
        }
    }

    /// Gets the global env associated with this builder.
    pub fn global_env(&self) -> &'env GlobalEnv {
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

    /// Sets the default location from a code attribute id.
    pub fn set_loc_from_attr(&mut self, attr_id: AttrId) {
        let loc = self.data.locations.get(&attr_id).expect("location");
        self.current_loc = loc.clone();
    }

    /// Sets the default location from a node id.
    pub fn set_loc_from_node(&mut self, node_id: NodeId) {
        let loc = self.fun_env.module_env.env.get_node_loc(node_id);
        self.current_loc = loc;
    }

    /// Gets the location from the bytecode attribute.
    pub fn get_loc(&self, attr_id: AttrId) -> Loc {
        self.data
            .locations
            .get(&attr_id)
            .cloned()
            .unwrap_or_else(|| self.fun_env.get_loc())
    }

    /// Creates a new bytecode attribute id with default location.
    pub fn new_attr(&mut self) -> AttrId {
        let id = AttrId::new(self.next_free_attr_index);
        self.next_free_attr_index += 1;
        self.data.locations.insert(id, self.current_loc.clone());
        id
    }

    /// Creates a new branching label for bytecode.
    pub fn new_label(&mut self) -> Label {
        let label = Label::new(self.next_free_label_index);
        self.next_free_label_index += 1;
        label
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

    /// Make a boolean not expression.
    pub fn mk_not(&self, arg: Exp) -> Exp {
        self.mk_bool_call(ast::Operation::Not, vec![arg])
    }

    /// Make a numerical constant expression.
    pub fn mk_num_const(&self, oper: ast::Operation) -> Exp {
        self.mk_call(&NUM_TYPE, oper, vec![])
    }

    /// Join an iterator of boolean expressions with a boolean binary operator.
    pub fn mk_join_bool(&self, oper: ast::Operation, args: impl Iterator<Item = Exp>) -> Exp {
        args.fold1(|a, b| self.mk_bool_call(oper.clone(), vec![a, b]))
            .expect("at least one argument")
    }

    /// Makes a local for a temporary.
    pub fn mk_local(&self, temp: TempIndex) -> Exp {
        let ty = self.data.local_types[temp].clone();
        let node_id = self.global_env().new_node(self.current_loc.clone(), ty);
        let sym = self.fun_env.get_local_name(temp);
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
        // Perform some minimal peephole optimization
        use Bytecode::*;
        match (self.data.code.last(), &bc) {
            // jump L; L: ..
            (Some(Jump(_, label1)), Label(_, label2)) if label1 == label2 => {
                *self.data.code.last_mut().unwrap() = bc;
            }
            _ => {
                self.data.code.push(bc);
            }
        }
    }

    /// Emits a bytecode via a function which takes a freshly generated attribute id.
    pub fn emit_with<F>(&mut self, f: F)
    where
        F: FnOnce(AttrId) -> Bytecode,
    {
        let attr_id = self.new_attr();
        self.emit(f(attr_id))
    }
}
