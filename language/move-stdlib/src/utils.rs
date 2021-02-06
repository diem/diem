// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

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

pub fn time_it<F>(msg: &str, mut f: F)
where
    F: FnMut(),
{
    let now = Instant::now();
    print!("{} ... ", msg);
    let _ = std::io::stdout().flush();
    f();
    println!("(took {:.3}s)", now.elapsed().as_secs_f64());
}
