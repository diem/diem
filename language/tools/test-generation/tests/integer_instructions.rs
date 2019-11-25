// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate test_generation;
use itertools::Itertools;
use test_generation::abstract_state::{AbstractState, AbstractValue};
use vm::file_format::{Bytecode, SignatureToken};

mod common;

const INTEGER_TYPES: &[SignatureToken] = &[
    SignatureToken::U8,
    SignatureToken::U64,
    SignatureToken::U128,
];

#[test]
fn bytecode_bin_ops() {
    for (op, ty) in [
        Bytecode::Add,
        Bytecode::Sub,
        Bytecode::Mul,
        Bytecode::Div,
        Bytecode::Mod,
        Bytecode::BitAnd,
        Bytecode::BitOr,
        Bytecode::Xor,
    ]
    .iter()
    .cartesian_product(INTEGER_TYPES.iter())
    {
        let mut state1 = AbstractState::new();
        state1.stack_push(AbstractValue::new_primitive(ty.clone()));
        state1.stack_push(AbstractValue::new_primitive(ty.clone()));
        let (state2, _) = common::run_instruction(op.clone(), state1);
        assert_eq!(
            state2.stack_peek(0),
            Some(AbstractValue::new_primitive(ty.clone())),
            "stack type postcondition not met"
        );
    }
}

#[test]
fn bytecode_shl_shr() {
    let tys = INTEGER_TYPES
        .iter()
        .cartesian_product(INTEGER_TYPES.iter())
        .filter(|(ty1, ty2)| ty1 != ty2);
    for (op, (ty1, ty2)) in [Bytecode::Shl, Bytecode::Shr].iter().cartesian_product(tys) {
        let mut state1 = AbstractState::new();
        state1.stack_push(AbstractValue::new_primitive(ty1.clone()));
        state1.stack_push(AbstractValue::new_primitive(ty2.clone()));
        let (state2, _) = common::run_instruction(op.clone(), state1);
        assert_eq!(
            state2.stack_peek(0),
            Some(AbstractValue::new_primitive(ty1.clone())),
            "stack type postcondition not met"
        );
    }
}

#[test]
fn bytecode_casting_ops() {
    for (op, ty1, ty2) in [
        (Bytecode::CastU8, SignatureToken::U8, SignatureToken::U8),
        (Bytecode::CastU64, SignatureToken::U64, SignatureToken::U8),
        (Bytecode::CastU64, SignatureToken::U64, SignatureToken::U64),
        (Bytecode::CastU128, SignatureToken::U128, SignatureToken::U8),
        (
            Bytecode::CastU128,
            SignatureToken::U128,
            SignatureToken::U64,
        ),
        (
            Bytecode::CastU128,
            SignatureToken::U128,
            SignatureToken::U128,
        ),
    ]
    .iter()
    {
        let mut state1 = AbstractState::new();
        state1.stack_push(AbstractValue::new_primitive(ty2.clone()));
        let (state2, _) = common::run_instruction(op.clone(), state1);
        assert_eq!(
            state2.stack_peek(0),
            Some(AbstractValue::new_primitive(ty1.clone())),
            "stack type postcondition not met"
        );
    }
}
