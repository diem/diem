// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    canonical_serialize::{CanonicalSerialize, CanonicalSerializer},
    Endianness, ARRAY_MAX_LENGTH,
};
use byteorder::WriteBytesExt;
use failure::prelude::*;
use std::collections::BTreeMap;

/// An implementation of LCS serializer (CanonicalSerializer) for std::io::Write, which includes
/// Vec<u8>.
#[derive(Clone)]
pub struct SimpleSerializer<W> {
    output: W,
}

impl<W> Default for SimpleSerializer<W>
where
    W: Default + std::io::Write,
{
    fn default() -> Self {
        SimpleSerializer::new()
    }
}

impl<W> SimpleSerializer<W>
where
    W: Default + std::io::Write,
{
    pub fn new() -> Self {
        SimpleSerializer {
            output: W::default(),
        }
    }

    /// Create a SimpleSerializer on the fly and serialize `object`
    pub fn serialize(object: &impl CanonicalSerialize) -> Result<W> {
        let mut serializer = Self::default();
        object.serialize(&mut serializer)?;
        Ok(serializer.get_output())
    }

    /// Consume the SimpleSerializer and return the output
    pub fn get_output(self) -> W {
        self.output
    }
}

impl<W> CanonicalSerializer for SimpleSerializer<W>
where
    W: std::io::Write,
{
    fn encode_bool(&mut self, b: bool) -> Result<&mut Self> {
        let byte: u8 = if b { 1 } else { 0 };
        self.output.write_u8(byte)?;
        Ok(self)
    }

    fn encode_bytes(&mut self, v: &[u8]) -> Result<&mut Self> {
        ensure!(
            v.len() <= ARRAY_MAX_LENGTH,
            "array length exceeded the maximum length limit. \
             length: {}, Max length limit: {}",
            v.len(),
            ARRAY_MAX_LENGTH,
        );

        // first add the length as a 4-byte integer
        self.encode_u32(v.len() as u32)?;
        self.output.write_all(v)?;
        Ok(self)
    }

    fn encode_i8(&mut self, v: i8) -> Result<&mut Self> {
        self.output.write_i8(v)?;
        Ok(self)
    }

    fn encode_i16(&mut self, v: i16) -> Result<&mut Self> {
        self.output.write_i16::<Endianness>(v)?;
        Ok(self)
    }

    fn encode_i32(&mut self, v: i32) -> Result<&mut Self> {
        self.output.write_i32::<Endianness>(v)?;
        Ok(self)
    }

    fn encode_i64(&mut self, v: i64) -> Result<&mut Self> {
        self.output.write_i64::<Endianness>(v)?;
        Ok(self)
    }

    fn encode_string(&mut self, s: &str) -> Result<&mut Self> {
        // String::as_bytes returns the UTF-8 encoded byte array
        self.encode_bytes(s.as_bytes())
    }

    fn encode_u8(&mut self, v: u8) -> Result<&mut Self> {
        self.output.write_u8(v)?;
        Ok(self)
    }

    fn encode_u16(&mut self, v: u16) -> Result<&mut Self> {
        self.output.write_u16::<Endianness>(v)?;
        Ok(self)
    }

    fn encode_u32(&mut self, v: u32) -> Result<&mut Self> {
        self.output.write_u32::<Endianness>(v)?;
        Ok(self)
    }

    fn encode_u64(&mut self, v: u64) -> Result<&mut Self> {
        self.output.write_u64::<Endianness>(v)?;
        Ok(self)
    }

    fn encode_tuple_iterator<K: CanonicalSerialize, V: CanonicalSerialize, I>(
        &mut self,
        iter: I,
    ) -> Result<&mut Self>
    where
        I: Iterator<Item = (K, V)>,
    {
        let mut map = BTreeMap::new();

        // Regardless of the order defined for K of the map, write in the order of the lexicographic
        // order of the canonical serialized bytes of K
        for (key, value) in iter {
            map.insert(
                SimpleSerializer::<Vec<u8>>::serialize(&key)?,
                SimpleSerializer::<Vec<u8>>::serialize(&value)?,
            );
        }

        ensure!(
            map.len() <= ARRAY_MAX_LENGTH,
            "array length exceeded the maximum limit. length: {}, max length limit: {}",
            map.len(),
            ARRAY_MAX_LENGTH,
        );

        // add the number of pairs in the map
        self.encode_u32(map.len() as u32)?;

        for (key, value) in map {
            self.output.write_all(key.as_ref())?;
            self.output.write_all(value.as_ref())?;
        }
        Ok(self)
    }

    fn encode_optional<T: CanonicalSerialize>(&mut self, v: &Option<T>) -> Result<&mut Self> {
        match v.as_ref() {
            Some(val) => {
                self.encode_bool(true)?;
                self.encode_struct(val)?;
            }
            None => {
                self.encode_bool(false)?;
            }
        }
        Ok(self)
    }

    fn encode_vec<T: CanonicalSerialize>(&mut self, v: &[T]) -> Result<&mut Self> {
        ensure!(
            v.len() <= ARRAY_MAX_LENGTH,
            "array length exceeded the maximum limit. length: {}, max length limit: {}",
            v.len(),
            ARRAY_MAX_LENGTH,
        );

        // add the number of items in the vec
        self.encode_u32(v.len() as u32)?;
        for value in v {
            self.encode_struct(value)?;
        }
        Ok(self)
    }
}
