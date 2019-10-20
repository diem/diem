// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for runtime types.

use crate::loaded_data::struct_def::StructDef;
use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "../unit_tests/type_prop_tests.rs"]
mod type_prop_tests;

/// Resolved form of runtime types.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
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
