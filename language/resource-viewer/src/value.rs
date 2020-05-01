// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Error, Result as AResult};
use libra_types::account_address::AccountAddress;
use move_vm_types::loaded_data::types::{FatStructType, FatType};
use serde::{de::Error as DeError, Deserialize};
use std::{
    convert::TryFrom,
    fmt::{self, Debug},
};

#[derive(Debug)]
pub struct MoveStruct(Vec<MoveValue>);

#[derive(Debug)]
pub enum MoveValue {
    U8(u8),
    U64(u64),
    U128(u128),
    Bool(bool),
    Address(AccountAddress),
    Vector(Vec<MoveValue>),
    Struct(MoveStruct),
}

#[derive(Debug)]
pub struct MoveStructLayout(Vec<MoveTypeLayout>);

#[derive(Debug)]
pub enum MoveTypeLayout {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Vector(Box<MoveTypeLayout>),
    Struct(MoveStructLayout),
}

impl MoveValue {
    pub fn simple_deserialize(blob: &[u8], ty: &MoveTypeLayout) -> AResult<Self> {
        Ok(lcs::from_bytes_seed(ty, blob)?)
    }
}
impl MoveStruct {
    pub fn new(value: Vec<MoveValue>) -> Self {
        MoveStruct(value)
    }

    pub fn simple_deserialize(blob: &[u8], ty: &MoveStructLayout) -> AResult<Self> {
        Ok(lcs::from_bytes_seed(ty, blob)?)
    }

    pub fn fields(&self) -> &[MoveValue] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<MoveValue> {
        self.0
    }
}

impl MoveStructLayout {
    pub fn new(types: Vec<MoveTypeLayout>) -> Self {
        MoveStructLayout(types)
    }
    pub fn fields(&self) -> &[MoveTypeLayout] {
        &self.0
    }
}

impl<'d> serde::de::DeserializeSeed<'d> for &MoveTypeLayout {
    type Value = MoveValue;

    fn deserialize<D: serde::de::Deserializer<'d>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        match self {
            MoveTypeLayout::Bool => bool::deserialize(deserializer).map(MoveValue::Bool),
            MoveTypeLayout::U8 => u8::deserialize(deserializer).map(MoveValue::U8),
            MoveTypeLayout::U64 => u64::deserialize(deserializer).map(MoveValue::U64),
            MoveTypeLayout::U128 => u128::deserialize(deserializer).map(MoveValue::U128),
            MoveTypeLayout::Address => {
                AccountAddress::deserialize(deserializer).map(MoveValue::Address)
            }

            MoveTypeLayout::Struct(ty) => Ok(MoveValue::Struct(ty.deserialize(deserializer)?)),

            MoveTypeLayout::Vector(layout) => Ok(MoveValue::Vector(
                deserializer.deserialize_seq(VectorElementVisitor(layout))?,
            )),
        }
    }
}

struct VectorElementVisitor<'a>(&'a MoveTypeLayout);

impl<'d, 'a> serde::de::Visitor<'d> for VectorElementVisitor<'a> {
    type Value = Vec<MoveValue>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Vector")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'d>,
    {
        let mut vals = Vec::new();
        while let Some(elem) = seq.next_element_seed(self.0)? {
            vals.push(elem)
        }
        Ok(vals)
    }
}

struct StructFieldVisitor<'a>(&'a [MoveTypeLayout]);

impl<'d, 'a> serde::de::Visitor<'d> for StructFieldVisitor<'a> {
    type Value = Vec<MoveValue>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Struct")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'d>,
    {
        let mut val = Vec::new();
        for (i, field_type) in self.0.iter().enumerate() {
            if let Some(elem) = seq.next_element_seed(field_type)? {
                val.push(elem)
            } else {
                return Err(A::Error::invalid_length(i, &self));
            }
        }
        Ok(val)
    }
}

impl<'d> serde::de::DeserializeSeed<'d> for &MoveStructLayout {
    type Value = MoveStruct;

    fn deserialize<D: serde::de::Deserializer<'d>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        let layout = &self.0;
        let fields = deserializer.deserialize_tuple(layout.len(), StructFieldVisitor(layout))?;
        Ok(MoveStruct(fields))
    }
}

impl TryFrom<&FatStructType> for MoveStructLayout {
    type Error = Error;

    fn try_from(ty: &FatStructType) -> Result<Self, Self::Error> {
        Ok(MoveStructLayout(
            ty.layout
                .iter()
                .map(MoveTypeLayout::try_from)
                .collect::<AResult<Vec<_>>>()?,
        ))
    }
}

impl TryFrom<&FatType> for MoveTypeLayout {
    type Error = Error;

    fn try_from(ty: &FatType) -> Result<Self, Self::Error> {
        Ok(match ty {
            FatType::Address => MoveTypeLayout::Address,
            FatType::U8 => MoveTypeLayout::U8,
            FatType::U64 => MoveTypeLayout::U64,
            FatType::U128 => MoveTypeLayout::U128,
            FatType::Bool => MoveTypeLayout::Bool,
            FatType::Vector(v) => {
                MoveTypeLayout::Vector(Box::new(MoveTypeLayout::try_from(v.as_ref())?))
            }
            FatType::Struct(s) => MoveTypeLayout::Struct(MoveStructLayout(
                s.layout
                    .iter()
                    .map(MoveTypeLayout::try_from)
                    .collect::<AResult<Vec<_>>>()?,
            )),
            _ => return Err(anyhow!("Unexpected type: {:?}", ty)),
        })
    }
}
