// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate test_generation;
use test_generation::abstract_state::{AbstractState, AbstractValue, BorrowState};
use vm::file_format::{Bytecode, Kind, SignatureToken};

mod common;

#[test]
fn bytecode_copyloc() {
    let mut state1 = AbstractState::new();
    state1.local_insert(
        0,
        AbstractValue::new_primitive(SignatureToken::U64),
        BorrowState::Available,
    );
    let (state2, _) = common::run_instruction(Bytecode::CopyLoc(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::U64)),
        "stack type postcondition not met"
    );
    assert_eq!(
        state2.local_get(0),
        Some(&(
            AbstractValue::new_primitive(SignatureToken::U64),
            BorrowState::Available
        )),
        "locals signature postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_copyloc_no_local() {
    let state1 = AbstractState::new();
    common::run_instruction(Bytecode::CopyLoc(0), state1);
}

#[test]
#[should_panic]
fn bytecode_copyloc_local_unavailable() {
    let mut state1 = AbstractState::new();
    state1.local_insert(
        0,
        AbstractValue::new_primitive(SignatureToken::U64),
        BorrowState::Unavailable,
    );
    common::run_instruction(Bytecode::CopyLoc(0), state1);
}

#[test]
fn bytecode_moveloc() {
    let mut state1 = AbstractState::new();
    state1.local_insert(
        0,
        AbstractValue::new_primitive(SignatureToken::U64),
        BorrowState::Available,
    );
    let (state2, _) = common::run_instruction(Bytecode::MoveLoc(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::U64)),
        "stack type postcondition not met"
    );
    assert_eq!(
        state2.local_get(0),
        Some(&(
            AbstractValue::new_primitive(SignatureToken::U64),
            BorrowState::Unavailable
        )),
        "locals signature postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_moveloc_no_local() {
    let state1 = AbstractState::new();
    common::run_instruction(Bytecode::MoveLoc(0), state1);
}

#[test]
#[should_panic]
fn bytecode_moveloc_local_unavailable() {
    let mut state1 = AbstractState::new();
    state1.local_insert(
        0,
        AbstractValue::new_primitive(SignatureToken::U64),
        BorrowState::Unavailable,
    );
    common::run_instruction(Bytecode::MoveLoc(0), state1);
}

#[test]
fn bytecode_mutborrowloc() {
    let mut state1 = AbstractState::new();
    state1.local_insert(
        0,
        AbstractValue::new_primitive(SignatureToken::U64),
        BorrowState::Available,
    );
    let (state2, _) = common::run_instruction(Bytecode::MutBorrowLoc(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_reference(
            SignatureToken::MutableReference(Box::new(SignatureToken::U64)),
            Kind::Copyable
        )),
        "stack type postcondition not met"
    );
    assert_eq!(
        state2.local_get(0),
        Some(&(
            AbstractValue::new_primitive(SignatureToken::U64),
            BorrowState::Available
        )),
        "locals signature postcondition not met"
    );
}

#[test]
fn bytecode_immborrowloc() {
    let mut state1 = AbstractState::new();
    state1.local_insert(
        0,
        AbstractValue::new_primitive(SignatureToken::U64),
        BorrowState::Available,
    );
    let (state2, _) = common::run_instruction(Bytecode::ImmBorrowLoc(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_reference(
            SignatureToken::Reference(Box::new(SignatureToken::U64),),
            Kind::Copyable
        )),
        "stack type postcondition not met"
    );
    assert_eq!(
        state2.local_get(0),
        Some(&(
            AbstractValue::new_primitive(SignatureToken::U64),
            BorrowState::Available
        )),
        "locals signature postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_mutborrowloc_no_local() {
    let state1 = AbstractState::new();
    common::run_instruction(Bytecode::MutBorrowLoc(0), state1);
}

#[test]
#[should_panic]
fn bytecode_immborrowloc_no_local() {
    let state1 = AbstractState::new();
    common::run_instruction(Bytecode::ImmBorrowLoc(0), state1);
}

// TODO: Turn back on when borrow graph and references are allowed
// #[test]
// #[should_panic]
// fn bytecode_mutborrowloc_local_unavailable() {
//     let mut state1 = AbstractState::new();
//     state1.local_insert(
//         0,
//         AbstractValue::new_primitive(SignatureToken::U64),
//         BorrowState::Unavailable,
//     );
//     common::run_instruction(Bytecode::MutBorrowLoc(0), state1);
// }
//
// #[test]
// #[should_panic]
// fn bytecode_immborrowloc_local_unavailable() {
//     let mut state1 = AbstractState::new();
//     state1.local_insert(
//         0,
//         AbstractValue::new_primitive(SignatureToken::U64),
//         BorrowState::Unavailable,
//     );
//     common::run_instruction(Bytecode::ImmBorrowLoc(0), state1);
// }
