extern crate test_generation;
use test_generation::abstract_state::{AbstractState, BorrowState};
use vm::file_format::{Bytecode, SignatureToken};

mod common;

#[test]
fn bytecode_copyloc() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.insert_local(0, SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::CopyLoc(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
    assert_eq!(
        state2.get_local(0),
        Some(&(SignatureToken::U64, BorrowState::Available)),
        "locals signature postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_copyloc_no_local() {
    let state1 = AbstractState::new(&Vec::new());
    common::run_instruction(Bytecode::CopyLoc(0), state1);
}

#[test]
#[should_panic]
fn bytecode_copyloc_local_unavailable() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.insert_local(0, SignatureToken::U64);
    state1.move_local(0);
    common::run_instruction(Bytecode::CopyLoc(0), state1);
}

#[test]
fn bytecode_moveloc() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.insert_local(0, SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::MoveLoc(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
    assert_eq!(
        state2.get_local(0),
        Some(&(SignatureToken::U64, BorrowState::Unavailable)),
        "locals signature postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_moveloc_no_local() {
    let state1 = AbstractState::new(&Vec::new());
    common::run_instruction(Bytecode::MoveLoc(0), state1);
}

#[test]
#[should_panic]
fn bytecode_moveloc_local_unavailable() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.insert_local(0, SignatureToken::U64);
    state1.move_local(0);
    common::run_instruction(Bytecode::MoveLoc(0), state1);
}

#[test]
fn bytecode_borrowloc() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.insert_local(0, SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::BorrowLoc(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::MutableReference(Box::new(
            SignatureToken::U64
        ))),
        "stack type postcondition not met"
    );
    assert_eq!(
        state2.get_local(0),
        Some(&(SignatureToken::U64, BorrowState::Available)),
        "locals signature postcondition not met"
    );
}

#[test]
#[should_panic]
fn bytecode_borrowloc_no_local() {
    let state1 = AbstractState::new(&Vec::new());
    common::run_instruction(Bytecode::BorrowLoc(0), state1);
}

#[test]
#[should_panic]
fn bytecode_borrowloc_local_unavailable() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.insert_local(0, SignatureToken::U64);
    state1.move_local(0);
    common::run_instruction(Bytecode::BorrowLoc(0), state1);
}
