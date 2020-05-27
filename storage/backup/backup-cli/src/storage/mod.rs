// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod local_fs;

use anyhow::Result;
use async_trait::async_trait;
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
        file_handle: &FileHandleRef,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>>;
}
