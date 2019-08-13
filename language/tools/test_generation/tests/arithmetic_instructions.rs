extern crate test_generation;
use test_generation::abstract_state::AbstractState;
use vm::file_format::{Bytecode, SignatureToken};

mod common;

#[test]
fn bytecode_add() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::Add, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_sub() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::Sub, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_mul() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::Mul, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_div() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::Div, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_mod() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::Mod, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}
