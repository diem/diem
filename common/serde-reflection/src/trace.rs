// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{Error, Result},
    format::*,
};
use serde::{ser, Serialize};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Tracer {
    /// Whether to trace the human readable variant of the Serialize trait.
    is_human_readable: bool,
    /// Formats of all named containers that we have traced so far.
    registry: BTreeMap<&'static str, Format>,
}

impl Tracer {
    /// Start tracing serialization.
    pub fn new(is_human_readable: bool) -> Self {
        Self {
            is_human_readable,
            registry: BTreeMap::new(),
        }
    }

    /// Trace the serialization of a particular value.
    /// Nested containers will be added to the global registry, indexed by
    /// their (non-qualified) name.
    pub fn trace<T>(&mut self, value: &T) -> Result<Format>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    /// Finish tracing and recover a map of normalized formats.
    pub fn registry(self) -> Result<BTreeMap<&'static str, Format>> {
        let mut registry = self.registry;
        for format in registry.values_mut() {
            format.normalize()?;
        }
        Ok(registry)
    }
}

impl Tracer {
    fn record(&mut self, name: &'static str, format: Format) -> Result<Format> {
        // Note that we assume unqualified names to be unique.
        let entry = self.registry.entry(name).or_insert(Format::Unknown);
        entry.merge(format)?;
        Ok(Format::TypeName(name.into()))
    }

    fn record_variant(
        &mut self,
        name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        variant: VariantFormat,
    ) -> Result<Format> {
        let mut variants = BTreeMap::new();
        variants.insert(
            variant_index,
            Named {
                name: variant_name.into(),
                value: variant,
            },
        );
        self.record(name, Format::Variant(variants))
    }
}

impl<'a> ser::Serializer for &'a mut Tracer {
    type Ok = Format;
    type Error = Error;
    type SerializeSeq = SeqTracer<'a>;
    type SerializeTuple = TupleTracer<'a>;
    type SerializeTupleStruct = TupleStructTracer<'a>;
    type SerializeTupleVariant = TupleVariantTracer<'a>;
    type SerializeMap = MapTracer<'a>;
    type SerializeStruct = StructTracer<'a>;
    type SerializeStructVariant = StructVariantTracer<'a>;

    fn serialize_bool(self, _v: bool) -> Result<Format> {
        Ok(Format::Bool)
    }

    fn serialize_i8(self, _v: i8) -> Result<Format> {
        Ok(Format::I8)
    }

    fn serialize_i16(self, _v: i16) -> Result<Format> {
        Ok(Format::I16)
    }

    fn serialize_i32(self, _v: i32) -> Result<Format> {
        Ok(Format::I32)
    }

    fn serialize_i64(self, _v: i64) -> Result<Format> {
        Ok(Format::I64)
    }

    fn serialize_i128(self, _v: i128) -> Result<Format> {
        Ok(Format::I128)
    }

    fn serialize_u8(self, _v: u8) -> Result<Format> {
        Ok(Format::U8)
    }

    fn serialize_u16(self, _v: u16) -> Result<Format> {
        Ok(Format::U16)
    }

    fn serialize_u32(self, _v: u32) -> Result<Format> {
        Ok(Format::U32)
    }

    fn serialize_u64(self, _v: u64) -> Result<Format> {
        Ok(Format::U64)
    }

    fn serialize_u128(self, _v: u128) -> Result<Format> {
        Ok(Format::U128)
    }

    fn serialize_f32(self, _v: f32) -> Result<Format> {
        Ok(Format::F32)
    }

    fn serialize_f64(self, _v: f64) -> Result<Format> {
        Ok(Format::F64)
    }

    fn serialize_char(self, _v: char) -> Result<Format> {
        Ok(Format::Char)
    }

    fn serialize_str(self, _v: &str) -> Result<Format> {
        Ok(Format::Str)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Format> {
        Ok(Format::Bytes)
    }

    fn serialize_none(self) -> Result<Format> {
        Ok(Format::Unknown)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Format>
    where
        T: ?Sized + Serialize,
    {
        let format = value.serialize(self)?;
        Ok(Format::Option(Box::new(format)))
    }

    fn serialize_unit(self) -> Result<Format> {
        Ok(Format::Unit)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Format> {
        self.record(name, Format::UnitStruct)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
    ) -> Result<Format> {
        self.record_variant(name, variant_index, variant_name, VariantFormat::Unit)
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Format>
    where
        T: ?Sized + Serialize,
    {
        let format = Box::new(value.serialize(&mut *self)?);
        self.record(name, Format::NewTypeStruct(format))
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        value: &T,
    ) -> Result<Format>
    where
        T: ?Sized + Serialize,
    {
        let format = Box::new(value.serialize(&mut *self)?);
        self.record_variant(
            name,
            variant_index,
            variant_name,
            VariantFormat::NewType(format),
        )
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SeqTracer {
            tracer: self,
            value: Format::Unknown,
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(TupleTracer {
            tracer: self,
            values: Vec::new(),
        })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(TupleStructTracer {
            tracer: self,
            name,
            fields: Vec::new(),
        })
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(TupleVariantTracer {
            tracer: self,
            name,
            variant_index,
            variant_name,
            fields: Vec::new(),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapTracer {
            tracer: self,
            key: Format::Unknown,
            value: Format::Unknown,
        })
    }

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(StructTracer {
            tracer: self,
            name,
            fields: Vec::new(),
        })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(StructVariantTracer {
            tracer: self,
            name,
            variant_index,
            variant_name,
            fields: Vec::new(),
        })
    }

    fn is_human_readable(&self) -> bool {
        self.is_human_readable
    }
}

pub struct SeqTracer<'a> {
    tracer: &'a mut Tracer,
    value: Format,
}

impl<'a> ser::SerializeSeq for SeqTracer<'a> {
    type Ok = Format;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let format = value.serialize(&mut *self.tracer)?;
        self.value.merge(format)
    }

    fn end(self) -> Result<Format> {
        Ok(Format::Seq(Box::new(self.value)))
    }
}

pub struct TupleTracer<'a> {
    tracer: &'a mut Tracer,
    values: Vec<Format>,
}

impl<'a> ser::SerializeTuple for TupleTracer<'a> {
    type Ok = Format;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let format = value.serialize(&mut *self.tracer)?;
        self.values.push(format);
        Ok(())
    }

    fn end(self) -> Result<Format> {
        Ok(Format::Tuple(self.values))
    }
}

pub struct TupleStructTracer<'a> {
    tracer: &'a mut Tracer,
    name: &'static str,
    fields: Vec<Format>,
}

impl<'a> ser::SerializeTupleStruct for TupleStructTracer<'a> {
    type Ok = Format;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let format = value.serialize(&mut *self.tracer)?;
        self.fields.push(format);
        Ok(())
    }

    fn end(self) -> Result<Format> {
        let format = Format::TupleStruct(self.fields);
        self.tracer.record(self.name, format)
    }
}

pub struct TupleVariantTracer<'a> {
    tracer: &'a mut Tracer,
    name: &'static str,
    variant_index: u32,
    variant_name: &'static str,
    fields: Vec<Format>,
}

impl<'a> ser::SerializeTupleVariant for TupleVariantTracer<'a> {
    type Ok = Format;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let format = value.serialize(&mut *self.tracer)?;
        self.fields.push(format);
        Ok(())
    }

    fn end(self) -> Result<Format> {
        let variant = VariantFormat::Tuple(self.fields);
        self.tracer
            .record_variant(self.name, self.variant_index, self.variant_name, variant)
    }
}

pub struct MapTracer<'a> {
    tracer: &'a mut Tracer,
    key: Format,
    value: Format,
}

impl<'a> ser::SerializeMap for MapTracer<'a> {
    type Ok = Format;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let format = key.serialize(&mut *self.tracer)?;
        self.key.merge(format)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let format = value.serialize(&mut *self.tracer)?;
        self.value.merge(format)
    }

    fn end(self) -> Result<Format> {
        Ok(Format::Map {
            key: Box::new(self.key),
            value: Box::new(self.value),
        })
    }
}

pub struct StructTracer<'a> {
    tracer: &'a mut Tracer,
    name: &'static str,
    fields: Vec<Named<Format>>,
}

impl<'a> ser::SerializeStruct for StructTracer<'a> {
    type Ok = Format;
    type Error = Error;

    fn serialize_field<T>(&mut self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let format = value.serialize(&mut *self.tracer)?;
        self.fields.push(Named {
            name: name.into(),
            value: format,
        });
        Ok(())
    }

    fn end(self) -> Result<Format> {
        let format = Format::Struct(self.fields);
        self.tracer.record(self.name, format)
    }
}

pub struct StructVariantTracer<'a> {
    tracer: &'a mut Tracer,
    name: &'static str,
    variant_index: u32,
    variant_name: &'static str,
    fields: Vec<Named<Format>>,
}

impl<'a> ser::SerializeStructVariant for StructVariantTracer<'a> {
    type Ok = Format;
    type Error = Error;

    fn serialize_field<T>(&mut self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let format = value.serialize(&mut *self.tracer)?;
        self.fields.push(Named {
            name: name.into(),
            value: format,
        });
        Ok(())
    }

    fn end(self) -> Result<Format> {
        let variant = VariantFormat::Struct(self.fields);
        self.tracer
            .record_variant(self.name, self.variant_index, self.variant_name, variant)
    }
}
