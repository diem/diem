// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::{Error, Result};
use serde::{ser, Serialize};
use std::collections::BTreeMap;

/// Serialize the given data structure as a `Vec<u8>` of LCS.
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to
/// fail, if `T` contains sequences which are longer than `MAX_SEQUENCE_LENGTH`,
/// or if `T` attempts to serialize an unsupported datatype such as a f32,
/// f64, or char.
///
/// # Examples
///
/// ```
/// use libra_canonical_serialization::to_bytes;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Ip([u8; 4]);
///
/// #[derive(Serialize)]
/// struct Port(u16);
///
/// #[derive(Serialize)]
/// struct Service {
///     ip: Ip,
///     port: Vec<Port>,
///     connection_max: Option<u32>,
///     enabled: bool,
/// }
///
/// let service = Service {
///     ip: Ip([192, 168, 1, 1]),
///     port: vec![Port(8001), Port(8002), Port(8003)],
///     connection_max: Some(5000),
///     enabled: false,
/// };
///
/// let bytes = to_bytes(&service).unwrap();
/// let expected = vec![
///     0xc0, 0xa8, 0x01, 0x01, 0x03, 0x41, 0x1f, 0x42,
///     0x1f, 0x43, 0x1f, 0x01, 0x88, 0x13, 0x00, 0x00,
///     0x00,
/// ];
/// assert_eq!(bytes, expected);
/// ```
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut serializer = Serializer::new();
    value.serialize(&mut serializer)?;
    Ok(serializer.end())
}

pub fn is_human_readable() -> bool {
    let mut serializer = Serializer::new();
    ser::Serializer::is_human_readable(&&mut serializer)
}

/// Serialization implementation for LCS
struct Serializer {
    output: Vec<u8>,
}

impl Serializer {
    /// Creates a new `Serializer` which will emit LCS.
    fn new() -> Self {
        Self { output: Vec::new() }
    }

    /// The `Serializer::end` method should be called after a type has been
    /// fully serialized. This will return the buffer containing the LCS
    /// serialized data.
    fn end(self) -> Vec<u8> {
        self.output
    }

    fn serialize_u32_as_uleb128(&mut self, mut value: u32) -> Result<()> {
        use serde::ser::Serializer;
        while value >= 0x80 {
            // Write 7 (lowest) bits of data and set the 8th bit to 1.
            let byte = (value & 0x7f) as u8;
            self.serialize_u8(byte | 0x80)?;
            value >>= 7;
        }
        // Write the remaining bits of data and set the highest bit to 0.
        self.serialize_u8(value as u8)
    }

    fn serialize_variant_index(&mut self, v: u32) -> Result<()> {
        self.serialize_u32_as_uleb128(v)
    }

    /// Serialize a sequence length as a u32.
    fn serialize_seq_len(&mut self, len: usize) -> Result<()> {
        if len > crate::MAX_SEQUENCE_LENGTH {
            return Err(Error::ExceededMaxLen(len));
        }
        self.serialize_u32_as_uleb128(len as u32)
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_u8(v.into())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_u8(v as u8)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_u16(v as u16)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_u32(v as u32)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_i128(self, v: i128) -> Result<()> {
        self.serialize_u128(v as u128)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.push(v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<()> {
        self.output.extend_from_slice(&v.to_le_bytes());
        Ok(())
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(Error::NotSupported("serialize_f32"))
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(Error::NotSupported("serialize_f64"))
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        Err(Error::NotSupported("serialize_char"))
    }

    // Just serialize the string as a raw byte array
    fn serialize_str(self, v: &str) -> Result<()> {
        self.serialize_bytes(v.as_bytes())
    }

    // Serialize a byte array as an array of bytes.
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.serialize_seq_len(v.len())?;
        self.output.extend_from_slice(v);
        Ok(())
    }

    // An absent optional is represented as `00`
    fn serialize_none(self) -> Result<()> {
        self.serialize_u8(0)
    }

    // A present optional is represented as `01` followed by the serialized value
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_u8(1)?;
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.serialize_variant_index(variant_index)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_variant_index(variant_index)?;
        value.serialize(self)
    }

    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which for LCS is either nothing for fixed structures or for variable
    // length structures, the length encoded as a u32.
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if let Some(len) = len {
            self.serialize_seq_len(len)?;
            Ok(self)
        } else {
            Err(Error::MissingLen)
        }
    }

    // Tuples are fixed sized structs so we don't need to encode the length
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_variant_index(variant_index)?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer::new(self))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_variant_index(variant_index)?;
        Ok(self)
    }

    // LCS is not a human readable format
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

#[doc(hidden)]
struct MapSerializer<'a> {
    ser: &'a mut Serializer,
    map: BTreeMap<Vec<u8>, Vec<u8>>,
    next_key: Option<Vec<u8>>,
}

impl<'a> MapSerializer<'a> {
    fn new(ser: &'a mut Serializer) -> Self {
        MapSerializer {
            ser,
            map: BTreeMap::new(),
            next_key: None,
        }
    }
}

impl<'a> ser::SerializeMap for MapSerializer<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.next_key.is_some() {
            return Err(Error::ExpectedMapValue);
        }

        let mut s = Serializer::new();
        key.serialize(&mut s)?;
        self.next_key = Some(s.output);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self.next_key.take() {
            Some(key) => {
                let mut s = Serializer::new();
                value.serialize(&mut s)?;
                self.map.insert(key, s.output);
            }
            None => {
                return Err(Error::ExpectedMapKey);
            }
        }

        Ok(())
    }

    fn end(self) -> Result<()> {
        if self.next_key.is_some() {
            return Err(Error::ExpectedMapValue);
        }

        let len = self.map.len();
        self.ser.serialize_seq_len(len)?;

        for (key, value) in self.map {
            self.ser.output.extend(key.into_iter());
            self.ser.output.extend(value.into_iter());
        }

        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
