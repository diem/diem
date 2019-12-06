// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod local_storage;

use crate::FileHandle;
use anyhow::Result;
use async_trait::async_trait;
use futures::{stream::BoxStream, StreamExt};

/// `Adapter` defines the interfaces of the storage backend we use for backup and restore.
#[async_trait]
pub trait Adapter {
    /// Writes the content of the file to the storage backend and returns a handle to the file once
    /// finished.
    async fn write_new_file(
        &self,
        content: impl StreamExt<Item = Vec<u8>> + Send + Unpin + 'async_trait,
    ) -> Result<FileHandle>;

    /// Returns the content of the file in a stream.
    #[allow(clippy::ptr_arg)]
    fn read_file_content(&self, file_handle: &FileHandle) -> BoxStream<Result<Vec<u8>>>;
}
