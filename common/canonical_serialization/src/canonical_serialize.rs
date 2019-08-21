// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use failure::prelude::*;
use std::collections::BTreeMap;

///! In order to guarantee consistency of object representation to byte representation for
///! signature generation and verification, Libra leverages Libra Canonical Serialization (LCS)
///! documented in `README.md`.

/// Interface that all types must implement to support LCS serialization.
pub trait CanonicalSerialize {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()>;
}

/// Trait for serializers that implement LCS
pub trait CanonicalSerializer {
    fn encode_bool(&mut self, b: bool) -> Result<&mut Self>;

    fn encode_btreemap<K: CanonicalSerialize, V: CanonicalSerialize>(
        &mut self,
        v: &BTreeMap<K, V>,
    ) -> Result<&mut Self>;

    fn encode_i8(&mut self, v: i8) -> Result<&mut Self>;

    fn encode_i16(&mut self, v: i16) -> Result<&mut Self>;

    fn encode_i32(&mut self, v: i32) -> Result<&mut Self>;

    fn encode_i64(&mut self, v: i64) -> Result<&mut Self>;

    fn encode_optional<T: CanonicalSerialize>(&mut self, v: &Option<T>) -> Result<&mut Self>;

    // Use this encoder when the length of the array is known to be fixed and always known at
    // deserialization time. The raw bytes of the array without length prefix are encoded.
    // For deserialization, use decode_bytes_with_len() which requires giving the length
    // as input
    fn encode_raw_bytes(&mut self, bytes: &[u8]) -> Result<&mut Self>;

    fn encode_string(&mut self, s: &str) -> Result<&mut Self>;

    fn encode_struct(&mut self, structure: &impl CanonicalSerialize) -> Result<&mut Self>
    where
        Self: std::marker::Sized,
    {
        structure.serialize(self)?;
        Ok(self)
    }

    fn encode_tuple2<T0, T1>(&mut self, v: &(T0, T1)) -> Result<&mut Self>
    where
        Self: Sized,
        T0: CanonicalSerialize,
        T1: CanonicalSerialize,
    {
        v.0.serialize(self)?;
        v.1.serialize(self)?;
        Ok(self)
    }

    fn encode_tuple3<T0, T1, T2>(&mut self, v: &(T0, T1, T2)) -> Result<&mut Self>
    where
        Self: Sized,
        T0: CanonicalSerialize,
        T1: CanonicalSerialize,
        T2: CanonicalSerialize,
    {
        v.0.serialize(self)?;
        v.1.serialize(self)?;
        v.2.serialize(self)?;
        Ok(self)
    }

    fn encode_u8(&mut self, v: u8) -> Result<&mut Self>;

    fn encode_u16(&mut self, v: u16) -> Result<&mut Self>;

    fn encode_u32(&mut self, v: u32) -> Result<&mut Self>;

    fn encode_u64(&mut self, v: u64) -> Result<&mut Self>;

    // Use this encoder to encode variable length byte arrays whose length may not be known at
    // deserialization time.
    fn encode_variable_length_bytes(&mut self, v: &[u8]) -> Result<&mut Self>;

    fn encode_vec<T: CanonicalSerialize>(&mut self, v: &[T]) -> Result<&mut Self>;
}

impl CanonicalSerialize for BTreeMap<Vec<u8>, Vec<u8>> {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_btreemap(self)?;
        Ok(())
    }
}

impl CanonicalSerialize for i8 {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_i8(*self)?;
        Ok(())
    }
}

impl CanonicalSerialize for i16 {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_i16(*self)?;
        Ok(())
    }
}

impl CanonicalSerialize for i32 {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_i32(*self)?;
        Ok(())
    }
}

impl CanonicalSerialize for i64 {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_i64(*self)?;
        Ok(())
    }
}

impl<T> CanonicalSerialize for Option<T>
where
    T: CanonicalSerialize,
{
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_optional(self)?;
        Ok(())
    }
}

impl CanonicalSerialize for &str {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_string(self)?;
        Ok(())
    }
}

impl CanonicalSerialize for String {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_string(self.as_str())?;
        Ok(())
    }
}

impl<T0, T1> CanonicalSerialize for (T0, T1)
where
    T0: CanonicalSerialize,
    T1: CanonicalSerialize,
{
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_tuple2(self)?;
        Ok(())
    }
}

impl<T0, T1, T2> CanonicalSerialize for (T0, T1, T2)
where
    T0: CanonicalSerialize,
    T1: CanonicalSerialize,
    T2: CanonicalSerialize,
{
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_tuple3(self)?;
        Ok(())
    }
}

impl CanonicalSerialize for u8 {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_u8(*self)?;
        Ok(())
    }
}

impl CanonicalSerialize for u16 {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_u16(*self)?;
        Ok(())
    }
}

impl CanonicalSerialize for u32 {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_u32(*self)?;
        Ok(())
    }
}

impl CanonicalSerialize for u64 {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_u64(*self)?;
        Ok(())
    }
}

/// usize is architecture dependent, LCS encodes it as a 64-bit unsigned integer and fails
/// if usize is larger than a 64-bit integer.
impl CanonicalSerialize for usize {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_u64(*self as u64)?;
        Ok(())
    }
}

impl<T> CanonicalSerialize for Vec<T>
where
    T: CanonicalSerialize,
{
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_vec(self.as_ref())?;
        Ok(())
    }
}
