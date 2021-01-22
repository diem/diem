// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests;

use super::{BackupHandle, BackupHandleRef, FileHandle, FileHandleRef};

use crate::{
    storage::{BackupStorage, ShellSafeName, TextLine},
    utils::{error_notes::ErrorNotes, path_exists, PathToString},
};
use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tokio::{
    fs::{create_dir, create_dir_all, read_dir, OpenOptions},
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
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
    const METADATA_DIR: &'static str = "metadata";

    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn new_with_opt(opt: LocalFsOpt) -> Self {
        Self::new(opt.dir)
    }

    pub fn metadata_dir(&self) -> PathBuf {
        self.dir.join(Self::METADATA_DIR)
    }
}

#[async_trait]
impl BackupStorage for LocalFs {
    async fn create_backup(&self, name: &ShellSafeName) -> Result<BackupHandle> {
        create_dir(self.dir.join(name.as_ref()))
            .await
            .err_notes(name)?;
        Ok(name.to_string())
    }

    async fn create_for_write(
        &self,
        backup_handle: &BackupHandleRef,
        name: &ShellSafeName,
    ) -> Result<(FileHandle, Box<dyn AsyncWrite + Send + Unpin>)> {
        let file_handle = Path::new(backup_handle)
            .join(name.as_ref())
            .path_to_string()?;
        let abs_path = self.dir.join(&file_handle).path_to_string()?;
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&abs_path)
            .await
            .err_notes(&abs_path)?;
        Ok((file_handle, Box::new(file)))
    }

    async fn open_for_read(
        &self,
        file_handle: &FileHandleRef,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>> {
        let path = self.dir.join(file_handle);
        let file = OpenOptions::new()
            .read(true)
            .open(&path)
            .await
            .err_notes(&path)?;
        Ok(Box::new(file))
    }

    async fn save_metadata_line(&self, name: &ShellSafeName, content: &TextLine) -> Result<()> {
        let dir = self.metadata_dir();
        create_dir_all(&dir).await.err_notes(name)?; // in case not yet created

        let path = dir.join(name.as_ref());
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
            .err_notes(&path)?;
        file.write_all(content.as_ref().as_bytes())
            .await
            .err_notes(&path)?;

        Ok(())
    }

    async fn list_metadata_files(&self) -> Result<Vec<FileHandle>> {
        let dir = self.metadata_dir();
        let rel_path = Path::new(Self::METADATA_DIR);

        let mut res = Vec::new();
        if path_exists(&dir).await {
            let mut entries = read_dir(&dir).await.err_notes(&dir)?;
            while let Some(entry) = entries.next_entry().await.err_notes(&dir)? {
                res.push(rel_path.join(entry.file_name()).path_to_string()?)
            }
        }
        Ok(res)
    }
}
