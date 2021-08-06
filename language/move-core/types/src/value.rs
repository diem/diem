// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_address::AccountAddress, identifier::Identifier};
use anyhow::Result as AResult;
use serde::{
    de::Error as DeError,
    ser::{SerializeMap, SerializeSeq, SerializeTuple},
    Deserialize, Serialize,
};
use std::fmt::{self, Debug};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MoveStruct {
    /// The representation used by the MoveVM
    Runtime(Vec<MoveValue>),
    /// A decorated representation with human-readable field names that can be used by clients
    WithFields(Vec<(Identifier, MoveValue)>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

/// A layout associated with a named field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveFieldLayout {
    name: Identifier,
    layout: MoveTypeLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoveStructLayout {
    /// The representation used by the MoveVM
    Runtime(Vec<MoveTypeLayout>),
    /// A decorated representation with human-readable field names that can be used by clients
    WithFields(Vec<MoveFieldLayout>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        Ok(bcs::from_bytes_seed(ty, blob)?)
    }

    pub fn simple_serialize(&self) -> Option<Vec<u8>> {
        bcs::to_bytes(self).ok()
    }

    pub fn vector_u8(v: Vec<u8>) -> Self {
        MoveValue::Vector(v.into_iter().map(MoveValue::U8).collect())
    }

    pub fn vector_address(v: Vec<AccountAddress>) -> Self {
        MoveValue::Vector(v.into_iter().map(MoveValue::Address).collect())
    }
}

pub fn serialize_values<'a, I>(vals: I) -> Vec<Vec<u8>>
where
    I: IntoIterator<Item = &'a MoveValue>,
{
    vals.into_iter()
        .map(|val| {
            val.simple_serialize()
                .expect("serialization should succeed")
        })
        .collect()
}

impl MoveStruct {
    pub fn new(value: Vec<MoveValue>) -> Self {
        Self::Runtime(value)
    }

    pub fn with_fields(values: Vec<(Identifier, MoveValue)>) -> Self {
        Self::WithFields(values)
    }

    pub fn simple_deserialize(blob: &[u8], ty: &MoveStructLayout) -> AResult<Self> {
        Ok(bcs::from_bytes_seed(ty, blob)?)
    }

    pub fn fields(&self) -> &[MoveValue] {
        match self {
            Self::Runtime(vals) => vals,
            Self::WithFields(_) => {
                // It's not possible to implement this without changing the return type, and some
                // panicking is the best move
                panic!("Getting fields for decorated representation")
            }
        }
    }

    pub fn into_fields(self) -> Vec<MoveValue> {
        match self {
            Self::Runtime(vals) => vals,
            Self::WithFields(vals) => vals.into_iter().map(|(_, f)| f).collect(),
        }
    }
}

impl MoveStructLayout {
    pub fn new(types: Vec<MoveTypeLayout>) -> Self {
        Self::Runtime(types)
    }

    pub fn with_fields(types: Vec<MoveFieldLayout>) -> Self {
        Self::WithFields(types)
    }

    pub fn fields(&self) -> &[MoveTypeLayout] {
        match self {
            Self::Runtime(vals) => vals,
            Self::WithFields(_) => {
                // It's not possible to implement this without changing the return type, and some
                // performance-critical VM serialization code uses the Runtime case of this.
                // panicking is the best move
                panic!("Getting fields for decorated representation")
            }
        }
    }

    pub fn into_fields(self) -> Vec<MoveTypeLayout> {
        match self {
            Self::Runtime(vals) => vals,
            Self::WithFields(vals) => vals.into_iter().map(|f| f.layout).collect(),
        }
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

struct DecoratedStructFieldVisitor<'a>(&'a [MoveFieldLayout]);

impl<'d, 'a> serde::de::Visitor<'d> for DecoratedStructFieldVisitor<'a> {
    type Value = Vec<(Identifier, MoveValue)>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Struct")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'d>,
    {
        let mut vals = Vec::new();
        for (i, layout) in self.0.iter().enumerate() {
            match seq.next_element_seed(layout)? {
                Some(elem) => vals.push(elem),
                None => return Err(A::Error::invalid_length(i, &self)),
            }
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

impl<'d> serde::de::DeserializeSeed<'d> for &MoveFieldLayout {
    type Value = (Identifier, MoveValue);

    fn deserialize<D: serde::de::Deserializer<'d>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        Ok((self.name.clone(), self.layout.deserialize(deserializer)?))
    }
}

impl<'d> serde::de::DeserializeSeed<'d> for &MoveStructLayout {
    type Value = MoveStruct;

    fn deserialize<D: serde::de::Deserializer<'d>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        match self {
            MoveStructLayout::Runtime(layout) => {
                let fields =
                    deserializer.deserialize_tuple(layout.len(), StructFieldVisitor(layout))?;
                Ok(MoveStruct::Runtime(fields))
            }
            MoveStructLayout::WithFields(layout) => {
                let fields = deserializer
                    .deserialize_tuple(layout.len(), DecoratedStructFieldVisitor(layout))?;
                Ok(MoveStruct::WithFields(fields))
            }
        }
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
        match self {
            Self::Runtime(s) => {
                let mut t = serializer.serialize_tuple(s.len())?;
                for v in s.iter() {
                    t.serialize_element(v)?;
                }
                t.end()
            }
            Self::WithFields(s) => {
                let mut t = serializer.serialize_map(Some(s.len()))?;
                for (f, v) in s.iter() {
                    t.serialize_entry(f, v)?;
                }
                t.end()
            }
        }
    }
}
