// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use move_core_types::language_storage::StructTag;
use once_cell::sync::Lazy;
use std::collections::btree_map::BTreeMap;

pub const PACKED_TYPE_MAP_BYTES: &[u8] =
    std::include_bytes!("../../../stdlib/compiled/packed_types/packed_types.txt");

static STAGED_TYPE_MAP: Lazy<BTreeMap<Vec<u8>, StructTag>> = Lazy::new(build_mapping);

pub(crate) fn build_mapping() -> BTreeMap<Vec<u8>, StructTag> {
    serde_json::from_slice::<Vec<(String, StructTag)>>(PACKED_TYPE_MAP_BYTES)
        .unwrap()
        .into_iter()
        .map(|(k, v)| (hex::decode(k.as_bytes()).unwrap(), v))
        .collect()
}

pub(crate) fn resource_vec_to_type_tag(resource_vec: &[u8]) -> Result<StructTag> {
    STAGED_TYPE_MAP
        .get(resource_vec)
        .cloned()
        .ok_or_else(|| anyhow!("Unknown AccessPath"))
}
