extern crate test_generation;
use test_generation::abstract_state::AbstractState;
use vm::file_format::{Bytecode, SignatureToken};

mod common;

#[test]
fn bytecode_bitand() {
    let mut state1 = AbstractState::new();
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::BitAnd, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_bitor() {
    let mut state1 = AbstractState::new();
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::BitAnd, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_xor() {
    let mut state1 = AbstractState::new();
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::Xor, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}
