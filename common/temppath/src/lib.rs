// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use rand::RngCore;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// A simple wrapper for creating a temporary directory that is automatically deleted when it's
/// dropped. Used in lieu of tempfile due to the large number of dependencies.
#[derive(Debug, PartialEq)]
pub struct TempPath {
    path_buf: PathBuf,
    persist: bool,
}

impl Drop for TempPath {
    fn drop(&mut self) {
        if !self.persist {
            fs::remove_dir_all(&self.path_buf)
                .or_else(|_| fs::remove_file(&self.path_buf))
                .unwrap_or(());
        }
    }
}

impl TempPath {
    /// Create new, uninitialized temporary path in the system temp directory.
    pub fn new() -> Self {
        Self::new_with_temp_dir(std::env::temp_dir())
    }

    /// Create new, uninitialized temporary path in the specified directory.
    pub fn new_with_temp_dir(temp_dir: PathBuf) -> Self {
        let mut temppath = temp_dir;
        let mut rng = rand::thread_rng();
        let mut bytes = [0_u8; 16];
        rng.fill_bytes(&mut bytes);
        temppath.push(hex::encode(&bytes));

        TempPath {
            path_buf: temppath,
            persist: false,
        }
    }

    /// Return the underlying path to this temporary directory.
    pub fn path(&self) -> &Path {
        &self.path_buf
    }

    /// Keep the temp path
    pub fn persist(&mut self) {
        self.persist = true;
    }

    pub fn create_as_file(&self) -> io::Result<()> {
        let mut builder = fs::OpenOptions::new();
        builder.write(true).create_new(true);
        builder.open(self.path())?;
        Ok(())
    }

    pub fn create_as_dir(&self) -> io::Result<()> {
        let builder = fs::DirBuilder::new();
        builder.create(self.path())?;
        Ok(())
    }
}

impl std::convert::AsRef<Path> for TempPath {
    fn as_ref(&self) -> &Path {
        self.path()
    }
}

impl Default for TempPath {
    fn default() -> Self {
        Self::new()
    }
}
