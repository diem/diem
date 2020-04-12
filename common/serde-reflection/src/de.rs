// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{Error, Result},
    format::{ContainerFormat, ContainerFormatEntry, Format, FormatHolder, Named, VariantFormat},
    trace::{Records, Tracer},
};
use serde::de::{self, DeserializeSeed, IntoDeserializer, Visitor};
use std::collections::BTreeMap;

pub(crate) struct Deserializer<'de, 'a> {
    tracer: &'a mut Tracer,
    records: &'de Records,
    format: &'a mut Format,
}

impl<'de, 'a> Deserializer<'de, 'a> {
    pub(crate) fn new(
        tracer: &'a mut Tracer,
        records: &'de Records,
        format: &'a mut Format,
    ) -> Self {
        Deserializer {
            tracer,
            records,
            format,
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for Deserializer<'de, 'a> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_any"))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::Bool)?;
        visitor.visit_bool(false)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::I8)?;
        visitor.visit_i8(0)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::I16)?;
        visitor.visit_i16(0)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::I32)?;
        visitor.visit_i32(0)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::I64)?;
        visitor.visit_i64(0)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::I128)?;
        visitor.visit_i128(0)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::U8)?;
        visitor.visit_u8(0)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::U16)?;
        visitor.visit_u16(0)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::U32)?;
        visitor.visit_u32(0)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::U64)?;
        visitor.visit_u64(0)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::U128)?;
        visitor.visit_u128(0)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::F32)?;
        visitor.visit_f32(0.0)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::F64)?;
        visitor.visit_f64(0.0)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::Char)?;
        visitor.visit_char('A')
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::Str)?;
        visitor.visit_borrowed_str("")
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::Str)?;
        visitor.visit_string(String::new())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::Bytes)?;
        visitor.visit_borrowed_bytes(b"")
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::Bytes)?;
        visitor.visit_byte_buf(Vec::new())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format
            .unify(Format::Option(Box::new(Format::Unknown)))?;
        let format = match self.format {
            Format::Option(x) => x,
            _ => unreachable!(),
        };
        if **format == Format::Unknown {
            let inner = Deserializer::new(self.tracer, self.records, format.as_mut());
            visitor.visit_some(inner)
        } else {
            // Cut exploration.
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::Unit)?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::TypeName(name.into()))?;
        self.tracer
            .registry
            .entry(name)
            .unify(ContainerFormat::UnitStruct)?;
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::TypeName(name.into()))?;
        // If a newtype struct was visited by the serialization tracer, use the recorded value.
        if let Some((format, hint)) = self.tracer.get_recorded_value(self.records, name) {
            // Hints are recorded during serialization-tracing therefore the registry is already accurate.
            return visitor
                .visit_newtype_struct(hint.into_deserializer())
                .map_err(|err| match err {
                    Error::DeserializationError(msg) => {
                        Error::UnexpectedDeserializationFormat(name, format.clone(), msg)
                    }
                    _ => err,
                });
        }
        self.tracer
            .registry
            .entry(name)
            .unify(ContainerFormat::NewTypeStruct(Box::new(Format::Unknown)))?;

        let mut format = Format::Unknown;
        let inner = Deserializer::new(self.tracer, self.records, &mut format);
        let value = visitor.visit_newtype_struct(inner)?;
        match self.tracer.registry.get_mut(name) {
            Some(ContainerFormat::NewTypeStruct(x)) => {
                *x = Box::new(format);
            }
            _ => unreachable!(),
        };
        Ok(value)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::Seq(Box::new(Format::Unknown)))?;
        let format = match self.format {
            Format::Seq(x) => x,
            _ => unreachable!(),
        };
        if **format == Format::Unknown {
            // Simulate vector of size 1.
            let inner =
                SeqDeserializer::new(self.tracer, self.records, std::iter::once(format.as_mut()));
            visitor.visit_seq(inner)
        } else {
            // Cut exploration with a vector of size 0.
            let inner = SeqDeserializer::new(self.tracer, self.records, std::iter::empty());
            visitor.visit_seq(inner)
        }
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format
            .unify(Format::Tuple(vec![Format::Unknown; len]))?;
        let formats = match self.format {
            Format::Tuple(x) => x,
            _ => unreachable!(),
        };
        let inner = SeqDeserializer::new(self.tracer, self.records, formats.iter_mut());
        visitor.visit_seq(inner)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::TypeName(name.into()))?;
        let mut formats = vec![Format::Unknown; len];
        // Update the registry with an intermediate result to stop recursion.
        self.tracer
            .registry
            .entry(name)
            .unify(ContainerFormat::TupleStruct(formats.clone()))?;
        // Compute the formats.
        let inner = SeqDeserializer::new(self.tracer, self.records, formats.iter_mut());
        let value = visitor.visit_seq(inner)?;
        // Finally, update the registry.
        match self.tracer.registry.get_mut(name) {
            Some(ContainerFormat::TupleStruct(x)) => {
                *x = formats;
            }
            _ => unreachable!(),
        };
        Ok(value)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::Map {
            key: Box::new(Format::Unknown),
            value: Box::new(Format::Unknown),
        })?;
        let formats = match self.format {
            Format::Map { key, value } => vec![key.as_mut(), value.as_mut()],
            _ => unreachable!(),
        };
        if *formats[0] == Format::Unknown || *formats[1] == Format::Unknown {
            // Simulate a map with one entry.
            let inner = SeqDeserializer::new(self.tracer, self.records, formats.into_iter());
            visitor.visit_map(inner)
        } else {
            // Stop exploration.
            let inner = SeqDeserializer::new(self.tracer, self.records, std::iter::empty());
            visitor.visit_map(inner)
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.unify(Format::TypeName(name.into()))?;
        let mut formats: Vec<_> = fields
            .iter()
            .map(|&name| Named {
                name: name.into(),
                value: Format::Unknown,
            })
            .collect();
        // Update the registry with an intermediate result to stop recursion.
        self.tracer
            .registry
            .entry(name)
            .unify(ContainerFormat::Struct(formats.clone()))?;
        // Compute the formats.
        let inner = SeqDeserializer::new(
            self.tracer,
            self.records,
            formats.iter_mut().map(|named| &mut named.value),
        );
        let value = visitor.visit_seq(inner)?;
        // Finally, update the registry.
        match self.tracer.registry.get_mut(name) {
            Some(ContainerFormat::Struct(x)) => {
                *x = formats;
            }
            _ => unreachable!(),
        };
        Ok(value)
    }

    // Assumption: The first variant(s) should be "base cases", i.e. not cause infinite recursion
    // while constructing sample values.
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        assert!(
            !variants.is_empty(),
            "Enums should have at least one variant."
        );
        self.format.unify(Format::TypeName(name.into()))?;
        // Update the registry with an intermediate result to stop recursion.
        self.tracer
            .registry
            .entry(name)
            .unify(ContainerFormat::Enum(BTreeMap::new()))?;
        let current_variants = match self.tracer.registry.get(name) {
            Some(ContainerFormat::Enum(x)) => x,
            _ => unreachable!(),
        };
        let (index, mut variant) = if current_variants.len() == variants.len()
            || self.tracer.incomplete_enums.contains(name)
        {
            // If we have found all the variants OR if the enum is marked as
            // incomplete already, pick the first variant.
            (0, current_variants[&0].clone())
        } else {
            assert!(
                current_variants.len() < variants.len(),
                "Current variants cannot exceed the entire list."
            );
            // Otherwise, create a new variant format.
            let index = current_variants.len() as u32;
            let variant = Named {
                name: variants[index as usize].into(),
                value: VariantFormat::Unknown,
            };
            (index, variant)
        };

        // Compute the formats.
        let inner = EnumDeserializer::new(self.tracer, self.records, index, &mut variant.value);
        let value = visitor.visit_enum(inner)?;
        // Finally, update the registry.
        let current_variants = match self.tracer.registry.get_mut(name) {
            Some(ContainerFormat::Enum(x)) => x,
            _ => unreachable!(),
        };
        current_variants.insert(index, variant);
        if current_variants.len() != variants.len() {
            self.tracer.incomplete_enums.insert(name.into());
        }
        Ok(value)
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
        self.tracer.is_human_readable
    }
}

struct SeqDeserializer<'de, 'a, I> {
    tracer: &'a mut Tracer,
    records: &'de Records,
    formats: I,
}

impl<'de, 'a, I> SeqDeserializer<'de, 'a, I> {
    fn new(tracer: &'a mut Tracer, records: &'de Records, formats: I) -> Self {
        Self {
            tracer,
            records,
            formats,
        }
    }
}

impl<'de, 'a, I> de::SeqAccess<'de> for SeqDeserializer<'de, 'a, I>
where
    I: Iterator<Item = &'a mut Format>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        let format = match self.formats.next() {
            Some(x) => x,
            None => return Ok(None),
        };
        let inner = Deserializer::new(self.tracer, self.records, format);
        seed.deserialize(inner).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        self.formats.size_hint().1
    }
}

impl<'de, 'a, I> de::MapAccess<'de> for SeqDeserializer<'de, 'a, I>
where
    // Must have an even number of elements
    I: Iterator<Item = &'a mut Format>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        let format = match self.formats.next() {
            Some(x) => x,
            None => return Ok(None),
        };
        let inner = Deserializer::new(self.tracer, self.records, format);
        seed.deserialize(inner).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let format = match self.formats.next() {
            Some(x) => x,
            None => unreachable!(),
        };
        let inner = Deserializer::new(self.tracer, self.records, format);
        seed.deserialize(inner)
    }

    fn size_hint(&self) -> Option<usize> {
        self.formats.size_hint().1.map(|x| x / 2)
    }
}

struct EnumDeserializer<'de, 'a> {
    tracer: &'a mut Tracer,
    records: &'de Records,
    index: u32,
    format: &'a mut VariantFormat,
}

impl<'de, 'a> EnumDeserializer<'de, 'a> {
    fn new(
        tracer: &'a mut Tracer,
        records: &'de Records,
        index: u32,
        format: &'a mut VariantFormat,
    ) -> Self {
        Self {
            tracer,
            records,
            index,
            format,
        }
    }
}

impl<'de, 'a> de::EnumAccess<'de> for EnumDeserializer<'de, 'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let index = self.index;
        let value = seed.deserialize(index.into_deserializer())?;
        Ok((value, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for EnumDeserializer<'de, 'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        self.format.unify(VariantFormat::Unit)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        self.format
            .unify(VariantFormat::NewType(Box::new(Format::Unknown)))?;
        let format = match self.format {
            VariantFormat::NewType(x) => x.as_mut(),
            _ => unreachable!(),
        };
        let inner = Deserializer::new(self.tracer, self.records, format);
        seed.deserialize(inner)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format
            .unify(VariantFormat::Tuple(vec![Format::Unknown; len]))?;
        let formats = match self.format {
            VariantFormat::Tuple(x) => x,
            _ => unreachable!(),
        };
        let inner = SeqDeserializer::new(self.tracer, self.records, formats.iter_mut());
        visitor.visit_seq(inner)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let formats: Vec<_> = fields
            .iter()
            .map(|&name| Named {
                name: name.into(),
                value: Format::Unknown,
            })
            .collect();
        self.format.unify(VariantFormat::Struct(formats))?;

        let formats = match self.format {
            VariantFormat::Struct(x) => x,
            _ => unreachable!(),
        };
        let inner = SeqDeserializer::new(
            self.tracer,
            self.records,
            formats.iter_mut().map(|named| &mut named.value),
        );
        visitor.visit_seq(inner)
    }
}
