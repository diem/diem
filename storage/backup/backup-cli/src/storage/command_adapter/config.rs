// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{BackupHandle, FileHandle};
use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;

#[derive(Clone, Debug, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
}

impl EnvVar {
    pub fn backup_name(value: String) -> Self {
        Self::new("BACKUP_NAME".to_string(), value)
    }

    pub fn file_name(value: String) -> Self {
        Self::new("FILE_NAME".to_string(), value)
    }

    pub fn file_handle(value: FileHandle) -> Self {
        Self::new("FILE_HANDLE".to_string(), value)
    }

    pub fn backup_handle(value: BackupHandle) -> Self {
        Self::new("BACKUP_HANDLE".to_string(), value)
    }

    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

#[derive(Deserialize)]
pub struct Commands {
    /// Command line to create backup.
    /// input env vars:
    ///     $BACKUP_NAME
    /// expected output on stdout:
    ///     BackupHandle, trailing newline is trimmed
    pub create_backup: String,
    /// Command line to open a file for writing.
    /// input env vars:
    ///     $BACKUP_HANDLE returned from the previous command
    ///     $FILE_NAME
    /// stdin will be fed with byte stream.
    /// expected output on stdout:
    ///     FileHandle, trailing newline
    pub create_for_write: String,
    /// Command line to open a file for reading.
    /// input env vars:
    ///     $FILE_NAME
    /// expected stdout to stream out bytes of the file.
    pub open_for_read: String,
}

#[derive(Deserialize)]
pub struct CommandAdapterConfig {
    /// Command lines that implements `BackupStorage` APIs.
    pub commands: Commands,
    /// Additional environment variables to be set when command lines are spawned.
    pub env_vars: Vec<EnvVar>,
}

impl CommandAdapterConfig {
    pub async fn load_from_file(path: &PathBuf) -> Result<Self> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut content = Vec::new();
        file.read_to_end(&mut content).await?;

        Ok(toml::from_slice(&content)?)
    }

    pub fn load_from_str(content: &str) -> Result<Self> {
        Ok(toml::from_str(content)?)
    }
}
