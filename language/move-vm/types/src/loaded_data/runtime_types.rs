// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::vm_error::{StatusCode, VMStatus};
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use vm::errors::VMResult;

use crate::loaded_data::types::FatType;
use vm::file_format::{Kind, StructDefinitionIndex};

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StructType {
    pub fields: Vec<Type>,
    pub is_resource: bool,
    pub type_parameters: Vec<Kind>,
    pub name: Identifier,
    pub module: ModuleId,
    pub struct_def: StructDefinitionIndex,
}

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Type {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Vector(Box<Type>),
    Struct(usize),
    StructInstantiation(usize, Vec<Type>),
    Reference(Box<Type>),
    MutableReference(Box<Type>),
    TyParam(usize),
}

impl Type {
    pub fn subst(&self, ty_args: &[Type]) -> VMResult<Type> {
        let res = match self {
            Type::TyParam(idx) => match ty_args.get(*idx) {
                Some(ty) => ty.clone(),
                None => {
                    return Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!(
                            "type substitution failed: index out of bounds -- len {} got {}",
                            ty_args.len(),
                            idx
                        )));
                }
            },
            Type::Bool => Type::Bool,
            Type::U8 => Type::U8,
            Type::U64 => Type::U64,
            Type::U128 => Type::U128,
            Type::Address => Type::Address,
            Type::Vector(ty) => Type::Vector(Box::new(ty.subst(ty_args)?)),
            Type::Reference(ty) => Type::Reference(Box::new(ty.subst(ty_args)?)),
            Type::MutableReference(ty) => Type::MutableReference(Box::new(ty.subst(ty_args)?)),
            Type::Struct(def_idx) => Type::Struct(*def_idx),
            Type::StructInstantiation(def_idx, instantiation) => {
                let mut inst = vec![];
                for ty in instantiation {
                    inst.push(ty.subst(ty_args)?)
                }
                Type::StructInstantiation(*def_idx, inst)
            }
        };
        Ok(res)
    }
}

pub trait TypeConverter {
    fn type_to_fat_type(&self, type_: &Type) -> VMResult<FatType>;
}
