// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleId {
    role_id: u64,
}

impl RoleId {
    pub fn role_id(&self) -> u64 {
        self.role_id
    }
}

impl MoveStructType for RoleId {
    const MODULE_NAME: &'static IdentStr = ident_str!("Roles");
    const STRUCT_NAME: &'static IdentStr = ident_str!("RoleId");
}

impl MoveResource for RoleId {}
