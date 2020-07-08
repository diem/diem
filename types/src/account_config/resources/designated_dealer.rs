// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

// TODO: add fields
#[derive(Debug, Serialize, Deserialize)]
pub struct DesignatedDealer {}

impl MoveResource for DesignatedDealer {
    const MODULE_NAME: &'static str = "DesignatedDealer";
    const STRUCT_NAME: &'static str = "Dealer";
}
