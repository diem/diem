// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::{Error, Result},
    format::*,
    trace::{SerializationRecords, Tracer},
    value::Value,
};
use serde::{ser, Serialize};

/// Serialize a single value.
/// The lifetime 'a is set by the serialization call site and the `&'a mut`
/// references used to return tracing results and serialization records.
pub(crate) struct Serializer<'a> {
    tracer: &'a mut Tracer,
    records: &'a mut SerializationRecords,
}

impl<'a> Serializer<'a> {
    pub(crate) fn new(tracer: &'a mut Tracer, records: &'a mut SerializationRecords) -> Self {
        Self { tracer, records }
    }
}

impl<'a> ser::Serializer for Serializer<'a> {
    type Ok = (Format, Value);
    type Error = Error;
    type SerializeSeq = SeqSerializer<'a>;
    type SerializeTuple = TupleSerializer<'a>;
    type SerializeTupleStruct = TupleStructSerializer<'a>;
    type SerializeTupleVariant = TupleVariantSerializer<'a>;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = StructSerializer<'a>;
    type SerializeStructVariant = StructVariantSerializer<'a>;

    fn serialize_bool(self, v: bool) -> Result<(Format, Value)> {
        Ok((Format::Bool, Value::Bool(v)))
    }

    fn serialize_i8(self, v: i8) -> Result<(Format, Value)> {
        Ok((Format::I8, Value::I8(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<(Format, Value)> {
        Ok((Format::I16, Value::I16(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<(Format, Value)> {
        Ok((Format::I32, Value::I32(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<(Format, Value)> {
        Ok((Format::I64, Value::I64(v)))
    }

    fn serialize_i128(self, v: i128) -> Result<(Format, Value)> {
        Ok((Format::I128, Value::I128(v)))
    }

    fn serialize_u8(self, v: u8) -> Result<(Format, Value)> {
        Ok((Format::U8, Value::U8(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<(Format, Value)> {
        Ok((Format::U16, Value::U16(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<(Format, Value)> {
        Ok((Format::U32, Value::U32(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<(Format, Value)> {
        Ok((Format::U64, Value::U64(v)))
    }

    fn serialize_u128(self, v: u128) -> Result<(Format, Value)> {
        Ok((Format::U128, Value::U128(v)))
    }

    fn serialize_f32(self, v: f32) -> Result<(Format, Value)> {
        Ok((Format::F32, Value::F32(v)))
    }

    fn serialize_f64(self, v: f64) -> Result<(Format, Value)> {
        Ok((Format::F64, Value::F64(v)))
    }

    fn serialize_char(self, v: char) -> Result<(Format, Value)> {
        Ok((Format::Char, Value::Char(v)))
    }

    fn serialize_str(self, v: &str) -> Result<(Format, Value)> {
        Ok((Format::Str, Value::Str(v.into())))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<(Format, Value)> {
        Ok((Format::Bytes, Value::Bytes(v.into())))
    }

    fn serialize_none(self) -> Result<(Format, Value)> {
        Ok((Format::Unknown, Value::Option(None)))
    }

    fn serialize_some<T>(self, v: &T) -> Result<(Format, Value)>
    where
        T: ?Sized + Serialize,
    {
        let (format, value) = v.serialize(self)?;
        Ok((
            Format::Option(Box::new(format)),
            Value::Option(Some(Box::new(value))),
        ))
    }

    fn serialize_unit(self) -> Result<(Format, Value)> {
        Ok((Format::Unit, Value::Unit))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<(Format, Value)> {
        self.tracer
            .record_container(self.records, name, ContainerFormat::UnitStruct, Value::Unit)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
    ) -> Result<(Format, Value)> {
        self.tracer.record_variant(
            self.records,
            name,
            variant_index,
            variant_name,
            VariantFormat::Unit,
            Value::Unit,
        )
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<(Format, Value)>
    where
        T: ?Sized + Serialize,
    {
        let (format, value) = value.serialize(Serializer::new(self.tracer, self.records))?;
        self.tracer.record_container(
            self.records,
            name,
            ContainerFormat::NewTypeStruct(Box::new(format)),
            value,
        )
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        value: &T,
    ) -> Result<(Format, Value)>
    where
        T: ?Sized + Serialize,
    {
        let (format, value) = value.serialize(Serializer::new(self.tracer, self.records))?;
        self.tracer.record_variant(
            self.records,
            name,
            variant_index,
            variant_name,
            VariantFormat::NewType(Box::new(format)),
            value,
        )
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SeqSerializer {
            tracer: self.tracer,
            records: self.records,
            format: Format::Unknown,
            values: Vec::new(),
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(TupleSerializer {
            tracer: self.tracer,
            records: self.records,
            formats: Vec::new(),
            values: Vec::new(),
        })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(TupleStructSerializer {
            tracer: self.tracer,
            records: self.records,
            name,
            formats: Vec::new(),
            values: Vec::new(),
        })
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(TupleVariantSerializer {
            tracer: self.tracer,
            records: self.records,
            name,
            variant_index,
            variant_name,
            formats: Vec::new(),
            values: Vec::new(),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer {
            tracer: self.tracer,
            records: self.records,
            key_format: Format::Unknown,
            value_format: Format::Unknown,
            values: Vec::new(),
        })
    }

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(StructSerializer {
            tracer: self.tracer,
            records: self.records,
            name,
            fields: Vec::new(),
            values: Vec::new(),
        })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant_name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(StructVariantSerializer {
            tracer: self.tracer,
            records: self.records,
            name,
            variant_index,
            variant_name,
            fields: Vec::new(),
            values: Vec::new(),
        })
    }

    fn is_human_readable(&self) -> bool {
        self.tracer.is_human_readable
    }
}

pub struct SeqSerializer<'a> {
    tracer: &'a mut Tracer,
    records: &'a mut SerializationRecords,

    format: Format,
    values: Vec<Value>,
}

impl<'a> ser::SerializeSeq for SeqSerializer<'a> {
    type Ok = (Format, Value);
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let (format, value) = value.serialize(Serializer::new(self.tracer, self.records))?;
        self.format.unify(format)?;
        self.values.push(value);
        Ok(())
    }

    fn end(self) -> Result<(Format, Value)> {
        Ok((Format::Seq(Box::new(self.format)), Value::Seq(self.values)))
    }
}

pub struct TupleSerializer<'a> {
    tracer: &'a mut Tracer,
    records: &'a mut SerializationRecords,

    formats: Vec<Format>,
    values: Vec<Value>,
}

impl<'a> ser::SerializeTuple for TupleSerializer<'a> {
    type Ok = (Format, Value);
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let (format, value) = value.serialize(Serializer::new(self.tracer, self.records))?;
        self.formats.push(format);
        self.values.push(value);
        Ok(())
    }

    fn end(self) -> Result<(Format, Value)> {
        Ok((Format::Tuple(self.formats), Value::Seq(self.values)))
    }
}

pub struct TupleStructSerializer<'a> {
    tracer: &'a mut Tracer,
    records: &'a mut SerializationRecords,

    name: &'static str,
    formats: Vec<Format>,
    values: Vec<Value>,
}

impl<'a> ser::SerializeTupleStruct for TupleStructSerializer<'a> {
    type Ok = (Format, Value);
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let (format, value) = value.serialize(Serializer::new(self.tracer, self.records))?;
        self.formats.push(format);
        self.values.push(value);
        Ok(())
    }

    fn end(self) -> Result<(Format, Value)> {
        let format = ContainerFormat::TupleStruct(self.formats);
        let value = Value::Seq(self.values);
        self.tracer
            .record_container(self.records, self.name, format, value)
    }
}

pub struct TupleVariantSerializer<'a> {
    tracer: &'a mut Tracer,
    records: &'a mut SerializationRecords,

    name: &'static str,
    variant_index: u32,
    variant_name: &'static str,
    formats: Vec<Format>,
    values: Vec<Value>,
}

impl<'a> ser::SerializeTupleVariant for TupleVariantSerializer<'a> {
    type Ok = (Format, Value);
    type Error = Error;

    fn serialize_field<T>(&mut self, v: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let (format, value) = v.serialize(Serializer::new(self.tracer, self.records))?;
        self.formats.push(format);
        self.values.push(value);
        Ok(())
    }

    fn end(self) -> Result<(Format, Value)> {
        let variant = VariantFormat::Tuple(self.formats);
        let value = Value::Seq(self.values);
        self.tracer.record_variant(
            self.records,
            self.name,
            self.variant_index,
            self.variant_name,
            variant,
            value,
        )
    }
}

pub struct MapSerializer<'a> {
    tracer: &'a mut Tracer,
    records: &'a mut SerializationRecords,

    key_format: Format,
    value_format: Format,
    values: Vec<Value>,
}

impl<'a> ser::SerializeMap for MapSerializer<'a> {
    type Ok = (Format, Value);
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let (format, value) = key.serialize(Serializer::new(self.tracer, self.records))?;
        self.key_format.unify(format)?;
        self.values.push(value);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let (format, value) = value.serialize(Serializer::new(self.tracer, self.records))?;
        self.value_format.unify(format)?;
        self.values.push(value);
        Ok(())
    }

    fn end(self) -> Result<(Format, Value)> {
        let format = Format::Map {
            key: Box::new(self.key_format),
            value: Box::new(self.value_format),
        };
        let value = Value::Seq(self.values);
        Ok((format, value))
    }
}

pub struct StructSerializer<'a> {
    tracer: &'a mut Tracer,
    records: &'a mut SerializationRecords,

    name: &'static str,
    fields: Vec<Named<Format>>,
    values: Vec<Value>,
}

impl<'a> ser::SerializeStruct for StructSerializer<'a> {
    type Ok = (Format, Value);
    type Error = Error;

    fn serialize_field<T>(&mut self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let (format, value) = value.serialize(Serializer::new(self.tracer, self.records))?;
        self.fields.push(Named {
            name: name.into(),
            value: format,
        });
        self.values.push(value);
        Ok(())
    }

    fn end(self) -> Result<(Format, Value)> {
        let format = ContainerFormat::Struct(self.fields);
        let value = Value::Seq(self.values);
        self.tracer
            .record_container(self.records, self.name, format, value)
    }
}

pub struct StructVariantSerializer<'a> {
    tracer: &'a mut Tracer,
    records: &'a mut SerializationRecords,
    name: &'static str,
    variant_index: u32,
    variant_name: &'static str,
    fields: Vec<Named<Format>>,
    values: Vec<Value>,
}

impl<'a> ser::SerializeStructVariant for StructVariantSerializer<'a> {
    type Ok = (Format, Value);
    type Error = Error;

    fn serialize_field<T>(&mut self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let (format, value) = value.serialize(Serializer::new(self.tracer, self.records))?;
        self.fields.push(Named {
            name: name.into(),
            value: format,
        });
        self.values.push(value);
        Ok(())
    }

    fn end(self) -> Result<(Format, Value)> {
        let variant = VariantFormat::Struct(self.fields);
        let value = Value::Seq(self.values);
        self.tracer.record_variant(
            self.records,
            self.name,
            self.variant_index,
            self.variant_name,
            variant,
            value,
        )
    }
}
