// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    abstract_state::{AbstractState, BorrowState},
    state_local_exists, state_local_is, state_local_place, state_local_set, state_local_take,
    state_local_take_borrow, state_never, state_stack_has, state_stack_has_polymorphic_eq,
    state_stack_local_polymorphic_eq, state_stack_pop, state_stack_push, state_stack_push_register,
    transitions::*,
};
use vm::file_format::{Bytecode, SignatureToken};

/// A `Precondition` is a boolean predicate on an `AbstractState`.
type Precondition = dyn Fn(&AbstractState) -> bool;

/// A `Effect` is a function that transforms on `AbstractState` to another
type Effect = dyn Fn(&AbstractState) -> AbstractState;

/// The `Summary` of a bytecode instruction contains a list of `Precondition`s
/// and a list of `Effect`s.
pub struct Summary {
    pub preconditions: Vec<Box<Precondition>>,
    pub effects: Vec<Box<Effect>>,
}

/// Return the `Summary` for a bytecode instruction, `instruction`
pub fn instruction_summary(instruction: Bytecode) -> Summary {
    match instruction {
        Bytecode::Pop => Summary {
            preconditions: vec![state_stack_has!(0, None)],
            effects: vec![state_stack_pop!()],
        },
        Bytecode::LdConst(_) => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(SignatureToken::U64)],
        },
        Bytecode::LdStr(_) => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(SignatureToken::String)],
        },
        Bytecode::LdAddr(_) => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(SignatureToken::Address)],
        },
        Bytecode::LdTrue => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(SignatureToken::Bool)],
        },
        Bytecode::LdFalse => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(SignatureToken::Bool)],
        },
        Bytecode::LdByteArray(_) => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(SignatureToken::ByteArray)],
        },
        Bytecode::CopyLoc(i) => Summary {
            preconditions: vec![
                state_local_exists!(i),
                state_local_is!(i, BorrowState::Available),
            ],
            effects: vec![state_local_take!(i), state_stack_push_register!()],
        },
        Bytecode::MoveLoc(i) => Summary {
            preconditions: vec![
                state_local_exists!(i),
                state_local_is!(i, BorrowState::Available),
            ],
            effects: vec![
                state_local_take!(i),
                state_stack_push_register!(),
                state_local_set!(i, BorrowState::Unavailable),
            ],
        },
        Bytecode::StLoc(i) => Summary {
            preconditions: vec![
                state_stack_has!(0, None),
                // TODO: This covers storing on an unrestricted local only
                state_local_exists!(i),
                state_stack_local_polymorphic_eq!(0, i as usize),
            ],
            effects: vec![
                state_stack_pop!(),
                state_local_place!(i),
                state_local_set!(i, BorrowState::Available),
            ],
        },
        Bytecode::MutBorrowLoc(i) => Summary {
            preconditions: vec![
                state_local_exists!(i),
                state_local_is!(i, BorrowState::Available),
            ],
            effects: vec![
                state_local_take_borrow!(i, true),
                state_stack_push_register!(),
            ],
        },
        Bytecode::ImmBorrowLoc(i) => Summary {
            preconditions: vec![
                state_local_exists!(i),
                state_local_is!(i, BorrowState::Available),
            ],
            effects: vec![
                state_local_take_borrow!(i, false),
                state_stack_push_register!(),
            ],
        },
        Bytecode::Add => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::U64)),
                state_stack_has!(1, Some(SignatureToken::U64)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::U64),
            ],
        },
        Bytecode::Sub => Summary {
            preconditions: vec![
                // TODO: op1 needs to be >= op2 (negative numbers not supported)
                state_stack_has!(0, Some(SignatureToken::U64)),
                state_stack_has!(1, Some(SignatureToken::U64)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::U64),
            ],
        },
        Bytecode::Mul => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::U64)),
                state_stack_has!(1, Some(SignatureToken::U64)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::U64),
            ],
        },
        Bytecode::Div => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::U64)),
                state_stack_has!(1, Some(SignatureToken::U64)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::U64),
            ],
        },
        Bytecode::Mod => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::U64)),
                state_stack_has!(1, Some(SignatureToken::U64)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::U64),
            ],
        },
        Bytecode::BitAnd => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::U64)),
                state_stack_has!(1, Some(SignatureToken::U64)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::U64),
            ],
        },
        Bytecode::BitOr => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::U64)),
                state_stack_has!(1, Some(SignatureToken::U64)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::U64),
            ],
        },
        Bytecode::Xor => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::U64)),
                state_stack_has!(1, Some(SignatureToken::U64)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::U64),
            ],
        },
        Bytecode::Or => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::Bool)),
                state_stack_has!(1, Some(SignatureToken::Bool)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::Bool),
            ],
        },
        Bytecode::And => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::Bool)),
                state_stack_has!(1, Some(SignatureToken::Bool)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::Bool),
            ],
        },
        Bytecode::Not => Summary {
            preconditions: vec![state_stack_has!(0, Some(SignatureToken::Bool))],
            effects: vec![state_stack_pop!(), state_stack_push!(SignatureToken::Bool)],
        },
        Bytecode::Eq => Summary {
            preconditions: vec![
                state_stack_has!(0, None),
                state_stack_has!(1, None),
                state_stack_has_polymorphic_eq!(0, 1),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::Bool),
            ],
        },
        Bytecode::Neq => Summary {
            preconditions: vec![
                state_stack_has!(0, None),
                state_stack_has!(1, None),
                state_stack_has_polymorphic_eq!(0, 1),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::Bool),
            ],
        },
        Bytecode::Lt => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::U64)),
                state_stack_has!(1, Some(SignatureToken::U64)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::Bool),
            ],
        },
        Bytecode::Gt => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::U64)),
                state_stack_has!(1, Some(SignatureToken::U64)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::Bool),
            ],
        },
        Bytecode::Le => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::U64)),
                state_stack_has!(1, Some(SignatureToken::U64)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::Bool),
            ],
        },
        Bytecode::Ge => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(SignatureToken::U64)),
                state_stack_has!(1, Some(SignatureToken::U64)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(SignatureToken::Bool),
            ],
        },
        Bytecode::GetTxnGasUnitPrice => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(SignatureToken::U64)],
        },
        Bytecode::GetTxnMaxGasUnits => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(SignatureToken::U64)],
        },
        Bytecode::GetGasRemaining => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(SignatureToken::U64)],
        },
        Bytecode::GetTxnSequenceNumber => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(SignatureToken::U64)],
        },
        Bytecode::Branch(_) => Summary {
            preconditions: vec![state_never!()],
            effects: vec![],
        },
        Bytecode::BrTrue(_) => Summary {
            preconditions: vec![state_never!()],
            effects: vec![state_stack_pop!()],
        },
        Bytecode::BrFalse(_) => Summary {
            preconditions: vec![state_never!()],
            effects: vec![state_stack_pop!()],
        },
        _ => Summary {
            preconditions: vec![state_never!()],
            effects: vec![],
        },
    }
}
