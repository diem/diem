// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use anyhow::Result as AResult;
use serde::{
    de::Error as DeError,
    ser::{SerializeSeq, SerializeTuple},
    Deserialize,
};
use std::fmt::{self, Debug};

#[derive(Debug, PartialEq, Eq)]
pub struct MoveStruct(Vec<MoveValue>);

#[derive(Debug, PartialEq, Eq)]
pub enum MoveValue {
    U8(u8),
    U64(u64),
    U128(u128),
    Bool(bool),
    Address(AccountAddress),
    Vector(Vec<MoveValue>),
    Struct(MoveStruct),
    Signer(AccountAddress),
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
    Signer,
}

impl MoveValue {
    pub fn simple_deserialize(blob: &[u8], ty: &MoveTypeLayout) -> AResult<Self> {
        Ok(lcs::from_bytes_seed(ty, blob)?)
    }

    pub fn simple_serialize(&self) -> Option<Vec<u8>> {
        lcs::to_bytes(self).ok()
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
            MoveTypeLayout::Signer => {
                AccountAddress::deserialize(deserializer).map(MoveValue::Signer)
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
            match seq.next_element_seed(field_type)? {
                Some(elem) => val.push(elem),
                None => return Err(A::Error::invalid_length(i, &self)),
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

impl serde::Serialize for MoveValue {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            MoveValue::Struct(s) => s.serialize(serializer),
            MoveValue::Bool(b) => serializer.serialize_bool(*b),
            MoveValue::U8(i) => serializer.serialize_u8(*i),
            MoveValue::U64(i) => serializer.serialize_u64(*i),
            MoveValue::U128(i) => serializer.serialize_u128(*i),
            MoveValue::Address(a) => a.serialize(serializer),
            MoveValue::Signer(a) => a.serialize(serializer),
            MoveValue::Vector(v) => {
                let mut t = serializer.serialize_seq(Some(v.len()))?;
                for val in v {
                    t.serialize_element(val)?;
                }
                t.end()
            }
        }
    }
}

impl serde::Serialize for MoveStruct {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut t = serializer.serialize_tuple(self.0.len())?;
        for v in self.0.iter() {
            t.serialize_element(v)?;
        }
        t.end()
    }
}
