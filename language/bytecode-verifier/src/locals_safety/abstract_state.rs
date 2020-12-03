// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the abstract state for the local safety analysis.

use crate::absint::{AbstractDomain, JoinResult};
use mirai_annotations::{checked_precondition, checked_verify};
use move_core_types::vm_status::StatusCode;
use vm::{
    errors::PartialVMError,
    file_format::{CodeOffset, FunctionDefinitionIndex, Kind, LocalIndex},
};

/// LocalState represents the current assignment state of a local
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LocalState {
    /// The local does not have a value
    Unavailable,
    /// The local was assigned a resource in at least one control flow path, but was `Unavailable`
    // in at least one other path
    MaybeResourceful,
    /// The local has a value
    Available,
}
use crate::binary_views::{BinaryIndexedView, FunctionView};
use LocalState::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AbstractState {
    current_function: Option<FunctionDefinitionIndex>,
    local_kinds: Vec<Kind>,
    local_states: Vec<LocalState>,
}

impl AbstractState {
    /// create a new abstract state
    pub fn new(resolver: &BinaryIndexedView, function_view: &FunctionView) -> Self {
        let num_args = function_view.parameters().len();
        let num_locals = num_args + function_view.locals().len();
        let local_states = (0..num_locals)
            .map(|i| if i < num_args { Available } else { Unavailable })
            .collect();

        let local_kinds = function_view
            .parameters()
            .0
            .iter()
            .chain(function_view.locals().0.iter())
            .map(|st| resolver.kind(st, &function_view.type_parameters()))
            .collect();

        Self {
            current_function: function_view.index(),
            local_states,
            local_kinds,
        }
    }

    pub fn local_kind(&self, idx: LocalIndex) -> Kind {
        self.local_kinds[idx as usize]
    }

    pub fn local_kinds(&self) -> &Vec<Kind> {
        &self.local_kinds
    }

    pub fn local_state(&self, idx: LocalIndex) -> LocalState {
        self.local_states[idx as usize]
    }

    pub fn local_states(&self) -> &Vec<LocalState> {
        &self.local_states
    }

    pub fn set_available(&mut self, idx: LocalIndex) {
        self.local_states[idx as usize] = Available
    }

    pub fn set_unavailable(&mut self, idx: LocalIndex) {
        checked_precondition!(self.local_states[idx as usize] == Available);
        self.local_states[idx as usize] = Unavailable
    }

    pub fn error(&self, status: StatusCode, offset: CodeOffset) -> PartialVMError {
        PartialVMError::new(status).at_code_offset(
            self.current_function.unwrap_or(FunctionDefinitionIndex(0)),
            offset,
        )
    }

    fn join_(&self, other: &Self) -> Self {
        checked_precondition!(self.current_function == other.current_function);
        checked_precondition!(self.local_kinds.len() == other.local_kinds.len());
        checked_precondition!(self.local_states.len() == other.local_states.len());
        let current_function = self.current_function;
        let local_kinds = self.local_kinds.clone();
        let local_states = self
            .local_states
            .iter()
            .zip(&other.local_states)
            .enumerate()
            .map(|(idx, (self_state, other_state))| {
                use LocalState::*;
                match (self_state, other_state) {
                    // Unavailable on both sides, nothing to add
                    (Unavailable, Unavailable) => Unavailable,

                    (MaybeResourceful, Unavailable)
                    | (Unavailable, MaybeResourceful)
                    | (MaybeResourceful, MaybeResourceful)
                    | (MaybeResourceful, Available)
                    | (Available, MaybeResourceful) => MaybeResourceful,

                    (Unavailable, Available) | (Available, Unavailable) => {
                        match self.local_kinds[idx] {
                            Kind::All | Kind::Resource => MaybeResourceful,
                            Kind::Copyable => Unavailable,
                        }
                    }

                    (Available, Available) => Available,
                }
            })
            .collect();

        Self {
            current_function,
            local_states,
            local_kinds,
        }
    }
}

impl AbstractDomain for AbstractState {
    /// attempts to join state to self and returns the result
    fn join(&mut self, state: &AbstractState) -> JoinResult {
        let joined = Self::join_(self, state);
        checked_verify!(self.local_states.len() == joined.local_states.len());
        let locals_unchanged = self
            .local_states
            .iter()
            .zip(&joined.local_states)
            .all(|(self_state, other_state)| self_state == other_state);
        if locals_unchanged {
            JoinResult::Unchanged
        } else {
            *self = joined;
            JoinResult::Changed
        }
    }
}
