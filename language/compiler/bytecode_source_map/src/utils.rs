// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::source_map::{ModuleSourceMap, SourceMap};
use serde::de::DeserializeOwned;
use std::fs::File;
use std::path::Path;

pub fn module_source_map_from_file<Location>(file_path: &Path) -> ModuleSourceMap<Location>
where
    Location: Clone + Eq + DeserializeOwned,
{
    let file = File::open(file_path).expect("Failed to open source map file");
    serde_json::from_reader(file).expect("Error while reading in source map information")
}

pub fn source_map_from_file<Location>(file_path: &Path) -> SourceMap<Location>
where
    Location: Clone + Eq + DeserializeOwned,
{
    let file = File::open(file_path).expect("Failed to open source map file");
    serde_json::from_reader(file).expect("Error while reading in source map information")
}
