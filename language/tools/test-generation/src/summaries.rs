// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    abstract_state::{AbstractState, AbstractValue, BorrowState, Mutability},
    error::VMError,
    state_create_struct, state_function_can_acquire_resource, state_local_availability_is,
    state_local_exists, state_local_kind_is, state_local_place, state_local_set, state_local_take,
    state_local_take_borrow, state_memory_safe, state_never, state_register_dereference,
    state_stack_function_call, state_stack_function_popn, state_stack_has,
    state_stack_has_polymorphic_eq, state_stack_has_reference, state_stack_has_struct,
    state_stack_kind_is, state_stack_local_polymorphic_eq, state_stack_pop, state_stack_push,
    state_stack_push_register, state_stack_push_register_borrow, state_stack_ref_polymorphic_eq,
    state_stack_satisfies_function_signature, state_stack_satisfies_struct_signature,
    state_stack_struct_borrow_field, state_stack_struct_has_field, state_stack_struct_popn,
    state_stack_unpack_struct, state_struct_is_resource,
    transitions::*,
};
use vm::file_format::{Bytecode, Kind, SignatureToken};

/// A `Precondition` is a boolean predicate on an `AbstractState`.
type Precondition = dyn Fn(&AbstractState) -> bool;

/// A `Effect` is a function that transforms on `AbstractState` to another
type Effect = dyn Fn(&AbstractState) -> Result<AbstractState, VMError>;

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
            preconditions: vec![
                state_stack_has!(0, None),
                state_stack_kind_is!(0, Kind::Unrestricted),
                state_memory_safe!(Some(0)),
            ],
            effects: vec![state_stack_pop!()],
        },
        Bytecode::LdConst(_) => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::U64
            ))],
        },
        Bytecode::LdStr(_) => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::String
            ))],
        },
        Bytecode::LdAddr(_) => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::Address
            ))],
        },
        Bytecode::LdTrue => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::Bool,
            ))],
        },
        Bytecode::LdFalse => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::Bool
            ))],
        },
        Bytecode::LdByteArray(_) => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::ByteArray
            ))],
        },
        Bytecode::CopyLoc(i) => Summary {
            preconditions: vec![
                state_local_exists!(i),
                state_local_kind_is!(i, Kind::Unrestricted),
                state_local_availability_is!(i, BorrowState::Available),
                state_memory_safe!(None),
            ],
            effects: vec![state_local_take!(i), state_stack_push_register!()],
        },
        Bytecode::MoveLoc(i) => Summary {
            preconditions: vec![
                state_local_exists!(i),
                state_local_availability_is!(i, BorrowState::Available),
                state_memory_safe!(None),
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
                state_local_exists!(i),
                // TODO: This covers storing on an unrestricted local only
                state_local_kind_is!(i, Kind::Unrestricted),
                state_stack_local_polymorphic_eq!(0, i as usize),
                state_memory_safe!(None),
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
                state_local_availability_is!(i, BorrowState::Available),
                state_memory_safe!(None),
            ],
            effects: vec![
                state_local_take_borrow!(i, Mutability::Mutable),
                state_stack_push_register!(),
            ],
        },
        Bytecode::ImmBorrowLoc(i) => Summary {
            preconditions: vec![
                state_local_exists!(i),
                state_local_availability_is!(i, BorrowState::Available),
                state_memory_safe!(None),
            ],
            effects: vec![
                state_local_take_borrow!(i, Mutability::Immutable),
                state_stack_push_register!(),
            ],
        },
        Bytecode::ReadRef => Summary {
            preconditions: vec![
                state_stack_has_reference!(0, Mutability::Either),
                state_memory_safe!(None),
            ],
            effects: vec![
                state_stack_pop!(),
                state_register_dereference!(),
                state_stack_push_register!(),
            ],
        },
        Bytecode::WriteRef => Summary {
            preconditions: vec![
                state_stack_has_reference!(0, Mutability::Mutable),
                state_stack_has!(1, None),
                state_stack_ref_polymorphic_eq!(0, 1),
                state_memory_safe!(None),
            ],
            effects: vec![state_stack_pop!(), state_stack_pop!()],
        },
        Bytecode::FreezeRef => Summary {
            preconditions: vec![
                state_stack_has_reference!(0, Mutability::Mutable),
                state_memory_safe!(None),
            ],
            effects: vec![
                state_stack_pop!(),
                state_register_dereference!(),
                state_stack_push_register_borrow!(Mutability::Immutable),
            ],
        },
        Bytecode::Add => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::U64)),
            ],
        },
        Bytecode::Sub => Summary {
            preconditions: vec![
                // TODO: op1 needs to be >= op2 (negative numbers not supported)
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::U64)),
            ],
        },
        Bytecode::Mul => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::U64)),
            ],
        },
        Bytecode::Div => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::U64)),
            ],
        },
        Bytecode::Mod => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::U64)),
            ],
        },
        Bytecode::BitAnd => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::U64)),
            ],
        },
        Bytecode::BitOr => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::U64)),
            ],
        },
        Bytecode::Xor => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::U64)),
            ],
        },
        Bytecode::Or => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::Bool))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::Bool))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ],
        },
        Bytecode::And => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::Bool))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::Bool))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ],
        },
        Bytecode::Not => Summary {
            preconditions: vec![state_stack_has!(
                0,
                Some(AbstractValue::new_primitive(SignatureToken::Bool))
            )],
            effects: vec![
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ],
        },
        Bytecode::Eq => Summary {
            preconditions: vec![
                state_stack_has!(0, None),
                state_stack_has!(1, None),
                state_stack_kind_is!(0, Kind::Unrestricted),
                state_stack_has_polymorphic_eq!(0, 1),
                state_memory_safe!(Some(0)),
                state_memory_safe!(Some(1)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ],
        },
        Bytecode::Neq => Summary {
            preconditions: vec![
                state_stack_has!(0, None),
                state_stack_has!(1, None),
                state_stack_kind_is!(0, Kind::Unrestricted),
                state_stack_has_polymorphic_eq!(0, 1),
                state_memory_safe!(Some(0)),
                state_memory_safe!(Some(1)),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ],
        },
        Bytecode::Lt => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ],
        },
        Bytecode::Gt => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ],
        },
        Bytecode::Le => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ],
        },
        Bytecode::Ge => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ],
        },
        Bytecode::GetTxnGasUnitPrice => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::U64
            ))],
        },
        Bytecode::GetTxnMaxGasUnits => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::U64,
            ))],
        },
        Bytecode::GetGasRemaining => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::U64,
            ))],
        },
        Bytecode::GetTxnSequenceNumber => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::U64,
            ))],
        },
        Bytecode::GetTxnSenderAddress => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::Address,
            ))],
        },
        Bytecode::GetTxnPublicKey => Summary {
            preconditions: vec![],
            effects: vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::ByteArray,
            ))],
        },
        Bytecode::Pack(i, _) => Summary {
            preconditions: vec![state_stack_satisfies_struct_signature!(i)],
            effects: vec![
                state_stack_struct_popn!(i),
                state_create_struct!(i),
                state_stack_push_register!(),
            ],
        },
        Bytecode::Unpack(i, _) => Summary {
            preconditions: vec![state_stack_has_struct!(Some(i))],
            effects: vec![state_stack_pop!(), state_stack_unpack_struct!(i)],
        },
        Bytecode::Exists(i, _) => Summary {
            // The result of `state_struct_is_resource` is represented abstractly
            // so concrete execution may differ
            preconditions: vec![
                state_struct_is_resource!(i),
                state_stack_has!(
                    0,
                    Some(AbstractValue::new_primitive(SignatureToken::Address))
                ),
            ],
            effects: vec![
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ],
        },
        Bytecode::MutBorrowField(i) => Summary {
            preconditions: vec![
                state_stack_has_reference!(0, Mutability::Mutable),
                state_stack_struct_has_field!(i),
                state_memory_safe!(None),
            ],
            effects: vec![state_stack_pop!(), state_stack_struct_borrow_field!(i)],
        },
        Bytecode::ImmBorrowField(i) => Summary {
            preconditions: vec![
                state_stack_has_reference!(0, Mutability::Immutable),
                state_stack_struct_has_field!(i),
                state_memory_safe!(None),
            ],
            effects: vec![state_stack_pop!(), state_stack_struct_borrow_field!(i)],
        },
        Bytecode::MutBorrowGlobal(i, _) => Summary {
            preconditions: vec![
                state_stack_has!(
                    0,
                    Some(AbstractValue::new_primitive(SignatureToken::Address))
                ),
                state_struct_is_resource!(i),
                state_memory_safe!(None),
            ],
            effects: vec![
                state_stack_pop!(),
                state_create_struct!(i),
                state_stack_push_register_borrow!(Mutability::Mutable),
            ],
        },
        Bytecode::ImmBorrowGlobal(i, _) => Summary {
            preconditions: vec![
                state_stack_has!(
                    0,
                    Some(AbstractValue::new_primitive(SignatureToken::Address))
                ),
                state_struct_is_resource!(i),
                state_memory_safe!(None),
            ],
            effects: vec![
                state_stack_pop!(),
                state_create_struct!(i),
                state_stack_push_register_borrow!(Mutability::Immutable),
            ],
        },
        Bytecode::MoveFrom(i, _) => Summary {
            preconditions: vec![
                state_function_can_acquire_resource!(),
                state_struct_is_resource!(i),
                state_stack_has!(
                    0,
                    Some(AbstractValue::new_primitive(SignatureToken::Address))
                ),
            ],
            effects: vec![
                state_stack_pop!(),
                state_create_struct!(i),
                state_stack_push_register!(),
            ],
        },
        Bytecode::MoveToSender(i, _) => Summary {
            preconditions: vec![
                state_struct_is_resource!(i),
                state_stack_has_struct!(Some(i)),
                state_memory_safe!(None),
            ],
            effects: vec![state_stack_pop!()],
        },
        Bytecode::Call(i, _) => Summary {
            preconditions: vec![state_stack_satisfies_function_signature!(i)],
            effects: vec![state_stack_function_popn!(i), state_stack_function_call!(i)],
        },
        // Control flow instructions are called manually and thus have
        // `state_never!()` as their precondition
        Bytecode::Branch(_) => Summary {
            preconditions: vec![state_never!()],
            effects: vec![],
        },
        Bytecode::BrTrue(_) => Summary {
            preconditions: vec![
                state_never!(),
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::Bool))),
            ],
            effects: vec![state_stack_pop!()],
        },
        Bytecode::BrFalse(_) => Summary {
            preconditions: vec![
                state_never!(),
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::Bool))),
            ],
            effects: vec![state_stack_pop!()],
        },
        Bytecode::Ret => Summary {
            preconditions: vec![state_never!()],
            effects: vec![],
        },
        Bytecode::Abort => Summary {
            preconditions: vec![
                state_never!(),
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: vec![state_stack_pop!()],
        },
    }
}
