extern crate test_generation;
use test_generation::abstract_state::{AbstractState, BorrowState};
use vm::file_format::{Bytecode, SignatureToken};

mod common;

#[test]
fn bytecode_copyloc() {
    let mut state1 = AbstractState::new();
    state1.local_insert(0, SignatureToken::U64, BorrowState::Available);
    let state2 = common::run_instruction(Bytecode::CopyLoc(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
    assert_eq!(
        state2.local_get(0),
        Some(&(SignatureToken::U64, BorrowState::Available)),
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
    state1.local_insert(0, SignatureToken::U64, BorrowState::Unavailable);
    common::run_instruction(Bytecode::CopyLoc(0), state1);
}

#[test]
fn bytecode_moveloc() {
    let mut state1 = AbstractState::new();
    state1.local_insert(0, SignatureToken::U64, BorrowState::Available);
    let state2 = common::run_instruction(Bytecode::MoveLoc(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
    assert_eq!(
        state2.local_get(0),
        Some(&(SignatureToken::U64, BorrowState::Unavailable)),
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
    state1.local_insert(0, SignatureToken::U64, BorrowState::Unavailable);
    common::run_instruction(Bytecode::MoveLoc(0), state1);
}

#[test]
fn bytecode_borrowloc() {
    let mut state1 = AbstractState::new();
    state1.local_insert(0, SignatureToken::U64, BorrowState::Available);
    let state2 = common::run_instruction(Bytecode::MutBorrowLoc(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::MutableReference(Box::new(
            SignatureToken::U64
        ))),
        "stack type postcondition not met"
    );
    assert_eq!(
        state2.local_get(0),
        Some(&(SignatureToken::U64, BorrowState::Available)),
        "locals signature postcondition not met"
    );
}

#[test]
fn bytecode_imm_borrowloc() {
    let mut state1 = AbstractState::new();
    state1.local_insert(0, SignatureToken::U64, BorrowState::Available);
    let state2 = common::run_instruction(Bytecode::ImmBorrowLoc(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Reference(Box::new(SignatureToken::U64))),
        "stack type postcondition not met"
    );
    assert_eq!(
        state2.local_get(0),
        Some(&(SignatureToken::U64, BorrowState::Available)),
        "locals signature postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_borrowloc_no_local() {
    let state1 = AbstractState::new();
    common::run_instruction(Bytecode::MutBorrowLoc(0), state1);
}

#[test]
#[should_panic]
fn bytecode_imm_borrowloc_no_local() {
    let state1 = AbstractState::new();
    common::run_instruction(Bytecode::ImmBorrowLoc(0), state1);
}

#[test]
#[should_panic]
fn bytecode_borrowloc_local_unavailable() {
    let mut state1 = AbstractState::new();
    state1.local_insert(0, SignatureToken::U64, BorrowState::Unavailable);
    common::run_instruction(Bytecode::MutBorrowLoc(0), state1);
}

#[test]
#[should_panic]
fn bytecode_imm_borrowloc_local_unavailable() {
    let mut state1 = AbstractState::new();
    state1.local_insert(0, SignatureToken::U64, BorrowState::Unavailable);
    common::run_instruction(Bytecode::ImmBorrowLoc(0), state1);
}
