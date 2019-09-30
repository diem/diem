// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for Move struct definition.

use crate::{loaded_data::types::Type, native_structs::NativeStructType};
use failure::prelude::*;
use libra_canonical_serialization::*;
use std::sync::Arc;

// Note that this data structure can represent recursive types but will end up creating reference
// cycles, which is bad. Other parts of the system disallow recursive types for now, but this may
// need to be handled more explicitly in the future.
///  Resolved form of struct definition.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum StructDef {
    Struct(Arc<StructDefInner>),
    Native(NativeStructType),
}

impl StructDef {
    /// Constructs a new [`StructDef`]
    pub fn new(field_definitions: Vec<Type>) -> Self {
        StructDef::Struct(Arc::new(StructDefInner { field_definitions }))
    }
}

// Do not implement Clone for this -- the outer StructDef should be Arc'd.
#[derive(Debug, Eq, PartialEq)]
pub struct StructDefInner {
    field_definitions: Vec<Type>,
}

impl StructDefInner {
    /// Get type declaration for each field in the struct.
    #[inline]
    pub fn field_definitions(&self) -> &[Type] {
        &self.field_definitions
    }
}

/// This isn't used by any normal code at the moment, but is used by the fuzzer to serialize types
/// alongside values.
impl CanonicalSerialize for StructDef {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        match self {
            StructDef::Struct(s) => {
                serializer.encode_u8(0x0)?;
                // Encode the number of field definitions, then the definitions themselves.
                s.field_definitions.len().serialize(serializer)?;
                for field_def in s.field_definitions.iter() {
                    field_def.serialize(serializer)?;
                }
            }
            StructDef::Native(n) => {
                serializer.encode_u8(0x1)?;
                serializer.encode_struct(n)?;
            }
        }
        Ok(())
    }
}

impl CanonicalDeserialize for StructDef {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        match deserializer.decode_u8()? {
            0x0 => {
                // Libra only runs on 64-bit machines.
                let num_defs = deserializer.decode_u64()? as usize;
                let field_defs: Result<_> = (0..num_defs)
                    .map(|_| Type::deserialize(deserializer))
                    .collect();
                Ok(StructDef::new(field_defs?))
            }
            0x1 => Ok(StructDef::Native(deserializer.decode_struct()?)),
            _ => bail!("Can't deserialize tag fot StructDef"),
        }
    }
}
