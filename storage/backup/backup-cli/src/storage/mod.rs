// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod command_adapter;
pub mod local_fs;

#[cfg(test)]
mod test_util;

use crate::storage::{
    command_adapter::{CommandAdapter, CommandAdapterOpt},
    local_fs::{LocalFs, LocalFsOpt},
};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::io::{AsyncRead, AsyncWrite};

pub type BackupHandle = String;
pub type BackupHandleRef = str;
pub type FileHandle = String;
pub type FileHandleRef = str;

#[async_trait]
pub trait BackupStorage {
    /// Hint that a bunch of files are gonna be created related to a backup identified by `name`,
    /// which is unique to the content of the backup, i.e. it won't be the same name unless you are
    /// backing up exactly the same thing.
    /// Storage can choose to take actions like create a dedicated folder or do nothing.
    /// Returns a string to identify this operation in potential succeeding file creation requests.
    async fn create_backup(&self, name: &str) -> Result<BackupHandle>;
    /// Ask to create a file for write, `backup_handle` was returned by `create_backup` to identify
    /// the current backup.
    async fn create_for_write(
        &self,
        backup_handle: &BackupHandleRef,
        name: &str,
    ) -> Result<(FileHandle, Box<dyn AsyncWrite + Send + Unpin>)>;
    /// Open file for reading.
    async fn open_for_read(
        &self,
        file_handle: &FileHandleRef,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>>;
}

#[derive(StructOpt)]
pub enum StorageOpt {
    #[structopt(about = "Select the LocalFs backup store.")]
    LocalFs(LocalFsOpt),
    #[structopt(about = "Select the CommandAdapter backup store.")]
    CommandAdapter(CommandAdapterOpt),
}

impl StorageOpt {
    pub async fn init_storage(self) -> Result<Arc<dyn BackupStorage>> {
        Ok(match self {
            StorageOpt::LocalFs(opt) => Arc::new(LocalFs::new_with_opt(opt)),
            StorageOpt::CommandAdapter(opt) => Arc::new(CommandAdapter::new_with_opt(opt).await?),
        })
    }
}
