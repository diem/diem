// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use move_vm_types::loaded_data::types::FatStructType;
use once_cell::sync::Lazy;
use std::{collections::btree_map::BTreeMap, fs::File, io::Write, path::PathBuf};
use vm_genesis::generate_genesis_type_mapping;

pub const TYPE_MAP_PATH: &str = "move_type_map/mapping.txt";
pub const STAGED_TYPE_MAP_BYTES: &[u8] = std::include_bytes!("../move_type_map/mapping.txt");

static STAGED_TYPE_MAP: Lazy<BTreeMap<Vec<u8>, FatStructType>> = Lazy::new(build_mapping);

pub fn update_mapping() {
    let new_mapping = generate_genesis_type_mapping()
        .into_iter()
        .map(|(k, v)| (hex::encode(&k), v))
        .collect::<Vec<_>>();
    let file_path = PathBuf::from(TYPE_MAP_PATH);
    let mapping_str = serde_json::to_string_pretty(&new_mapping).unwrap();
    let mut module_file = File::create(file_path).unwrap();
    module_file.write_all(&mapping_str.as_bytes()).unwrap();
    module_file.write_all(b"\n").unwrap();
}

pub fn build_mapping() -> BTreeMap<Vec<u8>, FatStructType> {
    serde_json::from_slice::<Vec<(String, FatStructType)>>(STAGED_TYPE_MAP_BYTES)
        .unwrap()
        .into_iter()
        .map(|(k, v)| (hex::decode(k.as_bytes()).unwrap(), v))
        .collect()
}

pub fn resource_vec_to_type_tag(resource_vec: &[u8]) -> Result<FatStructType> {
    STAGED_TYPE_MAP
        .get(resource_vec)
        .cloned()
        .ok_or_else(|| anyhow!("Unknown AccessPath"))
}
