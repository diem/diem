// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::storage::{BackupStorage, FileHandleRef};
use anyhow::Result;
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::io::AsyncReadExt;

#[async_trait]
pub trait BackupStorageExt {
    async fn read_all(&self, file_handle: &FileHandleRef) -> Result<Vec<u8>>;
    async fn load_json_file<T: DeserializeOwned>(&self, file_handle: &FileHandleRef) -> Result<T>;
    async fn load_lcs_file<T: DeserializeOwned>(&self, file_handle: &FileHandleRef) -> Result<T>;
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
}
