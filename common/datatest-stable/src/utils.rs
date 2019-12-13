// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

/// Helper function to iterate through all the files in the given directory, skipping hidden files,
/// and return an iterator of their paths.
pub fn iterate_directory(path: &Path) -> impl Iterator<Item = PathBuf> {
    walkdir::WalkDir::new(path)
        .into_iter()
        .map(::std::result::Result::unwrap)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .file_name()
                    .to_str()
                    .map_or(false, |s| !s.starts_with('.')) // Skip hidden files
        })
        .map(|entry| entry.path().to_path_buf())
}

pub fn derive_test_name(root: &Path, path: &Path, test_name: &str) -> String {
    let relative = path.strip_prefix(root).unwrap_or_else(|_| {
        panic!(
            "failed to strip prefix '{}' from path '{}'",
            root.display(),
            path.display()
        )
    });
    let mut test_name = test_name.to_string();
    test_name.push_str(&format!("::{}", relative.display()));
    test_name
}
