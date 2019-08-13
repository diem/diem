extern crate test_generation;
use test_generation::abstract_state::AbstractState;
use vm::file_format::{Bytecode, SignatureToken};

mod common;

#[test]
fn bytecode_gt() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::Gt, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_lt() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::Lt, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_ge() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::Ge, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_le() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::Le, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_eq_u64() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::Eq, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_eq_bool() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::Bool);
    state1.stack_push(SignatureToken::Bool);
    let state2 = common::run_instruction(Bytecode::Eq, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_neq_u64() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::U64);
    state1.stack_push(SignatureToken::U64);
    let state2 = common::run_instruction(Bytecode::Neq, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_neq_bool() {
    let mut state1 = AbstractState::new(&Vec::new());
    state1.stack_push(SignatureToken::Bool);
    state1.stack_push(SignatureToken::Bool);
    let state2 = common::run_instruction(Bytecode::Neq, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}
