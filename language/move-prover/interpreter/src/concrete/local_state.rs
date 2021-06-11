// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file implements the information needed in the local interpretation context, i.e., the
//! context created and updated when interpreting a single function.

use std::collections::BTreeMap;

use move_binary_format::errors::{Location, PartialVMError, VMError};
use move_core_types::vm_status::StatusCode;
use move_model::ast::TempIndex;

use crate::concrete::{
    ty::{CodeOffset, Type},
    value::{LocalSlot, Pointer, TypedValue},
};

#[derive(Clone, Debug)]
pub enum AbortInfo {
    /// User-specific abort
    User(u64, Location),
    /// Internal abort (e.g., integer overflow or resource does not exist in global storage)
    Internal(StatusCode, Location),
}

impl AbortInfo {
    /// Convert the AbortInfo into a VMError
    pub fn into_err(self) -> VMError {
        match self {
            Self::User(status_code, location) => PartialVMError::new(StatusCode::ABORTED)
                .with_sub_status(status_code)
                .finish(location),
            Self::Internal(status_code, location) => {
                PartialVMError::new(status_code).finish(location)
            }
        }
    }

    /// Retrieve the status code as a u64
    pub fn get_status_code(&self) -> u64 {
        match self {
            Self::User(status_code, _) => *status_code,
            Self::Internal(status_code, _) => *status_code as u64,
        }
    }
}

#[derive(Debug)]
pub enum TerminationStatus {
    /// This function has not terminated, it is running normally
    None,
    /// An abort has been triggered and the function is in post-abort state
    PostAbort(AbortInfo),
    /// The function terminated successfully with a list of return values
    Return(Vec<TypedValue>),
    /// The function terminated with an abort
    Abort(AbortInfo),
}

pub struct LocalState {
    /// slots that holds local variables
    slots: Vec<LocalSlot>,
    /// program counter
    pc: CodeOffset,
    /// whether we set the PC to branch in the handling of last bytecode
    pc_branch: bool,
    /// termination status
    termination: TerminationStatus,
    /// mutable parameters that gets destroyed during the execution
    destroyed_args: BTreeMap<TempIndex, TypedValue>,
}

impl LocalState {
    pub fn new(slots: Vec<LocalSlot>) -> Self {
        Self {
            slots,
            pc: 0,
            pc_branch: false,
            termination: TerminationStatus::None,
            destroyed_args: BTreeMap::new(),
        }
    }

    /// Get the number of slots available in this state
    pub fn num_slots(&self) -> usize {
        self.slots.len()
    }

    /// Get the type of the local slot at `index`
    pub fn get_type(&self, index: TempIndex) -> &Type {
        self.slots.get(index).unwrap().get_type()
    }

    /// Check whether the local slot at `index` holds a value
    pub fn has_value(&self, index: TempIndex) -> bool {
        self.slots.get(index).unwrap().has_value()
    }
    /// Get the value held in local slot `index`. Panics if the slot does not hold a value
    pub fn get_value(&self, index: TempIndex) -> TypedValue {
        self.slots.get(index).unwrap().get_value()
    }
    /// Put the value held in local slot `index`. Override if the slot already holds a value
    pub fn put_value_override(&mut self, index: TempIndex, val: TypedValue) {
        self.slots.get_mut(index).unwrap().put_value_override(val)
    }
    /// Put the value held in local slot `index`. Panics if the slot already holds a value
    pub fn put_value(&mut self, index: TempIndex, val: TypedValue) {
        self.slots.get_mut(index).unwrap().put_value(val)
    }
    /// Delete the value held in local slot `index`. Panics if the slot does not hold a value
    pub fn del_value(&mut self, index: TempIndex) -> TypedValue {
        self.slots.get_mut(index).unwrap().del_value()
    }

    /// Save a mutable argument that is destroyed
    pub fn save_destroyed_arg(&mut self, index: TempIndex, val: TypedValue) {
        let exists = self.destroyed_args.insert(index, val);
        if cfg!(debug_assertions) {
            assert!(exists.is_none());
        }
    }
    /// Load a mutable argument that is destroyed
    pub fn load_destroyed_arg(&mut self, index: TempIndex) -> TypedValue {
        self.destroyed_args.remove(&index).unwrap()
    }

    /// Get the current PC location (i.e., which bytecode to be executed)
    pub fn get_pc(&self) -> CodeOffset {
        self.pc
    }
    /// Set the PC location to jump to on next execution
    pub fn set_pc(&mut self, pc: CodeOffset) {
        if cfg!(debug_assertions) {
            assert!(!self.pc_branch);
        }
        self.pc = pc;
        self.pc_branch = true;
    }
    /// Decide the PC location for next bytecode instruction
    pub fn ready_pc_for_next_instruction(&mut self) {
        if self.pc_branch {
            self.pc_branch = false
        } else {
            self.pc += 1;
        }
    }

    /// Collect the pointers of the underlying values in the local slots
    pub fn collect_pointers(&self) -> BTreeMap<TempIndex, &Pointer> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| slot.get_content().map(|(_, ptr)| (idx, ptr)))
            .collect()
    }

    /// Mark that an abort is raised and we will be executing the abort action next
    pub fn transit_to_post_abort(&mut self, info: AbortInfo) {
        if cfg!(debug_assertions) {
            assert!(matches!(self.termination, TerminationStatus::None));
        }
        self.termination = TerminationStatus::PostAbort(info);
    }
    /// Check whether execution of the current function is finished or not
    pub fn is_terminated(&self) -> bool {
        matches!(
            self.termination,
            TerminationStatus::Return(_) | TerminationStatus::Abort(_)
        )
    }
    /// Check whether we are executing in a post-abort status
    pub fn is_post_abort(&self) -> bool {
        matches!(self.termination, TerminationStatus::PostAbort(_))
    }
    /// Mark that the current function terminated with an abort
    pub fn terminate_with_abort(&mut self, abort_info: AbortInfo) {
        if cfg!(debug_assertions) {
            assert!(!self.is_terminated());
        }
        let info = match &self.termination {
            TerminationStatus::None => {
                // no prior aborts has been seen, and no abort action attached
                abort_info
            }
            TerminationStatus::PostAbort(original_info) => {
                // re-abort, make sure we are aborting with the same status code
                if cfg!(debug_assertions) {
                    assert_eq!(
                        original_info.get_status_code(),
                        abort_info.get_status_code()
                    );
                }
                original_info.clone()
            }
            _ => unreachable!(),
        };
        self.termination = TerminationStatus::Abort(info);
    }
    /// Mark that the current function terminated with return values
    pub fn terminate_with_return(&mut self, ret_vals: Vec<TypedValue>) {
        if cfg!(debug_assertions) {
            assert!(!self.is_terminated());
        }
        self.termination = TerminationStatus::Return(ret_vals);
    }
    /// Consume and reduce the state into termination status
    pub fn into_termination_status(self) -> TerminationStatus {
        self.termination
    }
}
