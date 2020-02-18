// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::control_flow;
use libra_types::vm_error::StatusCode;
use stdlib::{stdlib_modules, StdLibOptions};
use vm::{
    access::ModuleAccess,
    errors::VMResult,
    file_format::{self, Bytecode, CompiledModule},
};

fn verify_module(module: &CompiledModule) -> VMResult<()> {
    for function_definition in module.function_defs().iter().filter(|def| !def.is_native()) {
        control_flow::verify(&module, &function_definition)?
    }
    Ok(())
}

//**************************************************************************************************
// Simple cases -  Copied from code unit verifier
//**************************************************************************************************

#[test]
fn invalid_fallthrough_br_true() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::LdFalse, Bytecode::BrTrue(1)]);
    let result = verify_module(&module);
    assert_eq!(
        result.unwrap_err().major_status,
        StatusCode::INVALID_FALL_THROUGH
    );
}

#[test]
fn invalid_fallthrough_br_false() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::LdTrue, Bytecode::BrFalse(1)]);
    let result = verify_module(&module);
    assert_eq!(
        result.unwrap_err().major_status,
        StatusCode::INVALID_FALL_THROUGH
    );
}

// all non-branch instructions should trigger invalid fallthrough; just check one of them
#[test]
fn invalid_fallthrough_non_branch() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::LdTrue, Bytecode::Pop]);
    let result = verify_module(&module);
    assert_eq!(
        result.unwrap_err().major_status,
        StatusCode::INVALID_FALL_THROUGH
    );
}

#[test]
fn valid_fallthrough_branch() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::Branch(0)]);
    let result = verify_module(&module);
    assert!(result.is_ok());
}

#[test]
fn valid_fallthrough_ret() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::Ret]);
    let result = verify_module(&module);
    assert!(result.is_ok());
}

#[test]
fn valid_fallthrough_abort() {
    let module = file_format::dummy_procedure_module(vec![Bytecode::LdU64(7), Bytecode::Abort]);
    let result = verify_module(&module);
    assert!(result.is_ok());
}

//**************************************************************************************************
// Std Lib
//**************************************************************************************************

#[test]
fn stdlib() {
    for module in stdlib_modules(StdLibOptions::Staged) {
        verify_module(module.as_inner()).unwrap()
    }
}
