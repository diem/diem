// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FreezingBit {
    is_frozen: bool,
}

impl FreezingBit {
    pub fn is_frozen(&self) -> bool {
        self.is_frozen
    }
}

impl MoveStructType for FreezingBit {
    const MODULE_NAME: &'static IdentStr = ident_str!("AccountFreezing");
    const STRUCT_NAME: &'static IdentStr = ident_str!("FreezingBit");
}

impl MoveResource for FreezingBit {}
