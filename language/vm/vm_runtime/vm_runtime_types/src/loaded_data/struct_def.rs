// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for Move struct definition.

use crate::{loaded_data::types::Type, native_structs::NativeStructType};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Note that this data structure can represent recursive types but will end up creating reference
// cycles, which is bad. Other parts of the system disallow recursive types for now, but this may
// need to be handled more explicitly in the future.
///  Resolved form of struct definition.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
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
