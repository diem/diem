// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests;

use super::{BackupHandle, BackupHandleRef, FileHandle, FileHandleRef};

use crate::{
    storage::{BackupStorage, ShellSafeName, TextLine},
    utils::path_exists,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::{
    fs::{create_dir, create_dir_all, read_dir, OpenOptions},
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    stream::StreamExt,
};

#[derive(StructOpt)]
pub struct LocalFsOpt {
    #[structopt(
        long = "dir",
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

    pub fn metadata_dir(&self) -> PathBuf {
        const METADATA_DIR: &str = "metadata";
        self.dir.join(METADATA_DIR)
    }
}

#[async_trait]
impl BackupStorage for LocalFs {
    async fn create_backup(&self, name: &ShellSafeName) -> Result<BackupHandle> {
        create_dir(self.dir.join(name.as_ref())).await?;
        Ok(name.to_string())
    }

    async fn create_for_write(
        &self,
        backup_handle: &BackupHandleRef,
        name: &ShellSafeName,
    ) -> Result<(FileHandle, Box<dyn AsyncWrite + Send + Unpin>)> {
        let file_handle = self
            .dir
            .join(backup_handle)
            .join(name.as_ref())
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

    async fn save_metadata_line(&self, name: &ShellSafeName, content: &TextLine) -> Result<()> {
        let dir = self.metadata_dir();
        create_dir_all(&dir).await?; // in case not yet created

        let path = dir.join(name.as_ref());
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await?;
        file.write_all(content.as_ref().as_bytes()).await?;

        Ok(())
    }

    async fn list_metadata_files(&self) -> Result<Vec<FileHandle>> {
        let dir = self.metadata_dir();

        let mut res = Vec::new();
        if path_exists(&dir).await {
            let mut entries = read_dir(&dir).await?;
            while let Some(entry) = entries.try_next().await? {
                res.push(
                    dir.join(entry.file_name())
                        .into_os_string()
                        .into_string()
                        .map_err(|s| anyhow!("into_string failed for OsString {:?}", s))?,
                )
            }
        }
        Ok(res)
    }
}
