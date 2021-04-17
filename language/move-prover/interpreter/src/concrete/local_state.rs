// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// TODO (mengxu) remove this when the module is in good shape
#![allow(dead_code)]

use move_binary_format::file_format::CodeOffset;
use move_model::ast::TempIndex;

use crate::concrete::{
    ty::Type,
    value::{BaseValue, LocalSlot, Pointer, RefTypedValue, TypedValue},
};

#[derive(Debug)]
pub enum TerminationStatus {
    None,
    PostAbort,
    Return(Vec<TypedValue>),
    Abort(u64),
}

pub struct LocalState {
    /// slots that holds local variables
    slots: Vec<LocalSlot>,
    /// program counter
    pc: CodeOffset,
    /// Whether we set the PC to branch in the handling of last bytecode
    pc_branch: bool,
    /// termination status
    termination: TerminationStatus,
}

impl LocalState {
    pub fn new(slots: Vec<LocalSlot>) -> Self {
        Self {
            slots,
            pc: 0,
            pc_branch: false,
            termination: TerminationStatus::None,
        }
    }

    pub fn get_type(&self, index: TempIndex) -> &Type {
        self.slots.get(index).unwrap().get_type()
    }

    pub fn get_value(&self, index: TempIndex) -> RefTypedValue {
        self.slots.get(index).unwrap().get_value()
    }
    pub fn put_value(&mut self, index: TempIndex, val: BaseValue, ptr: Pointer) {
        self.slots.get_mut(index).unwrap().put_value(val, ptr)
    }
    pub fn del_value(&mut self, index: TempIndex) -> TypedValue {
        self.slots.get_mut(index).unwrap().del_value()
    }

    pub fn get_pc(&self) -> CodeOffset {
        self.pc
    }
    pub fn set_pc(&mut self, pc: CodeOffset) {
        if cfg!(debug_assertions) {
            assert!(!self.pc_branch);
        }
        self.pc = pc;
        self.pc_branch = true;
    }
    pub fn ready_pc_for_next_instruction(&mut self) {
        if self.pc_branch {
            self.pc_branch = false
        } else {
            self.pc += 1;
        }
    }

    pub fn transit_to_post_abort(&mut self) {
        if cfg!(debug_assertions) {
            assert!(matches!(self.termination, TerminationStatus::None));
        }
        self.termination = TerminationStatus::PostAbort;
    }
    pub fn is_terminated(&self) -> bool {
        matches!(
            self.termination,
            TerminationStatus::Return(_) | TerminationStatus::Abort(_)
        )
    }
    pub fn terminate_with_abort(&mut self, abort_code: u64) {
        if cfg!(debug_assertions) {
            assert!(!self.is_terminated());
        }
        self.termination = TerminationStatus::Abort(abort_code);
    }
    pub fn terminate_with_return(&mut self, ret_vals: Vec<TypedValue>) {
        if cfg!(debug_assertions) {
            assert!(!self.is_terminated());
        }
        self.termination = TerminationStatus::Return(ret_vals);
    }
    pub fn into_termination_status(self) -> TerminationStatus {
        self.termination
    }
}
