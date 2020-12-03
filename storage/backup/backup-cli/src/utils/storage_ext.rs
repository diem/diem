// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{BackupHandle, BackupStorage, FileHandleRef};
use anyhow::Result;
use async_trait::async_trait;
use rand::random;
use serde::de::DeserializeOwned;
use std::{convert::TryInto, sync::Arc};
use tokio::io::AsyncReadExt;

#[async_trait]
pub trait BackupStorageExt {
    async fn read_all(&self, file_handle: &FileHandleRef) -> Result<Vec<u8>>;
    async fn load_json_file<T: DeserializeOwned>(&self, file_handle: &FileHandleRef) -> Result<T>;
    async fn load_lcs_file<T: DeserializeOwned>(&self, file_handle: &FileHandleRef) -> Result<T>;
    /// Adds a random suffix ".XXXX" to the backup name, so a retry won't pass a same backup name to
    /// the storage.
    async fn create_backup_with_random_suffix(&self, name: &str) -> Result<BackupHandle>;
}

#[async_trait]
impl BackupStorageExt for Arc<dyn BackupStorage> {
    async fn read_all(&self, file_handle: &FileHandleRef) -> Result<Vec<u8>> {
        let mut file = self.open_for_read(&file_handle).await?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).await?;
        Ok(bytes)
    }

    async fn load_lcs_file<T: DeserializeOwned>(&self, file_handle: &FileHandleRef) -> Result<T> {
        Ok(lcs::from_bytes(&self.read_all(&file_handle).await?)?)
    }

    async fn load_json_file<T: DeserializeOwned>(&self, file_handle: &FileHandleRef) -> Result<T> {
        Ok(serde_json::from_slice(&self.read_all(&file_handle).await?)?)
    }

    async fn create_backup_with_random_suffix(&self, name: &str) -> Result<BackupHandle> {
        self.create_backup(&format!("{}.{:04x}", name, random::<u16>()).try_into()?)
            .await
    }
}
