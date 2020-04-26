// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use regex::Regex;
use std::{fs::File, io::Read, path::Path};

pub mod baseline_test;

// =================================================================================================
// Constants

pub const DEFAULT_SENDER: &str = "0x8675309";

// =================================================================================================
// Env vars

pub fn read_env_var(v: &str) -> String {
    std::env::var(v).unwrap_or_else(|_| "".into())
}

pub fn read_bool_env_var(v: &str) -> bool {
    let val = read_env_var(v);
    val == "1" || val == "true"
}

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

/// Extracts matches out of some text file. `pat` must be a regular expression with one anonymous
/// group. The list of the content of this group is returned. Use as in in
/// `extract_matches(file, "use 0x0::([a-zA-Z_]+)")`
pub fn extract_matches(path: &Path, pat: &str) -> anyhow::Result<Vec<String>> {
    let rex = Regex::new(&format!("(?m){}", pat))?;
    let mut content = String::new();
    let mut file = File::open(path)?;
    file.read_to_string(&mut content)?;
    let mut at = 0;
    let mut res = vec![];
    while let Some(cap) = rex.captures(&content[at..]) {
        res.push(cap.get(1).unwrap().as_str().to_string());
        at += cap.get(0).unwrap().end();
    }
    Ok(res)
}
