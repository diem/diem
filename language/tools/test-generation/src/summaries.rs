// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    abstract_state::{AbstractState, AbstractValue, BorrowState, Mutability},
    error::VMError,
    function_instantiation_for_state, state_control_flow, state_create_struct,
    state_function_can_acquire_resource, state_local_availability_is, state_local_exists,
    state_local_kind_is, state_local_place, state_local_set, state_local_take,
    state_local_take_borrow, state_memory_safe, state_never, state_register_dereference,
    state_stack_bin_op, state_stack_function_call, state_stack_function_popn, state_stack_has,
    state_stack_has_integer, state_stack_has_polymorphic_eq, state_stack_has_reference,
    state_stack_has_struct, state_stack_is_castable, state_stack_kind_is,
    state_stack_local_polymorphic_eq, state_stack_pop, state_stack_push, state_stack_push_register,
    state_stack_push_register_borrow, state_stack_ref_polymorphic_eq,
    state_stack_satisfies_function_signature, state_stack_satisfies_struct_signature,
    state_stack_struct_borrow_field, state_stack_struct_has_field, state_stack_struct_popn,
    state_stack_unpack_struct, state_struct_is_resource, struct_instantiation_for_state,
    transitions::*,
    unpack_instantiation_for_state, with_ty_param,
};
use vm::file_format::{Bytecode, Kind, LocalsSignatureIndex, SignatureToken};

/// A `Precondition` is a boolean predicate on an `AbstractState`.
pub type Precondition = dyn Fn(&AbstractState) -> bool;

/// A `Effect` is a function that transforms on `AbstractState` to another
pub type NonInstantiableEffect = dyn Fn(&AbstractState) -> Result<AbstractState, VMError>;
pub type InstantiableEffect = dyn Fn(LocalsSignatureIndex) -> Vec<Box<NonInstantiableEffect>>;

type Instantiation = dyn Fn(&AbstractState) -> Vec<SignatureToken>;
type InstantiableInstruction = dyn Fn(LocalsSignatureIndex) -> Bytecode;

pub enum Effects {
    NoTyParams(Vec<Box<NonInstantiableEffect>>),
    TyParams(
        Box<Instantiation>,
        Box<InstantiableEffect>,
        Box<InstantiableInstruction>,
    ),
}

/// The `Summary` of a bytecode instruction contains a list of `Precondition`s
/// and a list of `Effect`s.
pub struct Summary {
    pub preconditions: Vec<Box<Precondition>>,
    pub effects: Effects,
}

/// Return the `Summary` for a bytecode instruction, `instruction`
pub fn instruction_summary(instruction: Bytecode, exact: bool) -> Summary {
    match instruction {
        Bytecode::Pop => Summary {
            preconditions: vec![
                state_stack_has!(0, None),
                state_stack_kind_is!(0, Kind::Unrestricted),
                state_memory_safe!(Some(0)),
            ],
            effects: Effects::NoTyParams(vec![state_stack_pop!()]),
        },
        Bytecode::LdU8(_) => Summary {
            preconditions: vec![],
            effects: Effects::NoTyParams(vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::U8
            ))]),
        },
        Bytecode::LdU64(_) => Summary {
            preconditions: vec![],
            effects: Effects::NoTyParams(vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::U64
            ))]),
        },
        Bytecode::LdU128(_) => Summary {
            preconditions: vec![],
            effects: Effects::NoTyParams(vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::U128
            ))]),
        },
        Bytecode::CastU8 => Summary {
            preconditions: vec![state_stack_is_castable!(SignatureToken::U8)],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::U8)),
            ]),
        },
        Bytecode::CastU64 => Summary {
            preconditions: vec![state_stack_is_castable!(SignatureToken::U64)],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::U64)),
            ]),
        },
        Bytecode::CastU128 => Summary {
            preconditions: vec![state_stack_is_castable!(SignatureToken::U128)],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::U128)),
            ]),
        },
        Bytecode::LdAddr(_) => Summary {
            preconditions: vec![],
            effects: Effects::NoTyParams(vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::Address
            ))]),
        },
        Bytecode::LdTrue => Summary {
            preconditions: vec![],
            effects: Effects::NoTyParams(vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::Bool,
            ))]),
        },
        Bytecode::LdFalse => Summary {
            preconditions: vec![],
            effects: Effects::NoTyParams(vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::Bool
            ))]),
        },
        Bytecode::LdByteArray(_) => Summary {
            preconditions: vec![],
            effects: Effects::NoTyParams(vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::ByteArray
            ))]),
        },
        Bytecode::CopyLoc(i) => Summary {
            preconditions: vec![
                state_local_exists!(i),
                state_local_kind_is!(i, Kind::Unrestricted),
                state_local_availability_is!(i, BorrowState::Available),
                state_memory_safe!(None),
            ],
            effects: Effects::NoTyParams(vec![state_local_take!(i), state_stack_push_register!()]),
        },
        Bytecode::MoveLoc(i) => Summary {
            preconditions: vec![
                state_local_exists!(i),
                state_local_availability_is!(i, BorrowState::Available),
                // TODO: We need to track borrowing of locals. Add this in when we allow the borrow
                // graph.
                // state_memory_safe!(Some(i as usize)),
            ],
            effects: Effects::NoTyParams(vec![
                state_local_take!(i),
                state_stack_push_register!(),
                state_local_set!(i, BorrowState::Unavailable),
            ]),
        },
        Bytecode::StLoc(i) => Summary {
            preconditions: vec![
                state_stack_has!(0, None),
                state_local_exists!(i),
                // TODO: This covers storing on an unrestricted local only
                state_local_kind_is!(i, Kind::Unrestricted),
                state_stack_local_polymorphic_eq!(0, i as usize),
                state_memory_safe!(Some(0)),
            ],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_local_place!(i),
                state_local_set!(i, BorrowState::Available),
            ]),
        },
        Bytecode::MutBorrowLoc(i) => Summary {
            // TODO: Add these back in when borrow graph is added
            // preconditions: vec![
            //     state_local_exists!(i),
            //     state_local_availability_is!(i, BorrowState::Available),
            //     state_memory_safe!(None),
            // ],
            preconditions: vec![state_never!()],
            effects: Effects::NoTyParams(vec![
                state_local_take_borrow!(i, Mutability::Mutable),
                state_stack_push_register!(),
            ]),
        },
        Bytecode::ImmBorrowLoc(i) => Summary {
            // TODO: Add these back in when the borrow graph is added
            //preconditions: vec![
            //    state_local_exists!(i),
            //    state_local_availability_is!(i, BorrowState::Available),
            //    state_memory_safe!(None),
            //],
            preconditions: vec![state_never!()],
            effects: Effects::NoTyParams(vec![
                state_local_take_borrow!(i, Mutability::Immutable),
                state_stack_push_register!(),
            ]),
        },
        Bytecode::ReadRef => Summary {
            preconditions: vec![
                state_stack_has_reference!(0, Mutability::Either),
                state_memory_safe!(None),
            ],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_register_dereference!(),
                state_stack_push_register!(),
            ]),
        },
        Bytecode::WriteRef => Summary {
            preconditions: vec![
                state_stack_has_reference!(0, Mutability::Mutable),
                state_stack_has!(1, None),
                state_stack_ref_polymorphic_eq!(0, 1),
                state_memory_safe!(None),
            ],
            effects: Effects::NoTyParams(vec![state_stack_pop!(), state_stack_pop!()]),
        },
        Bytecode::FreezeRef => Summary {
            preconditions: vec![
                state_stack_has_reference!(0, Mutability::Mutable),
                state_memory_safe!(None),
            ],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_register_dereference!(),
                state_stack_push_register_borrow!(Mutability::Immutable),
            ]),
        },
        Bytecode::Add
        | Bytecode::Sub
        | Bytecode::Mul
        | Bytecode::Div
        | Bytecode::Mod
        | Bytecode::BitAnd
        | Bytecode::BitOr
        | Bytecode::Xor => Summary {
            preconditions: vec![
                state_stack_has_integer!(0),
                state_stack_has_integer!(1),
                state_stack_has_polymorphic_eq!(0, 1),
            ],
            effects: Effects::NoTyParams(vec![state_stack_bin_op!()]),
        },
        Bytecode::Shl | Bytecode::Shr => Summary {
            preconditions: vec![state_stack_has_integer!(0), state_stack_has_integer!(1)],
            effects: Effects::NoTyParams(vec![state_stack_bin_op!()]),
        },
        Bytecode::Or | Bytecode::And => Summary {
            preconditions: vec![
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::Bool))),
                state_stack_has!(1, Some(AbstractValue::new_primitive(SignatureToken::Bool))),
            ],
            effects: Effects::NoTyParams(vec![state_stack_bin_op!()]),
        },
        Bytecode::Not => Summary {
            preconditions: vec![state_stack_has!(
                0,
                Some(AbstractValue::new_primitive(SignatureToken::Bool))
            )],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ]),
        },
        Bytecode::Eq | Bytecode::Neq => Summary {
            preconditions: vec![
                state_stack_has!(0, None),
                state_stack_has!(1, None),
                state_stack_kind_is!(0, Kind::Unrestricted),
                state_stack_has_polymorphic_eq!(0, 1),
                state_memory_safe!(Some(0)),
                state_memory_safe!(Some(1)),
            ],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ]),
        },
        Bytecode::Lt | Bytecode::Gt | Bytecode::Le | Bytecode::Ge => Summary {
            preconditions: vec![
                state_stack_has_integer!(0),
                state_stack_has_integer!(1),
                state_stack_has_polymorphic_eq!(0, 1),
            ],
            effects: Effects::NoTyParams(vec![state_stack_bin_op!(AbstractValue::new_primitive(
                SignatureToken::Bool
            ))]),
        },
        Bytecode::GetTxnSenderAddress => Summary {
            preconditions: vec![],
            effects: Effects::NoTyParams(vec![state_stack_push!(AbstractValue::new_primitive(
                SignatureToken::Address,
            ))]),
        },
        Bytecode::Pack(i, exact_instantiation) => Summary {
            preconditions: vec![state_stack_satisfies_struct_signature!(
                i,
                exact,
                exact_instantiation
            )],
            effects: Effects::TyParams(
                struct_instantiation_for_state!(i, exact, exact_instantiation),
                with_ty_param!((exact, exact_instantiation) => instantiation,
                    vec![
                        state_stack_struct_popn!(i),
                        state_create_struct!(i, instantiation),
                        state_stack_push_register!(),
                    ]
                ),
                with_ty_param!((exact, exact_instantiation) => instantiation, Bytecode::Pack(i, instantiation)),
            ),
        },
        Bytecode::Unpack(i, exact_instantiation) => Summary {
            preconditions: vec![state_stack_has_struct!(Some(i))],
            effects: Effects::TyParams(
                unpack_instantiation_for_state!(),
                with_ty_param!((exact, exact_instantiation) => instantiation,
                    vec![
                        state_stack_pop!(),
                        state_stack_unpack_struct!(i, instantiation)
                    ]
                ),
                with_ty_param!((exact, exact_instantiation) => instantiation, Bytecode::Unpack(i, instantiation)),
            ),
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
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_stack_push!(AbstractValue::new_primitive(SignatureToken::Bool)),
            ]),
        },
        Bytecode::MutBorrowField(i) => Summary {
            preconditions: vec![
                state_stack_has_reference!(0, Mutability::Mutable),
                state_stack_struct_has_field!(i),
                state_memory_safe!(None),
            ],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_stack_struct_borrow_field!(i),
            ]),
        },
        Bytecode::ImmBorrowField(i) => Summary {
            preconditions: vec![
                state_stack_has_reference!(0, Mutability::Immutable),
                state_stack_struct_has_field!(i),
                state_memory_safe!(None),
            ],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_stack_struct_borrow_field!(i),
            ]),
        },
        Bytecode::MutBorrowGlobal(i, instantiation) => Summary {
            preconditions: vec![
                state_stack_has!(
                    0,
                    Some(AbstractValue::new_primitive(SignatureToken::Address))
                ),
                state_struct_is_resource!(i),
                state_memory_safe!(None),
            ],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_create_struct!(i, instantiation),
                state_stack_push_register_borrow!(Mutability::Mutable),
            ]),
        },
        Bytecode::ImmBorrowGlobal(i, instantiation) => Summary {
            preconditions: vec![
                state_stack_has!(
                    0,
                    Some(AbstractValue::new_primitive(SignatureToken::Address))
                ),
                state_struct_is_resource!(i),
                state_memory_safe!(None),
            ],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_create_struct!(i, instantiation),
                state_stack_push_register_borrow!(Mutability::Immutable),
            ]),
        },
        Bytecode::MoveFrom(i, instantiation) => Summary {
            preconditions: vec![
                state_function_can_acquire_resource!(),
                state_struct_is_resource!(i),
                state_stack_has!(
                    0,
                    Some(AbstractValue::new_primitive(SignatureToken::Address))
                ),
            ],
            effects: Effects::NoTyParams(vec![
                state_stack_pop!(),
                state_create_struct!(i, instantiation),
                state_stack_push_register!(),
            ]),
        },
        Bytecode::MoveToSender(i, _) => Summary {
            preconditions: vec![
                state_struct_is_resource!(i),
                state_stack_has_struct!(Some(i)),
                state_memory_safe!(None),
            ],
            effects: Effects::NoTyParams(vec![state_stack_pop!()]),
        },
        Bytecode::Call(i, exact_instantiation) => Summary {
            preconditions: vec![state_stack_satisfies_function_signature!(i)],
            effects: Effects::TyParams(
                function_instantiation_for_state!(i),
                with_ty_param!((exact, exact_instantiation) => instantiation,
                    vec![
                        state_stack_function_popn!(i),
                        state_stack_function_call!(i, instantiation),
                    ]
                ),
                with_ty_param!((exact, exact_instantiation) => instantiation, Bytecode::Call(i, instantiation)),
            ),
        },
        // Control flow instructions are called manually and thus have
        // `state_control_flow!()` as their precondition
        Bytecode::Branch(_) => Summary {
            preconditions: vec![state_control_flow!()],
            effects: Effects::NoTyParams(vec![]),
        },
        Bytecode::BrTrue(_) => Summary {
            preconditions: vec![
                state_control_flow!(),
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::Bool))),
            ],
            effects: Effects::NoTyParams(vec![state_stack_pop!()]),
        },
        Bytecode::BrFalse(_) => Summary {
            preconditions: vec![
                state_control_flow!(),
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::Bool))),
            ],
            effects: Effects::NoTyParams(vec![state_stack_pop!()]),
        },
        Bytecode::Ret => Summary {
            preconditions: vec![state_control_flow!()],
            effects: Effects::NoTyParams(vec![]),
        },
        Bytecode::Abort => Summary {
            preconditions: vec![
                state_control_flow!(),
                state_stack_has!(0, Some(AbstractValue::new_primitive(SignatureToken::U64))),
            ],
            effects: Effects::NoTyParams(vec![state_stack_pop!()]),
        },
        // XXX: Deprecated instructions, to remove
        Bytecode::GetTxnGasUnitPrice
        | Bytecode::GetTxnMaxGasUnits
        | Bytecode::GetTxnSequenceNumber
        | Bytecode::GetTxnPublicKey
        | Bytecode::GetGasRemaining => Summary {
            preconditions: vec![state_never!()],
            effects: Effects::NoTyParams(vec![]),
        },
    }
}
