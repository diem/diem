// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that
//! - a constant's type only refers to primitive types
//! - a constant's data serializes correctly for that type
use libra_types::vm_status::StatusCode;
use vm::{
    access::ModuleAccess,
    errors::{verification_error, VMResult},
    file_format::{CompiledModule, CompiledScript, Constant, SignatureToken},
    IndexKind,
};

pub fn verify_module(module: &CompiledModule) -> VMResult<()> {
    for (idx, constant) in module.constant_pool().iter().enumerate() {
        verify_constant(idx, constant)?
    }
    Ok(())
}

pub fn verify_script(script: &CompiledScript) -> VMResult<()> {
    for (idx, constant) in script.as_inner().constant_pool.iter().enumerate() {
        verify_constant(idx, constant)?
    }
    Ok(())
}

fn verify_constant(idx: usize, constant: &Constant) -> VMResult<()> {
    verify_constant_type(idx, &constant.type_)?;
    verify_constant_data(idx, constant)
}

fn verify_constant_type(idx: usize, type_: &SignatureToken) -> VMResult<()> {
    use SignatureToken as S;
    match type_ {
        S::Bool | S::U8 | S::U64 | S::U128 | S::Address => Ok(()),
        S::Vector(inner) => verify_constant_type(idx, inner),
        S::Signer
        | S::Struct(_)
        | S::StructInstantiation(_, _)
        | S::Reference(_)
        | S::MutableReference(_)
        | S::TypeParameter(_) => Err(verification_error(
            IndexKind::ConstantPool,
            idx,
            StatusCode::INVALID_CONSTANT_TYPE,
        )),
    }
}

fn verify_constant_data(idx: usize, constant: &Constant) -> VMResult<()> {
    match constant.deserialize_constant() {
        Some(_) => Ok(()),
        None => Err(verification_error(
            IndexKind::ConstantPool,
            idx,
            StatusCode::MALFORMED_CONSTANT_DATA,
        )),
    }
}
