// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that
//! - a constant's type only refers to primitive types
//! - a constant's data serializes correctly for that type
use move_binary_format::{
    access::ModuleAccess,
    errors::{verification_error, Location, PartialVMResult, VMResult},
    file_format::{CompiledModule, CompiledScript, Constant, SignatureToken, TableIndex},
    IndexKind,
};
use move_core_types::vm_status::StatusCode;

pub fn verify_module(module: &CompiledModule) -> VMResult<()> {
    verify_module_impl(module).map_err(|e| e.finish(Location::Module(module.self_id())))
}

fn verify_module_impl(module: &CompiledModule) -> PartialVMResult<()> {
    for (idx, constant) in module.constant_pool().iter().enumerate() {
        verify_constant(idx, constant)?
    }
    Ok(())
}

pub fn verify_script(module: &CompiledScript) -> VMResult<()> {
    verify_script_impl(module).map_err(|e| e.finish(Location::Script))
}

fn verify_script_impl(script: &CompiledScript) -> PartialVMResult<()> {
    for (idx, constant) in script.constant_pool.iter().enumerate() {
        verify_constant(idx, constant)?
    }
    Ok(())
}

fn verify_constant(idx: usize, constant: &Constant) -> PartialVMResult<()> {
    verify_constant_type(idx, &constant.type_)?;
    verify_constant_data(idx, constant)
}

fn verify_constant_type(idx: usize, type_: &SignatureToken) -> PartialVMResult<()> {
    if type_.is_valid_for_constant() {
        Ok(())
    } else {
        Err(verification_error(
            StatusCode::INVALID_CONSTANT_TYPE,
            IndexKind::ConstantPool,
            idx as TableIndex,
        ))
    }
}

fn verify_constant_data(idx: usize, constant: &Constant) -> PartialVMResult<()> {
    match constant.deserialize_constant() {
        Some(_) => Ok(()),
        None => Err(verification_error(
            StatusCode::MALFORMED_CONSTANT_DATA,
            IndexKind::ConstantPool,
            idx as TableIndex,
        )),
    }
}
