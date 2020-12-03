// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for runtime types.

use diem_types::{account_address::AccountAddress, vm_status::StatusCode};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
    value::{MoveStructLayout, MoveTypeLayout},
};
use std::convert::TryInto;
use vm::errors::{PartialVMError, PartialVMResult};

use serde::{Deserialize, Serialize};

/// VM representation of a struct type in Move.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FatStructType {
    pub address: AccountAddress,
    pub module: Identifier,
    pub name: Identifier,
    pub is_resource: bool,
    pub ty_args: Vec<FatType>,
    pub layout: Vec<FatType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum FatType {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    Vector(Box<FatType>),
    Struct(Box<FatStructType>),
    Reference(Box<FatType>),
    MutableReference(Box<FatType>),
    TyParam(usize),
}

impl FatStructType {
    pub fn subst(&self, ty_args: &[FatType]) -> PartialVMResult<FatStructType> {
        Ok(Self {
            address: self.address,
            module: self.module.clone(),
            name: self.name.clone(),
            is_resource: self.is_resource,
            ty_args: self
                .ty_args
                .iter()
                .map(|ty| ty.subst(ty_args))
                .collect::<PartialVMResult<_>>()?,
            layout: self
                .layout
                .iter()
                .map(|ty| ty.subst(ty_args))
                .collect::<PartialVMResult<_>>()?,
        })
    }

    pub fn struct_tag(&self) -> PartialVMResult<StructTag> {
        let ty_args = self
            .ty_args
            .iter()
            .map(|ty| ty.type_tag())
            .collect::<PartialVMResult<Vec<_>>>()?;
        Ok(StructTag {
            address: self.address,
            module: self.module.clone(),
            name: self.name.clone(),
            type_params: ty_args,
        })
    }
}

impl FatType {
    pub fn subst(&self, ty_args: &[FatType]) -> PartialVMResult<FatType> {
        use FatType::*;

        let res = match self {
            TyParam(idx) => match ty_args.get(*idx) {
                Some(ty) => ty.clone(),
                None => {
                    return Err(
                        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                            .with_message(format!(
                            "fat type substitution failed: index out of bounds -- len {} got {}",
                            ty_args.len(),
                            idx
                        )),
                    );
                }
            },

            Bool => Bool,
            U8 => U8,
            U64 => U64,
            U128 => U128,
            Address => Address,
            Signer => Signer,
            Vector(ty) => Vector(Box::new(ty.subst(ty_args)?)),
            Reference(ty) => Reference(Box::new(ty.subst(ty_args)?)),
            MutableReference(ty) => MutableReference(Box::new(ty.subst(ty_args)?)),

            Struct(struct_ty) => Struct(Box::new(struct_ty.subst(ty_args)?)),
        };

        Ok(res)
    }

    pub fn type_tag(&self) -> PartialVMResult<TypeTag> {
        use FatType::*;

        let res = match self {
            Bool => TypeTag::Bool,
            U8 => TypeTag::U8,
            U64 => TypeTag::U64,
            U128 => TypeTag::U128,
            Address => TypeTag::Address,
            Signer => TypeTag::Signer,
            Vector(ty) => TypeTag::Vector(Box::new(ty.type_tag()?)),
            Struct(struct_ty) => TypeTag::Struct(struct_ty.struct_tag()?),

            Reference(_) | MutableReference(_) | TyParam(_) => {
                return Err(
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!("cannot derive type tag for {:?}", self)),
                )
            }
        };

        Ok(res)
    }
}

impl TryInto<MoveStructLayout> for &FatStructType {
    type Error = PartialVMError;

    fn try_into(self) -> Result<MoveStructLayout, Self::Error> {
        Ok(MoveStructLayout::new(
            self.layout
                .iter()
                .map(|ty| ty.try_into())
                .collect::<PartialVMResult<Vec<_>>>()?,
        ))
    }
}

impl TryInto<MoveTypeLayout> for &FatType {
    type Error = PartialVMError;

    fn try_into(self) -> Result<MoveTypeLayout, Self::Error> {
        Ok(match self {
            FatType::Address => MoveTypeLayout::Address,
            FatType::U8 => MoveTypeLayout::U8,
            FatType::U64 => MoveTypeLayout::U64,
            FatType::U128 => MoveTypeLayout::U128,
            FatType::Bool => MoveTypeLayout::Bool,
            FatType::Vector(v) => MoveTypeLayout::Vector(Box::new(v.as_ref().try_into()?)),
            FatType::Struct(s) => MoveTypeLayout::Struct(MoveStructLayout::new(
                s.layout
                    .iter()
                    .map(|ty| ty.try_into())
                    .collect::<PartialVMResult<Vec<_>>>()?,
            )),
            FatType::Signer => MoveTypeLayout::Signer,

            _ => return Err(PartialVMError::new(StatusCode::ABORT_TYPE_MISMATCH_ERROR)),
        })
    }
}
