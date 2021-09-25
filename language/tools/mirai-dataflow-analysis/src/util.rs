// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Output,
};

pub type TypeMap = BTreeMap<u32, String>;
pub type NodeMap = BTreeMap<u32, String>;

/// Combine stdout and stderr of a child process
pub fn get_child_output(out: &Output) -> String {
    (String::from_utf8_lossy(&out.stdout) + String::from_utf8_lossy(&out.stderr)).to_string()
}

/// Create an absolute path from a path that is relative to the current directory
pub fn make_absolute(relative_path: &Path) -> Result<PathBuf, String> {
    let mut absolute_path =
        std::env::current_dir().map_err(|msg| format!("Failed to get current path: {}", msg))?;
    absolute_path.push(relative_path);
    Ok(absolute_path)
}
