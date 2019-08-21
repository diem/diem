// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use failure::prelude::*;
use std::collections::BTreeMap;

///! In order to guarantee consistency of object representation to byte representation for
///! signature generation and verification, Libra leverages Libra Canonical Serialization (LCS)
///! documented in `README.md`.

/// Interface that all types must implement to support LCS deserialization.
pub trait CanonicalDeserialize {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized;
}

/// Trait for deserializers that implement LCS
pub trait CanonicalDeserializer {
    fn decode_bool(&mut self) -> Result<bool>;

    fn decode_btreemap<K: CanonicalDeserialize + std::cmp::Ord, V: CanonicalDeserialize>(
        &mut self,
    ) -> Result<BTreeMap<K, V>>;

    // decode a byte array with the given length as input
    fn decode_bytes_with_len(&mut self, len: u32) -> Result<Vec<u8>>;

    fn decode_i8(&mut self) -> Result<i8>;

    fn decode_i16(&mut self) -> Result<i16>;

    fn decode_i32(&mut self) -> Result<i32>;

    fn decode_i64(&mut self) -> Result<i64>;

    fn decode_optional<T: CanonicalDeserialize>(&mut self) -> Result<Option<T>>;

    fn decode_string(&mut self) -> Result<String>;

    fn decode_struct<T>(&mut self) -> Result<T>
    where
        T: CanonicalDeserialize,
        Self: Sized,
    {
        T::deserialize(self)
    }

    fn decode_tuple2<T0, T1>(&mut self) -> Result<(T0, T1)>
    where
        Self: Sized,
        T0: CanonicalDeserialize,
        T1: CanonicalDeserialize,
    {
        Ok((T0::deserialize(self)?, T1::deserialize(self)?))
    }

    fn decode_tuple3<T0, T1, T2>(&mut self) -> Result<(T0, T1, T2)>
    where
        Self: Sized,
        T0: CanonicalDeserialize,
        T1: CanonicalDeserialize,
        T2: CanonicalDeserialize,
    {
        Ok((
            T0::deserialize(self)?,
            T1::deserialize(self)?,
            T2::deserialize(self)?,
        ))
    }

    fn decode_u8(&mut self) -> Result<u8>;

    fn decode_u16(&mut self) -> Result<u16>;

    fn decode_u32(&mut self) -> Result<u32>;

    fn decode_u64(&mut self) -> Result<u64>;

    fn decode_variable_length_bytes(&mut self) -> Result<Vec<u8>>;

    fn decode_vec<T: CanonicalDeserialize>(&mut self) -> Result<Vec<T>>;
}

impl CanonicalDeserialize for BTreeMap<Vec<u8>, Vec<u8>> {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(deserializer.decode_btreemap()?)
    }
}

impl CanonicalDeserialize for i8 {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        let num = deserializer.decode_i8()?;
        Ok(num)
    }
}

impl CanonicalDeserialize for i16 {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        let num = deserializer.decode_i16()?;
        Ok(num)
    }
}

impl CanonicalDeserialize for i32 {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        let num = deserializer.decode_i32()?;
        Ok(num)
    }
}

impl CanonicalDeserialize for i64 {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        let num = deserializer.decode_i64()?;
        Ok(num)
    }
}

impl CanonicalDeserialize for String {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(deserializer.decode_string()?)
    }
}

impl<T0, T1> CanonicalDeserialize for (T0, T1)
where
    T0: CanonicalDeserialize,
    T1: CanonicalDeserialize,
{
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        deserializer.decode_tuple2()
    }
}

impl<T0, T1, T2> CanonicalDeserialize for (T0, T1, T2)
where
    T0: CanonicalDeserialize,
    T1: CanonicalDeserialize,
    T2: CanonicalDeserialize,
{
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        deserializer.decode_tuple3()
    }
}

impl CanonicalDeserialize for u8 {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        let num = deserializer.decode_u8()?;
        Ok(num)
    }
}

impl CanonicalDeserialize for u16 {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        deserializer.decode_u16()
    }
}

impl CanonicalDeserialize for u32 {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        deserializer.decode_u32()
    }
}

impl CanonicalDeserialize for u64 {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        let num = deserializer.decode_u64()?;
        Ok(num)
    }
}

impl<T> CanonicalDeserialize for Option<T>
where
    T: CanonicalDeserialize,
{
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        deserializer.decode_optional()
    }
}

/// usize is dependent on architecture. LCS encodes it as a 64-bit unsigned integer. The serializer
/// enforces that usize is smaller than or equal to the largest 64-bit unsigned integer.
impl CanonicalDeserialize for usize {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(deserializer.decode_u64()? as usize)
    }
}

impl<T> CanonicalDeserialize for Vec<T>
where
    T: CanonicalDeserialize,
{
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        deserializer.decode_vec()
    }
}
