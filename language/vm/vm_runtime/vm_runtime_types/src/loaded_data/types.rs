// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for runtime types.

use crate::loaded_data::struct_def::StructDef;
use canonical_serialization::*;
use failure::prelude::*;

#[cfg(test)]
#[path = "../unit_tests/type_prop_tests.rs"]
mod type_prop_tests;

/// Resolved form of runtime types.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Type {
    Bool,
    U64,
    String,
    ByteArray,
    Address,
    Struct(StructDef),
    Reference(Box<Type>),
    MutableReference(Box<Type>),
    TypeVariable(u16),
}

/// This isn't used by any normal code at the moment, but is used by the fuzzer to serialize types
/// alongside values.
impl CanonicalSerialize for Type {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        use Type::*;

        // Add a type for each tag.
        let _: &mut _ = match self {
            Bool => serializer.encode_u8(0x01)?,
            U64 => serializer.encode_u8(0x02)?,
            String => serializer.encode_u8(0x03)?,
            ByteArray => serializer.encode_u8(0x04)?,
            Address => serializer.encode_u8(0x05)?,
            Struct(struct_def) => {
                serializer.encode_u8(0x06)?;
                struct_def.serialize(serializer)?;
                serializer
            }
            Reference(ty) => {
                serializer.encode_u8(0x07)?;
                ty.serialize(serializer)?;
                serializer
            }
            MutableReference(ty) => {
                serializer.encode_u8(0x08)?;
                ty.serialize(serializer)?;
                serializer
            }
            TypeVariable(idx) => {
                serializer.encode_u8(0x09)?;
                serializer.encode_u16(*idx)?;
                serializer
            }
        };
        Ok(())
    }
}

impl CanonicalDeserialize for Type {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        use Type::*;

        let ty = match deserializer.decode_u8()? {
            0x01 => Bool,
            0x02 => U64,
            0x03 => String,
            0x04 => ByteArray,
            0x05 => Address,
            0x06 => Struct(StructDef::deserialize(deserializer)?),
            0x07 => Reference(Box::new(Type::deserialize(deserializer)?)),
            0x08 => MutableReference(Box::new(Type::deserialize(deserializer)?)),
            0x09 => TypeVariable(u16::deserialize(deserializer)?),
            other => bail!(
                "Error while deserializing type: found unexpected tag {:#x}",
                other
            ),
        };
        Ok(ty)
    }
}
