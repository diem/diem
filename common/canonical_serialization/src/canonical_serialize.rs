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
    fn encode_bool(&mut self, v: bool) -> Result<&mut Self>;

    fn encode_bytes(&mut self, v: &[u8]) -> Result<&mut Self>;

    fn encode_i8(&mut self, v: i8) -> Result<&mut Self>;

    fn encode_i16(&mut self, v: i16) -> Result<&mut Self>;

    fn encode_i32(&mut self, v: i32) -> Result<&mut Self>;

    fn encode_i64(&mut self, v: i64) -> Result<&mut Self>;

    fn encode_string(&mut self, v: &str) -> Result<&mut Self>;

    fn encode_u8(&mut self, v: u8) -> Result<&mut Self>;

    fn encode_u16(&mut self, v: u16) -> Result<&mut Self>;

    fn encode_u32(&mut self, v: u32) -> Result<&mut Self>;

    fn encode_u64(&mut self, v: u64) -> Result<&mut Self>;

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

    fn encode_btreemap<K: CanonicalSerialize, V: CanonicalSerialize>(
        &mut self,
        v: &BTreeMap<K, V>,
    ) -> Result<&mut Self>;

    fn encode_optional<T: CanonicalSerialize>(&mut self, v: &Option<T>) -> Result<&mut Self>;

    fn encode_struct(&mut self, structure: &impl CanonicalSerialize) -> Result<&mut Self>
    where
        Self: std::marker::Sized,
    {
        structure.serialize(self)?;
        Ok(self)
    }

    fn encode_vec<T: CanonicalSerialize>(&mut self, v: &[T]) -> Result<&mut Self>;
}

macro_rules! impl_canonical_serialize_for_complex {
    ($function:ident, $type:ty) => {
        impl CanonicalSerialize for $type {
            fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
                serializer.$function(self)?;
                Ok(())
            }
        }
    };
}

macro_rules! impl_canonical_serialize_for_primitive {
    ($function:ident, $type:ty) => {
        impl CanonicalSerialize for $type {
            fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
                serializer.$function(*self)?;
                Ok(())
            }
        }
    };
}

macro_rules! impl_canonical_serialize_for_tuple {
    ($function:ident,$($type:ident)+) => (
        impl<$($type), +> CanonicalSerialize for ($($type),+)
        where
            $($type: CanonicalSerialize,) +
        {
            fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
                serializer.$function(self)?;
                Ok(())
            }
        }
    );
}

impl_canonical_serialize_for_primitive!(encode_bool, bool);
impl_canonical_serialize_for_complex!(encode_btreemap, BTreeMap<Vec<u8>, Vec<u8>>);
impl_canonical_serialize_for_primitive!(encode_i8, i8);
impl_canonical_serialize_for_primitive!(encode_i16, i16);
impl_canonical_serialize_for_primitive!(encode_i32, i32);
impl_canonical_serialize_for_primitive!(encode_i64, i64);
impl_canonical_serialize_for_complex!(encode_string, &str);
impl_canonical_serialize_for_tuple!(encode_tuple2, T0 T1);
impl_canonical_serialize_for_tuple!(encode_tuple3, T0 T1 T2);
impl_canonical_serialize_for_primitive!(encode_u8, u8);
impl_canonical_serialize_for_primitive!(encode_u16, u16);
impl_canonical_serialize_for_primitive!(encode_u32, u32);
impl_canonical_serialize_for_primitive!(encode_u64, u64);

impl<T> CanonicalSerialize for Option<T>
where
    T: CanonicalSerialize,
{
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_optional(self)?;
        Ok(())
    }
}

impl CanonicalSerialize for String {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_string(self.as_str())?;
        Ok(())
    }
}

/// usize is architecture dependent, LCS encodes it as a 64-bit unsigned integer and fails
/// if usize is larger than a 64-bit integer.
impl CanonicalSerialize for usize {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        ensure!(
            *self <= u64::max_value() as usize,
            "usize bigger than max allowed. Expected <= {}, found: {}",
            u64::max_value(),
            *self,
        );
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
