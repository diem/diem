// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate test_generation;
use move_binary_format::file_format::{Bytecode, SignatureToken};
use test_generation::abstract_state::{AbstractState, AbstractValue};

mod common;

#[test]
fn bytecode_and() {
    let mut state1 = AbstractState::new();
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Bool));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Bool));
    let (state2, _) = common::run_instruction(Bytecode::And, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::Bool)),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_or() {
    let mut state1 = AbstractState::new();
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Bool));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Bool));
    let (state2, _) = common::run_instruction(Bytecode::Or, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::Bool)),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_not() {
    let mut state1 = AbstractState::new();
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Bool));
    state1.stack_push(AbstractValue::new_primitive(SignatureToken::Bool));
    let (state2, _) = common::run_instruction(Bytecode::Not, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::Bool)),
        "stack type postcondition not met"
    );
}
