// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    fat_type::{FatStructType, FatType},
    resolver::Resolver,
};
use anyhow::{anyhow, Result};
use move_binary_format::{
    errors::{Location, PartialVMError},
    file_format::{Ability, AbilitySet},
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag, TypeTag},
    resolver::MoveResolver,
    value::{MoveStruct, MoveValue},
    vm_status::VMStatus,
};
use serde::ser::{SerializeMap, SerializeSeq};
use std::{
    convert::{TryFrom, TryInto},
    fmt::{Display, Formatter},
};

mod fat_type;
mod module_cache;
mod resolver;

#[derive(Clone, Debug)]
pub struct AnnotatedMoveStruct {
    pub abilities: AbilitySet,
    pub type_: StructTag,
    pub value: Vec<(Identifier, AnnotatedMoveValue)>,
}

/// AnnotatedMoveValue is a fully expanded version of on chain Move data. This should only be used
/// for debugging/client purpose right now and just for a better visualization of on chain data. In
/// the long run, we would like to transform this struct to a Json value so that we can have a cross
/// platform interpretation of the on chain data.
#[derive(Clone, Debug)]
pub enum AnnotatedMoveValue {
    U8(u8),
    U64(u64),
    U128(u128),
    Bool(bool),
    Address(AccountAddress),
    Vector(TypeTag, Vec<AnnotatedMoveValue>),
    Bytes(Vec<u8>),
    Struct(AnnotatedMoveStruct),
}

impl AnnotatedMoveValue {
    pub fn get_type(&self) -> TypeTag {
        use AnnotatedMoveValue::*;
        match self {
            U8(_) => TypeTag::U8,
            U64(_) => TypeTag::U64,
            U128(_) => TypeTag::U128,
            Bool(_) => TypeTag::Bool,
            Address(_) => TypeTag::Address,
            Vector(t, _) => t.clone(),
            Bytes(_) => TypeTag::Vector(Box::new(TypeTag::U8)),
            Struct(s) => TypeTag::Struct(s.type_.clone()),
        }
    }
}

pub struct MoveValueAnnotator<'a, T: ?Sized> {
    cache: Resolver<'a, T>,
}

impl<'a, T: MoveResolver + ?Sized> MoveValueAnnotator<'a, T> {
    pub fn new(view: &'a T) -> Self {
        Self {
            cache: Resolver::new(view),
        }
    }

    pub fn get_resource_bytes(&self, addr: &AccountAddress, tag: &StructTag) -> Option<Vec<u8>> {
        self.cache.state.get_resource(addr, tag).ok()?
    }

    pub fn get_module_bytes(&self, module: &ModuleId) -> Option<Vec<u8>> {
        self.cache.state.get_module(module).ok()?
    }

    pub fn view_function_arguments(
        &self,
        module: &ModuleId,
        function: &IdentStr,
        args: &[Vec<u8>],
    ) -> Result<Vec<AnnotatedMoveValue>> {
        let types: Vec<FatType> = self
            .cache
            .resolve_function_arguments(module, function)?
            .into_iter()
            .filter(|t| !matches!(t, FatType::Signer))
            .collect();
        anyhow::ensure!(
            types.len() == args.len(),
            "unexpected error: argument types({}) and values({}) are not matched",
            types.len(),
            args.len(),
        );
        types
            .iter()
            .enumerate()
            .map(|(i, ty)| self.view_value_by_fat_type(ty, &args[i]))
            .collect::<Result<_>>()
    }

    pub fn view_resource(&self, tag: &StructTag, blob: &[u8]) -> Result<AnnotatedMoveStruct> {
        let ty = self.cache.resolve_struct(tag)?;
        let struct_def = (&ty).try_into().map_err(into_vm_status)?;
        let move_struct = MoveStruct::simple_deserialize(blob, &struct_def)?;
        self.annotate_struct(&move_struct, &ty)
    }

    pub fn view_value(&self, ty_tag: &TypeTag, blob: &[u8]) -> Result<AnnotatedMoveValue> {
        let ty = self.cache.resolve_type(ty_tag)?;
        self.view_value_by_fat_type(&ty, blob)
    }

    fn view_value_by_fat_type(&self, ty: &FatType, blob: &[u8]) -> Result<AnnotatedMoveValue> {
        let layout = ty.try_into().map_err(into_vm_status)?;
        let move_value = MoveValue::simple_deserialize(blob, &layout)?;
        self.annotate_value(&move_value, ty)
    }

    fn annotate_struct(
        &self,
        move_struct: &MoveStruct,
        ty: &FatStructType,
    ) -> Result<AnnotatedMoveStruct> {
        let struct_tag = ty
            .struct_tag()
            .map_err(|e| e.finish(Location::Undefined).into_vm_status())?;
        let field_names = self.cache.get_field_names(ty)?;
        let mut annotated_fields = vec![];
        for (ty, v) in ty.layout.iter().zip(move_struct.fields().iter()) {
            annotated_fields.push(self.annotate_value(v, ty)?);
        }
        Ok(AnnotatedMoveStruct {
            abilities: ty.abilities.0,
            type_: struct_tag,
            value: field_names
                .into_iter()
                .zip(annotated_fields.into_iter())
                .collect(),
        })
    }

    fn annotate_value(&self, value: &MoveValue, ty: &FatType) -> Result<AnnotatedMoveValue> {
        Ok(match (value, ty) {
            (MoveValue::Bool(b), FatType::Bool) => AnnotatedMoveValue::Bool(*b),
            (MoveValue::U8(i), FatType::U8) => AnnotatedMoveValue::U8(*i),
            (MoveValue::U64(i), FatType::U64) => AnnotatedMoveValue::U64(*i),
            (MoveValue::U128(i), FatType::U128) => AnnotatedMoveValue::U128(*i),
            (MoveValue::Address(a), FatType::Address) => AnnotatedMoveValue::Address(*a),
            (MoveValue::Vector(a), FatType::Vector(ty)) => match ty.as_ref() {
                FatType::U8 => AnnotatedMoveValue::Bytes(
                    a.iter()
                        .map(|v| match v {
                            MoveValue::U8(i) => Ok(*i),
                            _ => Err(anyhow!("unexpected value type")),
                        })
                        .collect::<Result<_>>()?,
                ),
                _ => AnnotatedMoveValue::Vector(
                    ty.type_tag().unwrap(),
                    a.iter()
                        .map(|v| self.annotate_value(v, ty.as_ref()))
                        .collect::<Result<_>>()?,
                ),
            },
            (MoveValue::Struct(s), FatType::Struct(ty)) => {
                AnnotatedMoveValue::Struct(self.annotate_struct(s, ty.as_ref())?)
            }
            _ => {
                return Err(anyhow!(
                    "Cannot annotate value {:?} with type {:?}",
                    value,
                    ty
                ))
            }
        })
    }
}

fn into_vm_status(e: PartialVMError) -> VMStatus {
    e.finish(Location::Undefined).into_vm_status()
}

fn write_indent(f: &mut Formatter, indent: u64) -> std::fmt::Result {
    for _i in 0..indent {
        write!(f, " ")?;
    }
    Ok(())
}

fn pretty_print_value(
    f: &mut Formatter,
    value: &AnnotatedMoveValue,
    indent: u64,
) -> std::fmt::Result {
    match value {
        AnnotatedMoveValue::Bool(b) => write!(f, "{}", b),
        AnnotatedMoveValue::U8(v) => write!(f, "{}u8", v),
        AnnotatedMoveValue::U64(v) => write!(f, "{}", v),
        AnnotatedMoveValue::U128(v) => write!(f, "{}u128", v),
        AnnotatedMoveValue::Address(a) => write!(f, "{}", a.short_str_lossless()),
        AnnotatedMoveValue::Vector(_, v) => {
            writeln!(f, "[")?;
            for value in v.iter() {
                write_indent(f, indent + 4)?;
                pretty_print_value(f, value, indent + 4)?;
                writeln!(f, ",")?;
            }
            write_indent(f, indent)?;
            write!(f, "]")
        }
        AnnotatedMoveValue::Bytes(v) => write!(f, "{}", hex::encode(&v)),
        AnnotatedMoveValue::Struct(s) => pretty_print_struct(f, s, indent),
    }
}

fn pretty_print_struct(
    f: &mut Formatter,
    value: &AnnotatedMoveStruct,
    indent: u64,
) -> std::fmt::Result {
    pretty_print_ability_modifiers(f, value.abilities)?;
    writeln!(f, "{} {{", value.type_)?;
    for (field_name, v) in value.value.iter() {
        write_indent(f, indent + 4)?;
        write!(f, "{}: ", field_name)?;
        pretty_print_value(f, v, indent + 4)?;
        writeln!(f)?;
    }
    write_indent(f, indent)?;
    write!(f, "}}")
}

fn pretty_print_ability_modifiers(f: &mut Formatter, abilities: AbilitySet) -> std::fmt::Result {
    for ability in abilities {
        match ability {
            Ability::Copy => write!(f, "copy ")?,
            Ability::Drop => write!(f, "drop ")?,
            Ability::Store => write!(f, "store ")?,
            Ability::Key => write!(f, "key ")?,
        }
    }
    Ok(())
}

impl serde::Serialize for AnnotatedMoveStruct {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_map(Some(self.value.len()))?;
        for (f, v) in &self.value {
            s.serialize_entry(f, v)?
        }
        s.end()
    }
}

impl serde::Serialize for AnnotatedMoveValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use AnnotatedMoveValue::*;
        match self {
            U8(n) => serializer.serialize_u8(*n),
            U64(n) => serializer.serialize_u64(*n),
            U128(n) => {
                // TODO: we could use serializer.serialize_u128 here, but it requires the serde_json
                // arbitrary_precision, which breaks some existing json-rpc test. figure
                // out what's going on or come up with a better workaround
                if let Ok(i) = u64::try_from(*n) {
                    serializer.serialize_u64(i)
                } else {
                    serializer.serialize_bytes(&n.to_le_bytes())
                }
            }
            Bool(b) => serializer.serialize_bool(*b),
            Address(a) => a.short_str_lossless().serialize(serializer),
            Vector(t, vals) => {
                assert_ne!(t, &TypeTag::U8);
                let mut vec = serializer.serialize_seq(Some(vals.len()))?;
                for v in vals {
                    vec.serialize_element(v)?;
                }
                vec.end()
            }
            Bytes(v) => {
                // try to deserialize as utf8, fall back to hex with if we can't
                let utf8_str = std::str::from_utf8(v);
                if let Ok(s) = utf8_str {
                    if s.chars().any(|c| c.is_ascii_control()) {
                        // has control characters; probably bytes
                        serializer.serialize_str(&hex::encode(v))
                    } else {
                        serializer.serialize_str(s)
                    }
                } else {
                    serializer.serialize_str(&hex::encode(v))
                }
            }
            Struct(s) => s.serialize(serializer),
        }
    }
}

impl Display for AnnotatedMoveValue {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        pretty_print_value(f, self, 0)
    }
}

impl Display for AnnotatedMoveStruct {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        pretty_print_struct(f, self, 0)
    }
}
