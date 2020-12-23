// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use diem_types::access_path::Path;
use move_core_types::language_storage::StructTag;

pub(crate) fn resource_vec_to_type_tag(resource_vec: &[u8]) -> Result<StructTag> {
    let path = bcs::from_bytes::<Path>(resource_vec)?;
    match path {
        Path::Resource(s) => Ok(s),
        Path::Code(_) => Err(anyhow!("Unexpected module path")),
    }
}
