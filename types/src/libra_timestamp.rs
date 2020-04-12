// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    move_resource::MoveResource,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// The access path where the Libra Timestamp resource is stored.
pub static LIBRA_TIMESTAMP_RESOURCE_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    AccessPath::resource_access_vec(&LibraTimestampResource::struct_tag(), &Accesses::empty())
});

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
