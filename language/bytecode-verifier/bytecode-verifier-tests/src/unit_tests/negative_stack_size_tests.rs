// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::CodeUnitVerifier;
use libra_types::vm_error::StatusCode;
use vm::file_format::{self, Bytecode};

#[test]
fn one_pop_no_push() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::Pop, Bytecode::Ret]);
    let result = CodeUnitVerifier::verify(&module);
    assert_eq!(
        result.unwrap_err().major_status,
        StatusCode::NEGATIVE_STACK_SIZE_WITHIN_BLOCK
    );
}

#[test]
fn one_pop_one_push() {
    // Height: 0 + (-1 + 1) = 0 would have passed original usage verifier
    let module = file_format::dummy_procedure_module(vec![Bytecode::ReadRef, Bytecode::Ret]);
    let result = CodeUnitVerifier::verify(&module);
    assert_eq!(
        result.unwrap_err().major_status,
        StatusCode::NEGATIVE_STACK_SIZE_WITHIN_BLOCK
    );
}

#[test]
fn two_pop_one_push() {
    // Height: 0 + 1 + (-2 + 1) = 0 would have passed original usage verifier
    let module =
        file_format::dummy_procedure_module(vec![Bytecode::LdU64(0), Bytecode::Add, Bytecode::Ret]);
    let result = CodeUnitVerifier::verify(&module);
    assert_eq!(
        result.unwrap_err().major_status,
        StatusCode::NEGATIVE_STACK_SIZE_WITHIN_BLOCK
    );
}

#[test]
fn two_pop_no_push() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::WriteRef, Bytecode::Ret]);
    let result = CodeUnitVerifier::verify(&module);
    assert_eq!(
        result.unwrap_err().major_status,
        StatusCode::NEGATIVE_STACK_SIZE_WITHIN_BLOCK
    );
}
