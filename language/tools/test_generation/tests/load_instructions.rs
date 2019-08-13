extern crate test_generation;
use test_generation::abstract_state::AbstractState;
use vm::file_format::{AddressPoolIndex, Bytecode, SignatureToken, StringPoolIndex};

mod common;

#[test]
fn bytecode_ldconst() {
    let state1 = AbstractState::new(&Vec::new());
    let state2 = common::run_instruction(Bytecode::LdConst(0), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_ldtrue() {
    let state1 = AbstractState::new(&Vec::new());
    let state2 = common::run_instruction(Bytecode::LdTrue, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_ldfalse() {
    let state1 = AbstractState::new(&Vec::new());
    let state2 = common::run_instruction(Bytecode::LdFalse, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Bool),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_ldstr() {
    let state1 = AbstractState::new(&Vec::new());
    let state2 = common::run_instruction(Bytecode::LdStr(StringPoolIndex::new(0)), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::String),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_ldaddr() {
    let state1 = AbstractState::new(&Vec::new());
    let state2 = common::run_instruction(Bytecode::LdAddr(AddressPoolIndex::new(0)), state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::Address),
        "stack type postcondition not met"
    );
}
