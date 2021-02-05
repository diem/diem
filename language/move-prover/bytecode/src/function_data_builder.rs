// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides a builder for `FunctionData`, including building expressions and rewriting
//! bytecode.

use crate::{
    function_target::{FunctionData, FunctionTarget},
    stackless_bytecode::{AttrId, Bytecode, Label, Operation, PropKind},
};
use itertools::Itertools;
use move_model::{
    ast,
    ast::{Exp, TempIndex, Value},
    model::{
        ConditionInfo, ConditionTag, FunctionEnv, GlobalEnv, Loc, NodeId, QualifiedId, StructId,
    },
    ty::{Type, BOOL_TYPE, NUM_TYPE},
};

/// A builder for `FunctionData`.
pub struct FunctionDataBuilder<'env> {
    pub fun_env: &'env FunctionEnv<'env>,
    pub data: FunctionData,
    next_free_attr_index: usize,
    next_free_label_index: usize,
    current_loc: Loc,
    next_debug_comment: Option<String>,
}

impl<'env> FunctionDataBuilder<'env> {}

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
            next_debug_comment: None,
        }
    }

    /// Gets the global env associated with this builder.
    pub fn global_env(&self) -> &'env GlobalEnv {
        self.fun_env.module_env.env
    }

    /// Gets a function target viewpoint on this builder. This locks the data for mutation
    /// until the returned value dies.
    pub fn get_target(&self) -> FunctionTarget<'_> {
        FunctionTarget::new(self.fun_env, &self.data)
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

    /// Sets the default location as well as information about verification conditions
    /// at this location. The later is stored in the global environment where it can later be
    /// retrieved based on location when mapping verification failures back to source level.
    pub fn set_loc_and_vc_info(&mut self, loc: Loc, tag: ConditionTag, message: &str) {
        self.global_env()
            .set_condition_info(loc.clone(), tag, ConditionInfo::for_message(message));
        self.set_loc(loc);
    }

    /// Sets the default location from a code attribute id.
    pub fn set_loc_from_attr(&mut self, attr_id: AttrId) {
        let loc = if let Some(l) = self.data.locations.get(&attr_id) {
            l.clone()
        } else {
            self.global_env().unknown_loc()
        };
        self.current_loc = loc;
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

    /// Make a boolean constant expression.
    pub fn mk_bool_const(&self, value: bool) -> Exp {
        let node_id = self
            .global_env()
            .new_node(self.current_loc.clone(), BOOL_TYPE.clone());
        Exp::Value(node_id, Value::Bool(value))
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

    /// Make an equality expression.
    pub fn mk_eq(&self, arg1: Exp, arg2: Exp) -> Exp {
        self.mk_bool_call(ast::Operation::Eq, vec![arg1, arg2])
    }

    /// Make an and expression.
    pub fn mk_and(&self, arg1: Exp, arg2: Exp) -> Exp {
        self.mk_bool_call(ast::Operation::And, vec![arg1, arg2])
    }

    /// Make an or expression.
    pub fn mk_or(&self, arg1: Exp, arg2: Exp) -> Exp {
        self.mk_bool_call(ast::Operation::Or, vec![arg1, arg2])
    }

    /// Make an implies expression.
    pub fn mk_implies(&self, arg1: Exp, arg2: Exp) -> Exp {
        self.mk_bool_call(ast::Operation::Implies, vec![arg1, arg2])
    }

    /// Make a numerical expression for some of the builtin constants.
    pub fn mk_builtin_num_const(&self, oper: ast::Operation) -> Exp {
        use ast::Operation::*;
        assert!(matches!(oper, MaxU8 | MaxU64 | MaxU128));
        self.mk_call(&NUM_TYPE, oper, vec![])
    }

    /// Join an iterator of boolean expressions with a boolean binary operator.
    pub fn mk_join_bool(
        &self,
        oper: ast::Operation,
        args: impl Iterator<Item = Exp>,
    ) -> Option<Exp> {
        args.fold1(|a, b| self.mk_bool_call(oper.clone(), vec![a, b]))
    }

    /// Join two boolean optional expression with binary operator.
    pub fn mk_join_opt_bool(
        &self,
        oper: ast::Operation,
        arg1: Option<Exp>,
        arg2: Option<Exp>,
    ) -> Option<Exp> {
        match (arg1, arg2) {
            (Some(a1), Some(a2)) => Some(self.mk_bool_call(oper, vec![a1, a2])),
            (Some(a1), None) => Some(a1),
            (None, Some(a2)) => Some(a2),
            _ => None,
        }
    }

    /// Makes a local for a temporary.
    pub fn mk_local(&self, temp: TempIndex) -> Exp {
        let ty = self.data.local_types[temp].clone();
        let node_id = self.global_env().new_node(self.current_loc.clone(), ty);
        Exp::Temporary(node_id, temp)
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

    /// Emits a sequence of bytecodes.
    pub fn emit_vec(&mut self, bcs: Vec<Bytecode>) {
        for bc in bcs {
            self.emit(bc);
        }
    }

    /// Emits a bytecode via a function which takes a freshly generated attribute id.
    pub fn emit_with<F>(&mut self, f: F)
    where
        F: FnOnce(AttrId) -> Bytecode,
    {
        let attr_id = self.new_attr();
        if let Some(comment) = std::mem::take(&mut self.next_debug_comment) {
            self.data.debug_comments.insert(attr_id, comment);
        }
        self.emit(f(attr_id))
    }

    /// Sets the debug comment which should be associated with the next instruction
    /// emitted with `self.emit_with(|id| ..)`.
    pub fn set_next_debug_comment(&mut self, comment: String) {
        self.next_debug_comment = Some(comment);
    }

    /// This will clear the state that the next `self.emit_with(..)` will add a debug comment.
    pub fn clear_next_debug_comment(&mut self) {
        self.next_debug_comment = None;
    }

    /// Emits a let: this creates a new temporary and emits an assumption that this temporary
    /// is equal to the given expression. This can be used to abbreviate large expressions
    /// which are used multiple times, or get the value of an expression into a temporary for
    /// bytecode. Returns the temporary and a local expression referring to it.
    pub fn emit_let(&mut self, def: Exp) -> (TempIndex, Exp) {
        let ty = self.global_env().get_node_type(def.node_id());
        let temp = self.new_temp(ty);
        let temp_exp = self.mk_local(temp);
        let definition = self.mk_eq(temp_exp.clone(), def);
        self.emit_with(|id| Bytecode::Prop(id, PropKind::Assume, definition));
        (temp, temp_exp)
    }

    /// Emits a new temporary with a havoced value of given type.
    pub fn emit_let_havoc(&mut self, ty: Type) -> (TempIndex, Exp) {
        let temp = self.new_temp(ty);
        let temp_exp = self.mk_local(temp);
        self.emit_with(|id| Bytecode::Call(id, vec![temp], Operation::Havoc, vec![], None));
        (temp, temp_exp)
    }
}
