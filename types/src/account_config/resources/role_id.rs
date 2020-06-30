// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::move_resource::MoveResource;
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

impl MoveResource for RoleId {
    const MODULE_NAME: &'static str = "Roles";
    const STRUCT_NAME: &'static str = "RoleId";
}
