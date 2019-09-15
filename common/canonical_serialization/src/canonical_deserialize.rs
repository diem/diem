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

    fn decode_bytes(&mut self) -> Result<Vec<u8>>;

    fn decode_i8(&mut self) -> Result<i8>;

    fn decode_i16(&mut self) -> Result<i16>;

    fn decode_i32(&mut self) -> Result<i32>;

    fn decode_i64(&mut self) -> Result<i64>;

    fn decode_string(&mut self) -> Result<String>;

    fn decode_u8(&mut self) -> Result<u8>;

    fn decode_u16(&mut self) -> Result<u16>;

    fn decode_u32(&mut self) -> Result<u32>;

    fn decode_u64(&mut self) -> Result<u64>;

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

    fn decode_btreemap<K: CanonicalDeserialize + std::cmp::Ord, V: CanonicalDeserialize>(
        &mut self,
    ) -> Result<BTreeMap<K, V>>;

    fn decode_optional<T: CanonicalDeserialize>(&mut self) -> Result<Option<T>>;

    fn decode_struct<T>(&mut self) -> Result<T>
    where
        T: CanonicalDeserialize,
        Self: Sized,
    {
        T::deserialize(self)
    }

    fn decode_vec<T: CanonicalDeserialize>(&mut self) -> Result<Vec<T>>;
}

macro_rules! impl_canonical_deserialize {
    ($function:ident, $type:ty) => {
        impl CanonicalDeserialize for $type {
            fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<(Self)> {
                deserializer.$function()
            }
        }
    };
}

macro_rules! impl_canonical_deserialize_for_tuple {
    ($function:ident, $($type:ident)+) => (
        impl<$($type), +> CanonicalDeserialize for ($($type), +)
        where
            $($type: CanonicalDeserialize,) +
        {
            fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<(Self)>
            where
                Self: Sized,
            {
                deserializer.$function()
            }
        }
    );
}

impl_canonical_deserialize!(decode_bool, bool);
impl_canonical_deserialize!(decode_btreemap, BTreeMap<Vec<u8>, Vec<u8>>);
impl_canonical_deserialize!(decode_i8, i8);
impl_canonical_deserialize!(decode_i16, i16);
impl_canonical_deserialize!(decode_i32, i32);
impl_canonical_deserialize!(decode_i64, i64);
impl_canonical_deserialize!(decode_string, String);
impl_canonical_deserialize_for_tuple!(decode_tuple2, T0 T1);
impl_canonical_deserialize_for_tuple!(decode_tuple3, T0 T1 T2);
impl_canonical_deserialize!(decode_u8, u8);
impl_canonical_deserialize!(decode_u16, u16);
impl_canonical_deserialize!(decode_u32, u32);
impl_canonical_deserialize!(decode_u64, u64);

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
