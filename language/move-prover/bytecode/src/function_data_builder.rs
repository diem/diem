// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides a builder for `FunctionData`, including building expressions and rewriting
//! bytecode.

use crate::{
    function_target::{FunctionData, FunctionTarget},
    stackless_bytecode::{AttrId, Bytecode, HavocKind, Label, Operation, PropKind},
};
use move_model::{
    ast::{Exp, TempIndex},
    exp_generator::ExpGenerator,
    model::{FunctionEnv, Loc},
    ty::Type,
};

#[derive(Default)]
pub struct FunctionDataBuilderOptions {
    pub no_fallthrough_jump_removal: bool,
}

/// A builder for `FunctionData`.
pub struct FunctionDataBuilder<'env> {
    pub fun_env: &'env FunctionEnv<'env>,
    pub data: FunctionData,
    pub options: FunctionDataBuilderOptions,
    next_free_attr_index: usize,
    next_free_label_index: usize,
    current_loc: Loc,
    next_vc_info: Option<String>,
    next_debug_comment: Option<String>,
}

impl<'env> ExpGenerator<'env> for FunctionDataBuilder<'env> {
    fn function_env(&self) -> &FunctionEnv<'env> {
        self.fun_env
    }

    fn get_current_loc(&self) -> Loc {
        self.current_loc.clone()
    }

    fn set_loc(&mut self, loc: Loc) {
        self.current_loc = loc;
    }

    fn add_local(&mut self, ty: Type) -> TempIndex {
        let idx = self.data.local_types.len();
        self.data.local_types.push(ty);
        idx
    }

    fn get_local_type(&self, temp: TempIndex) -> Type {
        self.data
            .local_types
            .get(temp)
            .expect("local variable")
            .clone()
    }
}

impl<'env> FunctionDataBuilder<'env> {
    /// Creates a new builder with customized options
    pub fn new_with_options(
        fun_env: &'env FunctionEnv<'env>,
        data: FunctionData,
        options: FunctionDataBuilderOptions,
    ) -> Self {
        let next_free_attr_index = data.next_free_attr_index();
        let next_free_label_index = data.next_free_label_index();
        FunctionDataBuilder {
            fun_env,
            data,
            options,
            next_free_attr_index,
            next_free_label_index,
            current_loc: fun_env.get_loc(),
            next_vc_info: None,
            next_debug_comment: None,
        }
    }

    /// Creates a new builder with options set to default values
    pub fn new(fun_env: &'env FunctionEnv<'env>, data: FunctionData) -> Self {
        Self::new_with_options(fun_env, data, FunctionDataBuilderOptions::default())
    }

    /// Gets a function target viewpoint on this builder. This locks the data for mutation
    /// until the returned value dies.
    pub fn get_target(&self) -> FunctionTarget<'_> {
        FunctionTarget::new(self.fun_env, &self.data)
    }

    /// Add a return parameter.
    pub fn add_return(&mut self, ty: Type) -> usize {
        let idx = self.data.return_types.len();
        self.data.return_types.push(ty);
        idx
    }

    /// Sets the default location as well as information about the verification condition
    /// message associated with the next instruction generated with `emit_with`.
    pub fn set_loc_and_vc_info(&mut self, loc: Loc, message: &str) {
        self.next_vc_info = Some(message.to_string());
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

    /// Emits a bytecode.
    pub fn emit(&mut self, bc: Bytecode) {
        use Bytecode::*;
        let no_fallthrough_jump_removal = self.options.no_fallthrough_jump_removal;
        // Perform some minimal peephole optimization
        match (self.data.code.last(), &bc) {
            // jump L; L: ..
            (Some(Jump(_, label1)), Label(_, label2))
                if !no_fallthrough_jump_removal && label1 == label2 =>
            {
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
        if let Some(info) = std::mem::take(&mut self.next_vc_info) {
            self.data.vc_infos.insert(attr_id, info);
        }
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
        let temp_exp = self.mk_temporary(temp);
        let definition = self.mk_identical(temp_exp.clone(), def);
        self.emit_with(|id| Bytecode::Prop(id, PropKind::Assume, definition));
        (temp, temp_exp)
    }

    /// Emits a new temporary with a havoced value of given type.
    pub fn emit_let_havoc(&mut self, ty: Type) -> (TempIndex, Exp) {
        let havoc_kind = if ty.is_mutable_reference() {
            HavocKind::MutationAll
        } else {
            HavocKind::Value
        };
        let temp = self.new_temp(ty);
        let temp_exp = self.mk_temporary(temp);
        self.emit_with(|id| {
            Bytecode::Call(id, vec![], Operation::Havoc(havoc_kind), vec![temp], None)
        });
        (temp, temp_exp)
    }
}
