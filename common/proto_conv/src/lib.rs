// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The traits `IntoProto` and `FromProto` describes conversion from Rust type to corresponding
//! Protobuf type, or vice versa.

pub mod test_helper;

use failure::prelude::*;
use protobuf::Message;

/// Convert Rust struct to Protobuf.
pub trait IntoProto {
    /// The corresponding Protobuf type.
    type ProtoType;

    /// Converts a Rust struct to corresponding Protobuf struct. This should not fail.
    fn into_proto(self) -> Self::ProtoType;
}

/// Convert Protobuf struct to Rust.
pub trait FromProto: Sized {
    /// The corresponding Protobuf type.
    type ProtoType;

    /// Converts a Protobuf struct to corresponding Rust struct. The conversion could fail if the
    /// Protobuf struct is invalid. For example, the Protobuf struct could have a byte array that
    /// is intended to be a `HashValue`, and the conversion would fail if the byte array is not
    /// exactly 32 bytes.
    fn from_proto(object: Self::ProtoType) -> Result<Self>;
}

pub trait IntoProtoBytes<P> {
    /// Encode a Rust struct to Protobuf bytes.
    fn into_proto_bytes(self) -> Result<Vec<u8>>;
}

/// blanket implementation for `protobuf::Message`.
impl<P, T> IntoProtoBytes<P> for T
where
    P: Message,
    T: IntoProto<ProtoType = P>,
{
    fn into_proto_bytes(self) -> Result<Vec<u8>> {
        Ok(self.into_proto().write_to_bytes()?)
    }
}

pub trait FromProtoBytes<P>: Sized {
    /// Decode a Rust struct from encoded Protobuf bytes.
    fn from_proto_bytes(bytes: &[u8]) -> Result<Self>;
}

/// blanket implementation for `protobuf::Message`.
impl<P, T> FromProtoBytes<P> for T
where
    P: Message,
    T: FromProto<ProtoType = P>,
{
    /// Decode a Rust struct from encoded Protobuf bytes.
    fn from_proto_bytes(bytes: &[u8]) -> Result<Self> {
        Self::from_proto(protobuf::parse_from_bytes(bytes)?)
    }
}

#[cfg(feature = "derive")]
pub use proto_conv_derive::{FromProto, IntoProto};

// For a few types like integers, the Rust type and Protobuf type are identical.
macro_rules! impl_direct_conversion {
    ($type_name: ty) => {
        impl FromProto for $type_name {
            type ProtoType = $type_name;

            fn from_proto(object: Self::ProtoType) -> Result<Self> {
                Ok(object)
            }
        }

        impl IntoProto for $type_name {
            type ProtoType = $type_name;

            fn into_proto(self) -> Self::ProtoType {
                self
            }
        }
    };
}

impl_direct_conversion!(u32);
impl_direct_conversion!(u64);
impl_direct_conversion!(i32);
impl_direct_conversion!(i64);
impl_direct_conversion!(bool);
impl_direct_conversion!(String);
impl_direct_conversion!(Vec<u8>);

// Note: repeated primitive type fields like Vec<u32> are not supported right now, because their
// corresponding protobuf type is Vec<u32>, not protobuf::RepeatedField<u32>.

impl<P, T> FromProto for Vec<T>
where
    P: protobuf::Message,
    T: FromProto<ProtoType = P>,
{
    type ProtoType = protobuf::RepeatedField<P>;

    fn from_proto(object: Self::ProtoType) -> Result<Self> {
        object
            .into_iter()
            .map(T::from_proto)
            .collect::<Result<Vec<_>>>()
    }
}

impl<P, T> IntoProto for Vec<T>
where
    P: protobuf::Message,
    T: IntoProto<ProtoType = P>,
{
    type ProtoType = protobuf::RepeatedField<P>;

    fn into_proto(self) -> Self::ProtoType {
        self.into_iter().map(T::into_proto).collect()
    }
}
