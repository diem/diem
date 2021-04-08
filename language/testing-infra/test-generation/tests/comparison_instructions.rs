// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate test_generation;
use itertools::Itertools;
use move_binary_format::file_format::{Bytecode, SignatureToken};
use test_generation::abstract_state::{AbstractState, AbstractValue};

mod common;

const INTEGER_TYPES: &[SignatureToken] = &[
    SignatureToken::U8,
    SignatureToken::U64,
    SignatureToken::U128,
];

#[test]
fn bytecode_comparison_integers() {
    for (op, ty) in [
        Bytecode::Lt,
        Bytecode::Gt,
        Bytecode::Le,
        Bytecode::Ge,
        Bytecode::Eq,
        Bytecode::Neq,
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
            Some(AbstractValue::new_primitive(SignatureToken::Bool)),
            "stack type postcondition not met"
        );
    }
}

#[test]
fn bytecode_eq_bool() {
    let mut state1 = AbstractState::new();
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Bool));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Bool));
    let (state2, _) = common::run_instruction(Bytecode::Eq, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::Bool)),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_neq_u64() {
    let mut state1 = AbstractState::new();
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::U64));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::U64));
    let (state2, _) = common::run_instruction(Bytecode::Neq, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::Bool)),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_neq_bool() {
    let mut state1 = AbstractState::new();
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Bool));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Bool));
    let (state2, _) = common::run_instruction(Bytecode::Neq, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::Bool)),
        "stack type postcondition not met"
    );
}
