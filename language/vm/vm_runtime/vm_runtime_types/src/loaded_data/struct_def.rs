// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for Move struct definition.

use crate::loaded_data::types::Type;
use canonical_serialization::*;
use failure::prelude::*;
use std::sync::Arc;

// Note that this data structure can represent recursive types but will end up creating reference
// cycles, which is bad. Other parts of the system disallow recursive types for now, but this may
// need to be handled more explicitly in the future.
///  Resolved form of struct definition.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct StructDef(Arc<StructDefInner>);

impl StructDef {
    /// Constructs a new [`StructDef`]
    pub fn new(field_definitions: Vec<Type>) -> Self {
        Self(Arc::new(StructDefInner { field_definitions }))
    }

    /// Get type declaration for each field in the struct.
    #[inline]
    pub fn field_definitions(&self) -> &[Type] {
        &self.0.field_definitions
    }
}

// Do not implement Clone for this -- the outer StructDef should be Arc'd.
#[derive(Debug, Eq, PartialEq)]
struct StructDefInner {
    field_definitions: Vec<Type>,
}

/// This isn't used by any normal code at the moment, but is used by the fuzzer to serialize types
/// alongside values.
impl CanonicalSerialize for StructDef {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        let field_defs = self.field_definitions();
        // Encode the number of field definitions, then the definitions themselves.
        field_defs.len().serialize(serializer)?;
        for field_def in field_defs {
            field_def.serialize(serializer)?;
        }
        Ok(())
    }
}

impl CanonicalDeserialize for StructDef {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self>
    where
        Self: Sized,
    {
        // Libra only runs on 64-bit machines.
        let num_defs = deserializer.decode_u64()? as usize;
        let field_defs: Result<_> = (0..num_defs)
            .map(|_| Type::deserialize(deserializer))
            .collect();
        Ok(StructDef::new(field_defs?))
    }
}
