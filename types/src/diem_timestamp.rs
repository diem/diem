// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DiemTimestampResource {
    pub diem_timestamp: DiemTimestamp,
}

impl MoveResource for DiemTimestampResource {
    const MODULE_NAME: &'static str = "DiemTimestamp";
    const STRUCT_NAME: &'static str = "CurrentTimeMicroseconds";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiemTimestamp {
    pub microseconds: u64,
}
