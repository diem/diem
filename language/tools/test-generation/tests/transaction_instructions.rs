// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate test_generation;
use test_generation::abstract_state::{AbstractState, AbstractValue};
use vm::file_format::{Bytecode, SignatureToken};

mod common;

#[test]
fn bytecode_gettxnsenderaddress() {
    let state1 = AbstractState::new();
    let (state2, _) = common::run_instruction(Bytecode::GetTxnSenderAddress, state1);
    assert_eq!(
        state2.stack_peek(0),
        Some(AbstractValue::new_primitive(SignatureToken::Address)),
        "stack type postcondition not met"
    );
}
