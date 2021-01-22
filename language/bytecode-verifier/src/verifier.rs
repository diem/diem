// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the public APIs supported by the bytecode verifier.
use crate::{
    ability_field_requirements, check_duplication::DuplicationChecker,
    code_unit_verifier::CodeUnitVerifier, constants, instantiation_loops::InstantiationLoopChecker,
    instruction_consistency::InstructionConsistency, script_signature, signature::SignatureChecker,
    struct_defs::RecursiveStructDefChecker,
};
use vm::{
    errors::VMResult,
    file_format::{CompiledModule, CompiledScript},
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
    ability_field_requirements::verify_module(&module)?;
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
    script_signature::verify_script(&script)
}
