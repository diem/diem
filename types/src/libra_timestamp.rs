// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_resource::MoveResource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct LibraTimestampResource {
    pub libra_timestamp: LibraTimestamp,
}

impl MoveResource for LibraTimestampResource {
    const MODULE_NAME: &'static str = "LibraTimestamp";
    const STRUCT_NAME: &'static str = "CurrentTimeMicroseconds";
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LibraTimestamp {
    pub microseconds: u64,
}
