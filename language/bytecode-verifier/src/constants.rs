// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements a checker for verifying that
//! - a constant's type only refers to primitive types
//! - a constant's data serializes correctly for that type
use libra_types::vm_error::StatusCode;
use move_vm_types::values::Value;
use vm::{
    access::ModuleAccess,
    errors::{verification_error, VMResult},
    file_format::{CompiledModule, Constant, SignatureToken},
    IndexKind,
};

pub struct ConstantsChecker<'a> {
    module: &'a CompiledModule,
}

impl<'a> ConstantsChecker<'a> {
    pub fn new(module: &'a CompiledModule) -> Self {
        Self { module }
    }

    pub fn verify(self) -> VMResult<()> {
        for (idx, constant) in self.module.constant_pool().iter().enumerate() {
            self.verify_constant(idx, constant)?
        }
        Ok(())
    }

    pub fn verify_constant(&self, idx: usize, constant: &Constant) -> VMResult<()> {
        self.verify_constant_type(idx, &constant.type_)?;
        self.verify_constant_data(idx, constant)
    }

    pub fn verify_constant_type(&self, idx: usize, type_: &SignatureToken) -> VMResult<()> {
        use SignatureToken as S;
        match type_ {
            S::Bool | S::U8 | S::U64 | S::U128 | S::Address => Ok(()),
            S::Vector(inner) => self.verify_constant_type(idx, inner),
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

    pub fn verify_constant_data(&self, idx: usize, constant: &Constant) -> VMResult<()> {
        match Value::deserialize_constant(constant) {
            Some(_) => Ok(()),
            None => Err(verification_error(
                IndexKind::ConstantPool,
                idx,
                StatusCode::MALFORMED_CONSTANT_DATA,
            )),
        }
    }
}
