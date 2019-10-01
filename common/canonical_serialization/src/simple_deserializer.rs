// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    canonical_deserialize::{CanonicalDeserialize, CanonicalDeserializer},
    Endianness, ARRAY_MAX_LENGTH,
};
use byteorder::ReadBytesExt;
use failure::prelude::*;
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
};

/// An implementation of LCS deserializer (CanonicalDeserialize) for [u8].
#[derive(Clone)]
pub struct SimpleDeserializer<'a> {
    raw_bytes: Cursor<&'a [u8]>,
}

impl<'a> SimpleDeserializer<'a> {
    pub fn new<T>(raw_bytes: &'a T) -> Self
    where
        T: AsRef<[u8]> + ?Sized,
    {
        Self {
            raw_bytes: Cursor::new(raw_bytes.as_ref()),
        }
    }

    pub fn deserialize<T>(data: &'a [u8]) -> Result<T>
    where
        T: CanonicalDeserialize,
    {
        let mut deserializer = Self::new(data);
        T::deserialize(&mut deserializer)
    }

    /// Returns true if the deserializer has no remaining bytes to deserialize
    pub fn is_empty(&self) -> bool {
        (self.len() as u64) - self.position() == 0
    }

    /// Returns the total length of the underlying bytes used by the deserializer
    pub fn len(&self) -> usize {
        self.raw_bytes.get_ref().len()
    }

    /// Returns the current index into the bytes used by the deserializer
    pub fn position(&self) -> u64 {
        self.raw_bytes.position()
    }
}

impl<'a> CanonicalDeserializer for SimpleDeserializer<'a> {
    fn decode_bool(&mut self) -> Result<bool> {
        let b = self.raw_bytes.read_u8()?;
        ensure!(b == 0 || b == 1, "bool must be 0 or 1, found {}", b,);
        Ok(b != 0)
    }

    fn decode_bytes(&mut self) -> Result<Vec<u8>> {
        let len = self.decode_u32()?;
        ensure!(
            len as usize <= ARRAY_MAX_LENGTH,
            "array length longer than max allowed length. len: {}, max: {}",
            len,
            ARRAY_MAX_LENGTH
        );

        // make sure there is enough bytes left in the buffer
        let remain = self.raw_bytes.get_ref().len() - self.raw_bytes.position() as usize;
        ensure!(
            remain >= (len as usize),
            "not enough bytes left. len: {}, remaining: {}",
            len,
            remain
        );

        let mut vec = vec![0; len as usize];
        self.raw_bytes.read_exact(&mut vec)?;
        Ok(vec)
    }

    fn decode_i8(&mut self) -> Result<i8> {
        Ok(self.raw_bytes.read_i8()?)
    }

    fn decode_i16(&mut self) -> Result<i16> {
        Ok(self.raw_bytes.read_i16::<Endianness>()?)
    }

    fn decode_i32(&mut self) -> Result<i32> {
        Ok(self.raw_bytes.read_i32::<Endianness>()?)
    }

    fn decode_i64(&mut self) -> Result<i64> {
        Ok(self.raw_bytes.read_i64::<Endianness>()?)
    }

    fn decode_string(&mut self) -> Result<String> {
        Ok(String::from_utf8(self.decode_bytes()?)?)
    }

    fn decode_u8(&mut self) -> Result<u8> {
        Ok(self.raw_bytes.read_u8()?)
    }

    fn decode_u16(&mut self) -> Result<u16> {
        Ok(self.raw_bytes.read_u16::<Endianness>()?)
    }

    fn decode_u32(&mut self) -> Result<u32> {
        Ok(self.raw_bytes.read_u32::<Endianness>()?)
    }

    fn decode_u64(&mut self) -> Result<u64> {
        Ok(self.raw_bytes.read_u64::<Endianness>()?)
    }

    fn decode_btreemap<K: CanonicalDeserialize + std::cmp::Ord, V: CanonicalDeserialize>(
        &mut self,
    ) -> Result<BTreeMap<K, V>> {
        let len = self.decode_u32()?;
        ensure!(
            len as usize <= ARRAY_MAX_LENGTH,
            "array length longer than max allowed. size: {}, max: {}",
            len,
            ARRAY_MAX_LENGTH
        );

        let mut map = BTreeMap::new();
        for _i in 0..len {
            let key = K::deserialize(self)?;
            let value = V::deserialize(self)?;
            map.insert(key, value);
        }
        Ok(map)
    }

    fn decode_optional<T: CanonicalDeserialize>(&mut self) -> Result<Option<T>> {
        if self.decode_bool()? {
            Ok(Some(T::deserialize(self)?))
        } else {
            Ok(None)
        }
    }

    fn decode_vec<T: CanonicalDeserialize>(&mut self) -> Result<Vec<T>> {
        let len = self.decode_u32()?;
        ensure!(
            len as usize <= ARRAY_MAX_LENGTH,
            "array length longer than max allowed. size: {}, max: {}",
            len,
            ARRAY_MAX_LENGTH
        );

        let mut vec = Vec::new();
        for _i in 0..len {
            let v = T::deserialize(self)?;
            vec.push(v);
        }
        Ok(vec)
    }
}
