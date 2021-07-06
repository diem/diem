// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail};
use std::path::Path;

/// Extension for Move source language files
pub const MOVE_EXTENSION: &str = "move";
/// Extension for Move IR files
pub const MOVE_IR_EXTENSION: &str = "mvir";
/// Extension for Move bytecode files
pub const MOVE_COMPILED_EXTENSION: &str = "mv";
/// Extension for Move source map files (mappings from source to bytecode)
pub const SOURCE_MAP_EXTENSION: &str = "mvsm";
/// Extension for error description map for compiled releases
pub const MOVE_ERROR_DESC_EXTENSION: &str = "errmap";

/// - For each directory in `paths`, it will return all files that satisfy the predicate
/// - Any file explicitly passed in `paths`, it will include that file in the result, regardless
///   of the file extension
pub fn find_filenames<Predicate: FnMut(&Path) -> bool>(
    paths: &[impl AsRef<Path>],
    mut is_file_desired: Predicate,
) -> anyhow::Result<Vec<String>> {
    let mut result = vec![];

    for s in paths {
        let path = s.as_ref();
        if !path.exists() {
            bail!("No such file or directory '{}'", path.to_string_lossy())
        }
        if path.is_file() && is_file_desired(path) {
            result.push(path_to_string(path)?);
            continue;
        }
        if !path.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if !entry.file_type().is_file() || !is_file_desired(&entry_path) {
                continue;
            }

            result.push(path_to_string(entry_path)?);
        }
    }
    Ok(result)
}

/// - For each directory in `paths`, it will return all files with the `MOVE_EXTENSION` found
///   recursively in that directory
/// - If `keep_specified_files` any file explicitly passed in `paths`, will be added to the result
///   Otherwise, they will be discarded
pub fn find_move_filenames(
    paths: &[impl AsRef<Path>],
    keep_specified_files: bool,
) -> anyhow::Result<Vec<String>> {
    if keep_specified_files {
        let (file_paths, other_paths): (Vec<&Path>, Vec<&Path>) =
            paths.iter().map(|p| p.as_ref()).partition(|s| s.is_file());
        let mut files = file_paths
            .into_iter()
            .map(path_to_string)
            .collect::<anyhow::Result<Vec<String>>>()?;
        files.extend(find_filenames(&other_paths, |path| {
            extension_equals(path, MOVE_EXTENSION)
        })?);
        Ok(files)
    } else {
        find_filenames(paths, |path| extension_equals(path, MOVE_EXTENSION))
    }
}

pub fn path_to_string(path: &Path) -> anyhow::Result<String> {
    match path.to_str() {
        Some(p) => Ok(p.to_string()),
        None => Err(anyhow!("non-Unicode file name")),
    }
}

pub fn extension_equals(path: &Path, target_ext: &str) -> bool {
    match path.extension().and_then(|s| s.to_str()) {
        Some(extension) => extension == target_ext,
        None => false,
    }
}
