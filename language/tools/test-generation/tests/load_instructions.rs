// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate test_generation;
use test_generation::abstract_state::{AbstractState, AbstractValue};
use vm::file_format::{AddressPoolIndex, Bytecode, SignatureToken, UserStringIndex};

mod common;

#[test]
fn bytecode_ldconst() {
    let state1 = AbstractState::new();
    let state2 = common::run_instruction(Bytecode::LdConst(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::U64)),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_ldtrue() {
    let state1 = AbstractState::new();
    let state2 = common::run_instruction(Bytecode::LdTrue, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::Bool)),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_ldfalse() {
    let state1 = AbstractState::new();
    let state2 = common::run_instruction(Bytecode::LdFalse, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::Bool)),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_ldstr() {
    let state1 = AbstractState::new();
    let state2 = common::run_instruction(Bytecode::LdStr(UserStringIndex::new(0)), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::String)),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_ldaddr() {
    let state1 = AbstractState::new();
    let state2 = common::run_instruction(Bytecode::LdAddr(AddressPoolIndex::new(0)), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::Address)),
        "stack type postcondition not met"
    );
}
