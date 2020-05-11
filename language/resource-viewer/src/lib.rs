// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cached_access_path_table::resource_vec_to_type_tag,
    resolver::Resolver,
    value::{MoveStruct, MoveStructLayout, MoveTypeLayout, MoveValue},
};
use anyhow::{anyhow, Result};
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
    contract_event::ContractEvent, language_storage::StructTag,
};
use move_core_types::identifier::Identifier;
use std::{
    collections::btree_map::BTreeMap,
    convert::TryFrom,
    fmt::{Display, Formatter},
};

pub use cached_access_path_table::update_mapping;
use move_vm_types::loaded_data::types::{FatStructType, FatType};

mod cached_access_path_table;
mod module_cache;
mod resolver;
pub mod value;

#[derive(Debug)]
pub struct AnnotatedAccountStateBlob(BTreeMap<StructTag, AnnotatedMoveStruct>);

#[derive(Debug)]
pub struct AnnotatedMoveStruct {
    is_resource: bool,
    type_: StructTag,
    value: Vec<(Identifier, AnnotatedMoveValue)>,
}

/// AnnotatedMoveValue is a fully expanded version of on chain move data. This should only be used
/// for debugging/client purpose right now and just for a better visualization of on chain data. In
/// the long run, we would like to transform this struct to a Json value so that we can have a cross
/// platform interpretation of the on chain data.
#[derive(Debug)]
pub enum AnnotatedMoveValue {
    U8(u8),
    U64(u64),
    U128(u128),
    Bool(bool),
    Address(AccountAddress),
    Vector(Vec<AnnotatedMoveValue>),
    Bytes(Vec<u8>),
    Struct(AnnotatedMoveStruct),
}

pub struct MoveValueAnnotator<'a> {
    cache: Resolver<'a>,
    _data_view: &'a dyn StateView,
}

impl<'a> MoveValueAnnotator<'a> {
    pub fn new(view: &'a dyn StateView) -> Self {
        Self {
            cache: Resolver::new(view, true),
            _data_view: view,
        }
    }

    pub fn view_access_path(
        &self,
        access_path: AccessPath,
        blob: &[u8],
    ) -> Result<AnnotatedMoveStruct> {
        let ty = resource_vec_to_type_tag(&access_path.path)?;
        let struct_def = MoveStructLayout::try_from(&ty)?;
        let move_struct = MoveStruct::simple_deserialize(blob, &struct_def)?;
        self.annotate_struct(&move_struct, &ty)
    }

    pub fn view_contract_event(&self, event: &ContractEvent) -> Result<AnnotatedMoveValue> {
        let ty = self.cache.resolve_type(event.type_tag())?;
        let move_ty = MoveTypeLayout::try_from(&ty)?;
        let move_value = MoveValue::simple_deserialize(event.event_data(), &move_ty)?;
        self.annotate_value(&move_value, &ty)
    }

    pub fn view_account_state(&self, state: &AccountState) -> Result<AnnotatedAccountStateBlob> {
        let mut output = BTreeMap::new();
        for (k, v) in state.iter() {
            let ty = resource_vec_to_type_tag(k.as_slice())?;
            let struct_def = MoveStructLayout::try_from(&ty)?;
            let move_struct = MoveStruct::simple_deserialize(v.as_slice(), &struct_def)?;
            output.insert(ty.struct_tag()?, self.annotate_struct(&move_struct, &ty)?);
        }
        Ok(AnnotatedAccountStateBlob(output))
    }

    fn annotate_struct(
        &self,
        move_struct: &MoveStruct,
        ty: &FatStructType,
    ) -> Result<AnnotatedMoveStruct> {
        let struct_tag = ty.struct_tag()?;
        let field_names = self.cache.get_field_names(ty)?;
        let mut annotated_fields = vec![];
        for (ty, v) in ty.layout.iter().zip(move_struct.fields().iter()) {
            annotated_fields.push(self.annotate_value(v, ty)?);
        }
        Ok(AnnotatedMoveStruct {
            is_resource: ty.is_resource,
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
        AnnotatedMoveValue::Address(a) => write!(f, "{}", a.short_str()),
        AnnotatedMoveValue::Vector(v) => {
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
    writeln!(
        f,
        "{}{} {{",
        if value.is_resource { "resource " } else { "" },
        value.type_
    )?;
    for (field_name, v) in value.value.iter() {
        write_indent(f, indent + 4)?;
        write!(f, "{}: ", field_name)?;
        pretty_print_value(f, v, indent + 4)?;
        writeln!(f)?;
    }
    write_indent(f, indent)?;
    write!(f, "}}")
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

impl Display for AnnotatedAccountStateBlob {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for v in self.0.values() {
            write!(f, "{}", v)?;
            writeln!(f, ",")?;
        }
        writeln!(f, "}}")
    }
}

#[derive(Default)]
pub struct NullStateView();

impl StateView for NullStateView {
    fn get(&self, _access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Err(anyhow!("No data"))
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        Err(anyhow!("No data"))
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
