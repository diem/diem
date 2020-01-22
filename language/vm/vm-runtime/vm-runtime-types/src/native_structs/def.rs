// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::loaded_data::types::Type;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum NativeStructTag {
    Vector = 0,
}

// TODO: Clean this up when we promote Vector to a primitive type.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct NativeStructType {
    pub tag: NativeStructTag,
    pub type_actuals: Vec<Type>,
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
