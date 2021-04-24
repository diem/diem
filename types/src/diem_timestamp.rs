// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DiemTimestampResource {
    pub diem_timestamp: DiemTimestamp,
}

impl MoveStructType for DiemTimestampResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("DiemTimestamp");
    const STRUCT_NAME: &'static IdentStr = ident_str!("CurrentTimeMicroseconds");
}

impl MoveResource for DiemTimestampResource {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiemTimestamp {
    pub microseconds: u64,
}
