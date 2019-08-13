extern crate test_generation;
use test_generation::abstract_state::AbstractState;
use vm::file_format::{Bytecode, SignatureToken};

mod common;

#[test]
fn bytecode_gettxngasunitprice() {
    let state1 = AbstractState::new(&Vec::new());
    let state2 = common::run_instruction(Bytecode::GetTxnGasUnitPrice, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_gettxnmaxgasunits() {
    let state1 = AbstractState::new(&Vec::new());
    let state2 = common::run_instruction(Bytecode::GetTxnMaxGasUnits, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_gettxnsequencenumber() {
    let state1 = AbstractState::new(&Vec::new());
    let state2 = common::run_instruction(Bytecode::GetTxnSequenceNumber, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}

#[test]
fn bytecode_getgasremaining() {
    let state1 = AbstractState::new(&Vec::new());
    let state2 = common::run_instruction(Bytecode::GetGasRemaining, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(SignatureToken::U64),
        "stack type postcondition not met"
    );
}
