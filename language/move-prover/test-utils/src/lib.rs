// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use regex::Regex;
use std::{fs::File, io::Read, path::Path};

pub mod baseline_test;

// =================================================================================================
// Constants

pub const DEFAULT_SENDER: &str = "0x8675309";

// =================================================================================================
// Extract test annotations out of sources

// Extracts lines out of some text file where each line starts with `start` which can be a regular
// expressions. Returns the list of such lines with `start` stripped. Use as in
// `extract_test_directives(file, "// dep:")`.
pub fn extract_test_directives(path: &Path, start: &str) -> anyhow::Result<Vec<String>> {
    let rex = Regex::new(&format!("(?m)^{}(?P<ann>.*?)$", start)).unwrap();
    let mut content = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut content)?;
    let mut at = 0;
    let mut res = vec![];
    while let Some(cap) = rex.captures(&content[at..]) {
        res.push(cap.name("ann").unwrap().as_str().trim().to_string());
        at += cap.get(0).unwrap().end();
    }
    Ok(res)
}
