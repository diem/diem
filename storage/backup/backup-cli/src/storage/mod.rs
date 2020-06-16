// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod command_adapter;
pub mod local_fs;

#[cfg(test)]
mod test_util;
#[cfg(test)]
mod tests;

use crate::storage::{
    command_adapter::{CommandAdapter, CommandAdapterOpt},
    local_fs::{LocalFs, LocalFsOpt},
};
use anyhow::{ensure, Result};
use async_trait::async_trait;
use once_cell::sync::Lazy;
#[cfg(test)]
use proptest::prelude::*;
use regex::Regex;
#[cfg(test)]
use std::convert::TryInto;
use std::{convert::TryFrom, ops::Deref, str::FromStr, sync::Arc};
use structopt::StructOpt;
use tokio::io::{AsyncRead, AsyncWrite};

pub type BackupHandle = String;
pub type BackupHandleRef = str;
pub type FileHandle = String;
pub type FileHandleRef = str;

#[cfg_attr(test, derive(Debug, Hash, Eq, PartialEq))]
pub struct ShellSafeName(String);

impl ShellSafeName {
    const PATTERN: &'static str = r"\A[a-zA-Z0-9][a-zA-Z0-9._-]{0,126}\z";

    fn sanitize(name: &str) -> Result<()> {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(ShellSafeName::PATTERN).unwrap());
        ensure!(RE.is_match(&name), "Illegal name: {}", name,);
        Ok(())
    }
}

impl TryFrom<String> for ShellSafeName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        Self::sanitize(&value).map(|_| Self(value))
    }
}

impl FromStr for ShellSafeName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::sanitize(s).map(|_| Self(s.to_string()))
    }
}

impl Deref for ShellSafeName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for ShellSafeName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
impl Arbitrary for ShellSafeName {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (&ShellSafeName::PATTERN[2..(ShellSafeName::PATTERN.len() - 2)]) // remove \A and \z
            .prop_map(|s| s.try_into().unwrap())
            .boxed()
    }
}

#[async_trait]
pub trait BackupStorage {
    /// Hint that a bunch of files are gonna be created related to a backup identified by `name`,
    /// which is unique to the content of the backup, i.e. it won't be the same name unless you are
    /// backing up exactly the same thing.
    /// Storage can choose to take actions like create a dedicated folder or do nothing.
    /// Returns a string to identify this operation in potential succeeding file creation requests.
    async fn create_backup(&self, name: &ShellSafeName) -> Result<BackupHandle>;
    /// Ask to create a file for write, `backup_handle` was returned by `create_backup` to identify
    /// the current backup.
    async fn create_for_write(
        &self,
        backup_handle: &BackupHandleRef,
        name: &ShellSafeName,
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
