// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use rand::RngCore;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// A simple wrapper for creating a temporary directory that is
/// automatically deleted when it's dropped.
///
/// We use this in lieu of tempfile because tempfile brings in too many
/// dependencies.
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
    /// Create new uninitialized temporary path, i.e. a file or directory
    /// isn't created automatically
    pub fn new() -> Self {
        let tmpdir = create_path();
        TempPath {
            path_buf: tmpdir,
            persist: false,
        }
    }

    /// Return the underlying path to this temporary directory.
    pub fn path(&self) -> &Path {
        &self.path_buf
    }

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

fn create_path() -> PathBuf {
    create_path_in_dir(std::env::temp_dir())
}

fn create_path_in_dir(path: PathBuf) -> PathBuf {
    let mut path = path;
    let mut rng = rand::thread_rng();
    let mut bytes = [0_u8; 16];
    rng.fill_bytes(&mut bytes);
    let path_string = hex::encode(&bytes);

    path.push(path_string);
    path
}

impl std::convert::AsRef<Path> for TempPath {
    fn as_ref(&self) -> &Path {
        self.path()
    }
}
