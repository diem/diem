// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::{struct_def::StructDef, types::Type},
    native_structs::vector::NativeVector,
};
use serde::{ser, Deserialize, Serialize};
use vm::gas_schedule::{AbstractMemorySize, GasCarrier};

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum NativeStructTag {
    Vector = 0,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct NativeStructType {
    pub tag: NativeStructTag,
    type_actuals: Vec<Type>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NativeStructValue {
    Vector(NativeVector),
}

// TODO(#1307)
impl ser::Serialize for NativeStructValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match self {
            NativeStructValue::Vector(v) => v.serialize(serializer),
        }
    }
}

impl NativeStructType {
    pub fn new(tag: NativeStructTag, type_actuals: Vec<Type>) -> Self {
        Self { tag, type_actuals }
    }
    pub fn new_vec(ty: Type) -> Self {
        Self {
            tag: NativeStructTag::Vector,
            type_actuals: vec![ty],
        }
    }

    pub fn type_actuals(&self) -> &[Type] {
        &self.type_actuals
    }
}

impl NativeStructValue {
    pub fn size(&self) -> AbstractMemorySize<GasCarrier> {
        match self {
            NativeStructValue::Vector(v) => v.size(),
        }
    }

    /// Normal code should always know what type this value has. This is made available only for
    /// tests.
    #[allow(non_snake_case)]
    #[doc(hidden)]
    pub(crate) fn to_struct_def_FOR_TESTING(&self) -> StructDef {
        match self {
            NativeStructValue::Vector(v) => StructDef::Native(NativeStructType::new_vec(
                v.get(0)
                    .map(|v| v.to_type_FOR_TESTING())
                    .unwrap_or(Type::Bool),
            )),
        }
    }
}
