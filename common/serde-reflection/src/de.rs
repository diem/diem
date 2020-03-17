// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{Error, Result},
    format::{Format, FormatHolder, Named, VariantFormat},
    trace::Tracer,
};
use serde::de::{self, DeserializeSeed, IntoDeserializer, Visitor};
use std::collections::BTreeMap;

pub(crate) struct Deserializer<'a> {
    tracer: &'a mut Tracer,
    format: &'a mut Format,
}

impl<'a> Deserializer<'a> {
    pub(crate) fn new(tracer: &'a mut Tracer, format: &'a mut Format) -> Self {
        Deserializer { tracer, format }
    }
}

impl<'de, 'a> de::Deserializer<'de> for Deserializer<'a> {
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
        self.format.merge(Format::Bool)?;
        visitor.visit_bool(false)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::I8)?;
        visitor.visit_i8(0)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::I16)?;
        visitor.visit_i16(0)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::I32)?;
        visitor.visit_i32(0)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::I64)?;
        visitor.visit_i64(0)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::I128)?;
        visitor.visit_i128(0)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::U8)?;
        visitor.visit_u8(0)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::U16)?;
        visitor.visit_u16(0)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::U32)?;
        visitor.visit_u32(0)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::U64)?;
        visitor.visit_u64(0)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::U128)?;
        visitor.visit_u128(0)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::F32)?;
        visitor.visit_f32(0.0)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::F64)?;
        visitor.visit_f64(0.0)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::Char)?;
        visitor.visit_char('A')
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::Str)?;
        visitor.visit_borrowed_str("")
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::Str)?;
        visitor.visit_string("".into())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::Bytes)?;
        visitor.visit_borrowed_bytes(b"")
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format
            .merge(Format::Option(Box::new(Format::Unknown)))?;
        let format = match &mut self.format {
            Format::Option(x) => x,
            _ => unreachable!(),
        };
        if **format == Format::Unknown {
            let inner = Deserializer::new(self.tracer, format.as_mut());
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
        self.format.merge(Format::Unit)?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::TypeName(name.into()))?;
        self.tracer
            .registry
            .entry(name)
            .or_insert(Format::Unknown)
            .merge(Format::UnitStruct)?;
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // If a newtype struct was visited by the serialization tracer, use the recorded value.
        if let Some(hint) = self.tracer.get_value(name) {
            return visitor.visit_newtype_struct(hint.clone().into_deserializer());
        }
        self.format.merge(Format::TypeName(name.into()))?;
        let entry = self.tracer.registry.entry(name).or_insert(Format::Unknown);
        entry.merge(Format::NewTypeStruct(Box::new(Format::Unknown)))?;

        let mut format = Format::Unknown;
        let inner = Deserializer::new(self.tracer, &mut format);
        let value = visitor.visit_newtype_struct(inner)?;
        match self.tracer.registry.get_mut(name) {
            Some(Format::NewTypeStruct(x)) => {
                *x = Box::new(format);
            }
            _ => unreachable!(),
        };
        Ok(value)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::Seq(Box::new(Format::Unknown)))?;
        let format = match &mut self.format {
            Format::Seq(x) => x,
            _ => unreachable!(),
        };
        if **format == Format::Unknown {
            // Simulate vector of size 1.
            let inner = SeqDeserializer::new(self.tracer, std::iter::once(format.as_mut()));
            visitor.visit_seq(inner)
        } else {
            // Cut exploration with a vector of size 0.
            let inner = SeqDeserializer::new(self.tracer, std::iter::empty());
            visitor.visit_seq(inner)
        }
    }

    fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format
            .merge(Format::Tuple(vec![Format::Unknown; len]))?;
        let formats = match &mut self.format {
            Format::Tuple(x) => x,
            _ => unreachable!(),
        };
        let inner = SeqDeserializer::new(self.tracer, formats.iter_mut());
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
        self.format.merge(Format::TypeName(name.into()))?;
        let entry = self.tracer.registry.entry(name).or_insert(Format::Unknown);
        let mut formats = vec![Format::Unknown; len];
        // Update the registry with an intermediate result to stop recursion.
        entry.merge(Format::TupleStruct(formats.clone()))?;
        // Compute the formats.
        let inner = SeqDeserializer::new(self.tracer, formats.iter_mut());
        let value = visitor.visit_seq(inner)?;
        // Finally, update the registry.
        match self.tracer.registry.get_mut(name) {
            Some(Format::TupleStruct(x)) => {
                *x = formats;
            }
            _ => unreachable!(),
        };
        Ok(value)
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format.merge(Format::Map {
            key: Box::new(Format::Unknown),
            value: Box::new(Format::Unknown),
        })?;
        let formats = match &mut self.format {
            Format::Map { key, value } => vec![key.as_mut(), value.as_mut()],
            _ => unreachable!(),
        };
        if *formats[0] == Format::Unknown || *formats[1] == Format::Unknown {
            // Simulate a map with one entry.
            let inner = SeqDeserializer::new(self.tracer, formats.into_iter());
            visitor.visit_map(inner)
        } else {
            // Stop exploration.
            let inner = SeqDeserializer::new(self.tracer, std::iter::empty());
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
        self.format.merge(Format::TypeName(name.into()))?;
        let entry = self.tracer.registry.entry(name).or_insert(Format::Unknown);
        let mut formats: Vec<_> = fields
            .iter()
            .map(|&name| Named {
                name: name.into(),
                value: Format::Unknown,
            })
            .collect();
        // Update the registry with an intermediate result to stop recursion.
        entry.merge(Format::Struct(formats.clone()))?;
        // Compute the formats.
        let inner = SeqDeserializer::new(
            self.tracer,
            formats.iter_mut().map(|named| &mut named.value),
        );
        let value = visitor.visit_seq(inner)?;
        // Finally, update the registry.
        match self.tracer.registry.get_mut(name) {
            Some(Format::Struct(x)) => {
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
        self.format.merge(Format::TypeName(name.into()))?;
        let entry = self.tracer.registry.entry(name).or_insert(Format::Unknown);
        // Update the registry with an intermediate result to stop recursion.
        entry.merge(Format::Variant(BTreeMap::new()))?;
        let current_variants = match entry {
            Format::Variant(x) => x,
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
        let inner = EnumDeserializer::new(self.tracer, index, &mut variant.value);
        let value = visitor.visit_enum(inner)?;
        // Finally, update the registry.
        let current_variants = match self.tracer.registry.get_mut(name) {
            Some(Format::Variant(x)) => x,
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

struct SeqDeserializer<'a, I> {
    tracer: &'a mut Tracer,
    formats: I,
}

impl<'a, I> SeqDeserializer<'a, I> {
    fn new(tracer: &'a mut Tracer, formats: I) -> Self {
        Self { tracer, formats }
    }
}

impl<'de, 'a, I> de::SeqAccess<'de> for SeqDeserializer<'a, I>
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
        let inner = Deserializer::new(self.tracer, format);
        seed.deserialize(inner).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        self.formats.size_hint().1
    }
}

impl<'de, 'a, I> de::MapAccess<'de> for SeqDeserializer<'a, I>
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
        let inner = Deserializer::new(self.tracer, format);
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
        let inner = Deserializer::new(self.tracer, format);
        seed.deserialize(inner)
    }

    fn size_hint(&self) -> Option<usize> {
        self.formats.size_hint().1.map(|x| x / 2)
    }
}

struct EnumDeserializer<'a> {
    tracer: &'a mut Tracer,
    index: u32,
    format: &'a mut VariantFormat,
}

impl<'a> EnumDeserializer<'a> {
    fn new(tracer: &'a mut Tracer, index: u32, format: &'a mut VariantFormat) -> Self {
        Self {
            tracer,
            index,
            format,
        }
    }
}

impl<'de, 'a> de::EnumAccess<'de> for EnumDeserializer<'a> {
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

impl<'de, 'a> de::VariantAccess<'de> for EnumDeserializer<'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        self.format.merge(VariantFormat::Unit)
    }

    fn newtype_variant_seed<T>(mut self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        self.format
            .merge(VariantFormat::NewType(Box::new(Format::Unknown)))?;
        let format = match &mut self.format {
            VariantFormat::NewType(x) => x.as_mut(),
            _ => unreachable!(),
        };
        let inner = Deserializer::new(self.tracer, format);
        seed.deserialize(inner)
    }

    fn tuple_variant<V>(mut self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.format
            .merge(VariantFormat::Tuple(vec![Format::Unknown; len]))?;
        let formats = match &mut self.format {
            VariantFormat::Tuple(x) => x,
            _ => unreachable!(),
        };
        let inner = SeqDeserializer::new(self.tracer, formats.iter_mut());
        visitor.visit_seq(inner)
    }

    fn struct_variant<V>(mut self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
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
        self.format.merge(VariantFormat::Struct(formats))?;

        let formats = match &mut self.format {
            VariantFormat::Struct(x) => x,
            _ => unreachable!(),
        };
        let inner = SeqDeserializer::new(
            self.tracer,
            formats.iter_mut().map(|named| &mut named.value),
        );
        visitor.visit_seq(inner)
    }
}
