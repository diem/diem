// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::{Error, Result};
use serde::de::{self, DeserializeSeed, IntoDeserializer, Visitor};

/// A structured Serde value.
/// Meant to be easily recorded while tracing serialization and easily used while tracing deserialization.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Bool(bool),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

    F32(f32),
    F64(f64),

    Char(char),
    Str(String),
    Bytes(Vec<u8>),

    Option(Option<Box<Value>>),
    Variant(u32, Box<Value>),
    Seq(Vec<Value>),
}

/// Deserializer meant to reconstruct the Rust value behind a particular Serde value.
pub struct Deserializer<'de> {
    value: &'de Value,
}

impl<'de> Deserializer<'de> {
    pub fn new(value: &'de Value) -> Self {
        Self { value }
    }
}

impl<'de> IntoDeserializer<'de, Error> for &'de Value {
    type Deserializer = Deserializer<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        Deserializer::new(self)
    }
}

impl Value {
    pub(crate) fn seq_values(&self) -> Result<&Vec<Value>> {
        match self {
            Value::Seq(x) => Ok(x),
            _ => Err(Error::DeserializationError("seq_values")),
        }
    }
}

macro_rules! declare_deserialize {
    ($method:ident, $token:ident, $visit:ident, $str:expr) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.value {
                Value::$token(x) => visitor.$visit(x.clone()),
                _ => Err(Error::DeserializationError($str)),
            }
        }
    }
}

macro_rules! declare_deserialize_borrowed {
    ($method:ident, $token:ident, $visit:ident, $str:expr) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            match self.value {
                Value::$token(x) => visitor.$visit(x),
                _ => Err(Error::DeserializationError($str)),
            }
        }
    }
}

impl<'de> de::Deserializer<'de> for Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_any"))
    }

    declare_deserialize!(deserialize_bool, Bool, visit_bool, "bool");

    declare_deserialize!(deserialize_i8, I8, visit_i8, "i8");
    declare_deserialize!(deserialize_i16, I16, visit_i16, "i16");
    declare_deserialize!(deserialize_i32, I32, visit_i32, "i32");
    declare_deserialize!(deserialize_i64, I64, visit_i64, "i64");
    declare_deserialize!(deserialize_i128, I128, visit_i128, "i128");

    declare_deserialize!(deserialize_u8, U8, visit_u8, "u8");
    declare_deserialize!(deserialize_u16, U16, visit_u16, "u16");
    declare_deserialize!(deserialize_u32, U32, visit_u32, "u32");
    declare_deserialize!(deserialize_u64, U64, visit_u64, "u64");
    declare_deserialize!(deserialize_u128, U128, visit_u128, "u128");

    declare_deserialize!(deserialize_f32, F32, visit_f32, "f32");
    declare_deserialize!(deserialize_f64, F64, visit_f64, "f64");

    declare_deserialize!(deserialize_char, Char, visit_char, "char");
    declare_deserialize!(deserialize_string, Str, visit_string, "string");
    declare_deserialize_borrowed!(deserialize_str, Str, visit_borrowed_str, "str");
    declare_deserialize!(deserialize_byte_buf, Bytes, visit_byte_buf, "byte_buf");
    declare_deserialize_borrowed!(deserialize_bytes, Bytes, visit_borrowed_bytes, "bytes");

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Option(None) => visitor.visit_none(),
            Value::Option(Some(x)) => visitor.visit_some(x.into_deserializer()),
            _ => Err(Error::DeserializationError("option")),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Unit => visitor.visit_unit(),
            _ => Err(Error::DeserializationError("unit")),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Unit => visitor.visit_unit(),
            _ => Err(Error::DeserializationError("unit struct")),
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Seq(x) => visitor.visit_seq(x.into_seq_deserializer()),
            _ => Err(Error::DeserializationError("seq")),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Seq(x) => visitor.visit_seq(x.into_seq_deserializer()),
            _ => Err(Error::DeserializationError("tuple")),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Seq(x) => visitor.visit_seq(x.into_seq_deserializer()),
            _ => Err(Error::DeserializationError("tuple struct")),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Seq(x) => visitor.visit_seq(x.into_seq_deserializer()),
            _ => Err(Error::DeserializationError("map")),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Seq(x) => visitor.visit_seq(x.into_seq_deserializer()),
            _ => Err(Error::DeserializationError("tuple struct")),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Variant(index, variant) => {
                let inner = EnumDeserializer::new(*index, &*variant);
                visitor.visit_enum(inner)
            }
            _ => Err(Error::DeserializationError("enum")),
        }
    }

    // Not needed since we always deserialize structs as sequences.
    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_identifier"))
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_ignored_any"))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

pub(crate) struct SeqDeserializer<I> {
    values: I,
}

pub(crate) trait IntoSeqDeserializer {
    type SeqDeserializer;

    fn into_seq_deserializer(self) -> Self::SeqDeserializer;
}

impl<I> SeqDeserializer<I> {
    fn new(values: I) -> Self {
        Self { values }
    }
}

impl<'de> IntoSeqDeserializer for &'de Vec<Value> {
    type SeqDeserializer = SeqDeserializer<std::slice::Iter<'de, Value>>;

    fn into_seq_deserializer(self) -> Self::SeqDeserializer {
        SeqDeserializer::new(self.iter())
    }
}

impl<'de, I> de::SeqAccess<'de> for SeqDeserializer<I>
where
    I: Iterator<Item = &'de Value>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.values.next() {
            Some(x) => seed.deserialize(x.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.values.size_hint().1
    }
}

impl<'de, I> de::MapAccess<'de> for SeqDeserializer<I>
where
    I: Iterator<Item = &'de Value>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.values.next() {
            Some(x) => seed.deserialize(x.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        match self.values.next() {
            Some(x) => seed.deserialize(x.into_deserializer()),
            None => Err(Error::DeserializationError("value in map")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.values.size_hint().1.map(|x| x / 2)
    }
}

struct EnumDeserializer<'de> {
    index: u32,
    value: &'de Value,
}

impl<'de> EnumDeserializer<'de> {
    fn new(index: u32, value: &'de Value) -> Self {
        Self { index, value }
    }
}

impl<'de> de::EnumAccess<'de> for EnumDeserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(self.index.into_deserializer())?;
        Ok((value, self))
    }
}

impl<'de> de::VariantAccess<'de> for EnumDeserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        match self.value {
            Value::Unit => Ok(()),
            _ => Err(Error::DeserializationError("unit variant")),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.value.into_deserializer())
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Seq(x) => visitor.visit_seq(x.into_seq_deserializer()),
            _ => Err(Error::DeserializationError("tuple variant")),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Value::Seq(x) => visitor.visit_seq(x.into_seq_deserializer()),
            _ => Err(Error::DeserializationError("struct variant")),
        }
    }
}
