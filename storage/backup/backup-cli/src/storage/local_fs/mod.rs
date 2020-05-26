// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests;

use super::{BackupHandle, BackupHandleRef, FileHandle, FileHandleRef};

use crate::storage::BackupStorage;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::{
    fs::{create_dir, OpenOptions},
    io::{AsyncRead, AsyncWrite},
};

#[derive(StructOpt)]
pub struct LocalFsOpt {
    #[structopt(
        long = "local-backup-store",
        parse(from_os_str),
        help = "Target local dir to hold backups."
    )]
    pub dir: PathBuf,
}

/// A storage backend that stores everything in a local directory.
pub struct LocalFs {
    /// The path where everything is stored.
    dir: PathBuf,
}

impl LocalFs {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn new_with_opt(opt: LocalFsOpt) -> Self {
        Self::new(opt.dir)
    }
}

#[async_trait]
impl BackupStorage for LocalFs {
    async fn create_backup(&self, name: &str) -> Result<BackupHandle> {
        create_dir(self.dir.join(name)).await?;
        Ok(name.to_string())
    }

    async fn create_for_write(
        &self,
        backup_handle: &BackupHandleRef,
        name: &str,
    ) -> Result<(FileHandle, Box<dyn AsyncWrite + Send + Unpin>)> {
        let file_handle = self
            .dir
            .join(backup_handle)
            .join(name)
            .into_os_string()
            .into_string()
            .map_err(|s| anyhow!("into_string failed for OsString '{:?}'", s))?;
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file_handle)
            .await?;
        Ok((file_handle, Box::new(file)))
    }

    /// N.B: `LocalFs` uses absolute paths as file handles, so `self.dir` doesn't matter.
    async fn open_for_read(
        &self,
        file_handle: &FileHandleRef,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
        let file = OpenOptions::new().read(true).open(file_handle).await?;
        Ok(Box::new(file))
    }
}
