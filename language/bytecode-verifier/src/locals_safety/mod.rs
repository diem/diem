// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the transfer functions for verifying local safety of a
//! procedure body.
//! It is concerned with the assignment state of a local variable at the time of usage, which is
//! a control flow sensitive check

mod abstract_state;

use crate::{
    absint::{AbstractInterpreter, BlockInvariant, BlockPostcondition, TransferFunctions},
    binary_views::{BinaryIndexedView, FunctionView},
};
use abstract_state::{AbstractState, LocalState};
use libra_types::vm_status::StatusCode;
use mirai_annotations::*;
use vm::{
    errors::{PartialVMError, PartialVMResult},
    file_format::{Bytecode, CodeOffset, Kind},
};

pub(crate) fn verify<'a>(
    resolver: &BinaryIndexedView,
    function_view: &'a FunctionView<'a>,
) -> PartialVMResult<()> {
    let initial_state = AbstractState::new(resolver, function_view);
    let inv_map = LocalsSafetyAnalysis().analyze_function(initial_state, &function_view);
    // Report all the join failures
    for (_block_id, BlockInvariant { post, .. }) in inv_map {
        match post {
            BlockPostcondition::Error(err) => return Err(err),
            // Block might be unprocessed if all predecessors had an error
            BlockPostcondition::Unprocessed | BlockPostcondition::Success => (),
        }
    }
    Ok(())
}

fn execute_inner(
    state: &mut AbstractState,
    bytecode: &Bytecode,
    offset: CodeOffset,
) -> PartialVMResult<()> {
    match bytecode {
        Bytecode::StLoc(idx) => match (state.local_state(*idx), state.local_kind(*idx)) {
            (LocalState::MaybeResourceful, _)
            | (LocalState::Available, Kind::Resource)
            | (LocalState::Available, Kind::All) => {
                return Err(state.error(StatusCode::STLOC_UNSAFE_TO_DESTROY_ERROR, offset))
            }
            _ => state.set_available(*idx),
        },

        Bytecode::MoveLoc(idx) => match state.local_state(*idx) {
            LocalState::MaybeResourceful | LocalState::Unavailable => {
                return Err(state.error(StatusCode::MOVELOC_UNAVAILABLE_ERROR, offset))
            }
            LocalState::Available => state.set_unavailable(*idx),
        },

        Bytecode::CopyLoc(idx) => match state.local_state(*idx) {
            LocalState::MaybeResourceful => {
                // checked in type checking
                return Err(state.error(StatusCode::VERIFIER_INVARIANT_VIOLATION, offset));
            }
            LocalState::Unavailable => {
                return Err(state.error(StatusCode::COPYLOC_UNAVAILABLE_ERROR, offset))
            }
            LocalState::Available => (),
        },

        Bytecode::MutBorrowLoc(idx) | Bytecode::ImmBorrowLoc(idx) => {
            match state.local_state(*idx) {
                LocalState::Unavailable | LocalState::MaybeResourceful => {
                    return Err(state.error(StatusCode::BORROWLOC_UNAVAILABLE_ERROR, offset))
                }
                LocalState::Available => (),
            }
        }

        Bytecode::Ret => {
            let local_states = state.local_states();
            let local_kinds = state.local_kinds();
            checked_precondition!(local_states.len() == local_kinds.len());
            for (local_state, local_kind) in local_states.iter().zip(local_kinds) {
                match (local_state, local_kind) {
                    (LocalState::MaybeResourceful, _)
                    | (LocalState::Available, Kind::Resource)
                    | (LocalState::Available, Kind::All) => {
                        return Err(state.error(StatusCode::UNSAFE_RET_UNUSED_RESOURCES, offset))
                    }
                    _ => (),
                }
            }
        }

        Bytecode::Pop
        | Bytecode::BrTrue(_)
        | Bytecode::BrFalse(_)
        | Bytecode::Abort
        | Bytecode::Branch(_)
        | Bytecode::Nop
        | Bytecode::FreezeRef
        | Bytecode::MutBorrowField(_)
        | Bytecode::MutBorrowFieldGeneric(_)
        | Bytecode::ImmBorrowField(_)
        | Bytecode::ImmBorrowFieldGeneric(_)
        | Bytecode::LdU8(_)
        | Bytecode::LdU64(_)
        | Bytecode::LdU128(_)
        | Bytecode::LdConst(_)
        | Bytecode::LdTrue
        | Bytecode::LdFalse
        | Bytecode::Call(_)
        | Bytecode::CallGeneric(_)
        | Bytecode::Pack(_)
        | Bytecode::PackGeneric(_)
        | Bytecode::Unpack(_)
        | Bytecode::UnpackGeneric(_)
        | Bytecode::ReadRef
        | Bytecode::WriteRef
        | Bytecode::CastU8
        | Bytecode::CastU64
        | Bytecode::CastU128
        | Bytecode::Add
        | Bytecode::Sub
        | Bytecode::Mul
        | Bytecode::Mod
        | Bytecode::Div
        | Bytecode::BitOr
        | Bytecode::BitAnd
        | Bytecode::Xor
        | Bytecode::Shl
        | Bytecode::Shr
        | Bytecode::Or
        | Bytecode::And
        | Bytecode::Not
        | Bytecode::Eq
        | Bytecode::Neq
        | Bytecode::Lt
        | Bytecode::Gt
        | Bytecode::Le
        | Bytecode::Ge
        | Bytecode::MutBorrowGlobal(_)
        | Bytecode::MutBorrowGlobalGeneric(_)
        | Bytecode::ImmBorrowGlobal(_)
        | Bytecode::ImmBorrowGlobalGeneric(_)
        | Bytecode::Exists(_)
        | Bytecode::ExistsGeneric(_)
        | Bytecode::MoveFrom(_)
        | Bytecode::MoveFromGeneric(_)
        | Bytecode::MoveTo(_)
        | Bytecode::MoveToGeneric(_) => (),
    };
    Ok(())
}

struct LocalsSafetyAnalysis();

impl TransferFunctions for LocalsSafetyAnalysis {
    type State = AbstractState;
    type AnalysisError = PartialVMError;

    fn execute(
        &mut self,
        state: &mut Self::State,
        bytecode: &Bytecode,
        index: CodeOffset,
        _last_index: CodeOffset,
    ) -> Result<(), Self::AnalysisError> {
        execute_inner(state, bytecode, index)
    }
}

impl AbstractInterpreter for LocalsSafetyAnalysis {}
