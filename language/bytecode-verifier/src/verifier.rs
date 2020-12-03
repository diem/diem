// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the public APIs supported by the bytecode verifier.
use crate::{
    check_duplication::DuplicationChecker, code_unit_verifier::CodeUnitVerifier, constants,
    instantiation_loops::InstantiationLoopChecker, instruction_consistency::InstructionConsistency,
    resources::ResourceTransitiveChecker, signature::SignatureChecker,
    struct_defs::RecursiveStructDefChecker,
};
use diem_types::vm_status::StatusCode;
use vm::{
    access::ScriptAccess,
    errors::{Location, PartialVMError, PartialVMResult, VMResult},
    file_format::{CompiledModule, CompiledScript, SignatureToken},
};

/// Helper for a "canonical" verification of a module.
///
/// Clients that rely on verification should call the proper passes
/// internally rather than using this function.
/// This function is intended to provide a verification path for clients
/// that do not require full control over verification
pub fn verify_module(module: &CompiledModule) -> VMResult<()> {
    DuplicationChecker::verify_module(&module)?;
    SignatureChecker::verify_module(&module)?;
    InstructionConsistency::verify_module(&module)?;
    constants::verify_module(&module)?;
    ResourceTransitiveChecker::verify_module(&module)?;
    RecursiveStructDefChecker::verify_module(&module)?;
    InstantiationLoopChecker::verify_module(&module)?;
    CodeUnitVerifier::verify_module(&module)
}

/// Helper for a "canonical" verification of a script.
///
/// Clients that rely on verification should call the proper passes
/// internally rather than using this function.
/// This function is intended to provide a verification path for clients
/// that do not require full control over verification
pub fn verify_script(script: &CompiledScript) -> VMResult<()> {
    DuplicationChecker::verify_script(&script)?;
    SignatureChecker::verify_script(&script)?;
    InstructionConsistency::verify_script(&script)?;
    constants::verify_script(&script)?;
    CodeUnitVerifier::verify_script(&script)?;
    verify_main_signature(&script)
}

/// This function checks the extra requirements on the signature of the main function of a script.
pub fn verify_main_signature(script: &CompiledScript) -> VMResult<()> {
    verify_main_signature_impl(script).map_err(|e| e.finish(Location::Script))
}

fn verify_main_signature_impl(script: &CompiledScript) -> PartialVMResult<()> {
    use SignatureToken as S;
    let arguments = &script.signature_at(script.as_inner().parameters).0;
    // Check that all `&signer` arguments occur before non-`&signer` arguments
    let all_args_have_valid_type = arguments
        .iter()
        .skip_while(|typ| {
            match typ {
                // &signer is a type that can only be populated by the Move VM. And its value is filled
                // based on the sender of the transaction
                S::Reference(inner) => matches!(&**inner, S::Signer),
                _ => false,
            }
        })
        .all(|typ| typ.is_valid_for_constant());
    if !all_args_have_valid_type {
        Err(PartialVMError::new(
            StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
        ))
    } else {
        Ok(())
    }
}
