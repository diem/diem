// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::CodeUnitVerifier;
use libra_types::vm_error::StatusCode;
use vm::file_format::{self, Bytecode};

#[test]
fn invalid_fallthrough_br_true() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::LdFalse, Bytecode::BrTrue(1)]);
    let result = CodeUnitVerifier::verify(&module);
    assert_eq!(
        result.unwrap_err().major_status,
        StatusCode::INVALID_FALL_THROUGH
    );
}

#[test]
fn invalid_fallthrough_br_false() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::LdTrue, Bytecode::BrFalse(1)]);
    let result = CodeUnitVerifier::verify(&module);
    assert_eq!(
        result.unwrap_err().major_status,
        StatusCode::INVALID_FALL_THROUGH
    );
}

// all non-branch instructions should trigger invalid fallthrough; just check one of them
#[test]
fn invalid_fallthrough_non_branch() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::LdTrue, Bytecode::Pop]);
    let result = CodeUnitVerifier::verify(&module);
    assert_eq!(
        result.unwrap_err().major_status,
        StatusCode::INVALID_FALL_THROUGH
    );
}

#[test]
fn valid_fallthrough_branch() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::Branch(0)]);
    let result = CodeUnitVerifier::verify(&module);
    assert!(result.is_ok());
}

#[test]
fn valid_fallthrough_ret() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::Ret]);
    let result = CodeUnitVerifier::verify(&module);
    assert!(result.is_ok());
}

#[test]
fn valid_fallthrough_abort() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::LdU64(7), Bytecode::Abort]);
    let result = CodeUnitVerifier::verify(&module);
    assert!(result.is_ok());
}
