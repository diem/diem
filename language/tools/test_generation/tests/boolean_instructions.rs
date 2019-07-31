extern crate test_generation;
use test_generation::abstract_state::AbstractState;
use vm::file_format::{Bytecode, SignatureToken};

mod common;

#[test]
fn bytecode_and() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::Bool);
    state1.stack_push(SignatureToken::Bool);
    let state2 = common::run_instruction(Bytecode::And, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_or() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::Bool);
    state1.stack_push(SignatureToken::Bool);
    let state2 = common::run_instruction(Bytecode::Or, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_not() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::Bool);
    state1.stack_push(SignatureToken::Bool);
    let state2 = common::run_instruction(Bytecode::Not, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}
