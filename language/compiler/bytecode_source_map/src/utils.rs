// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::source_map::{ModuleSourceMap, SourceMap};
use std::fs::File;
use std::path::Path;

pub fn module_source_map_from_file(filename: &str) -> ModuleSourceMap {
    let file_path = Path::new(filename);
    let file = File::open(file_path).expect("Failed to open source map file");
    serde_json::from_reader(file).expect("Error while reading in source map information")
}

pub fn source_map_from_file(filename: &str) -> SourceMap {
    let file_path = Path::new(filename);
    let file = File::open(file_path).expect("Failed to open source map file");
    serde_json::from_reader(file).expect("Error while reading in source map information")
}

//fn display(source_str: &str, span: Loc) { }
