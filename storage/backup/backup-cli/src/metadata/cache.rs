// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    metadata::{view::MetadataView, Metadata},
    storage::{BackupStorage, FileHandle},
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};
use tokio::{
    fs::{create_dir_all, read_dir, remove_file, OpenOptions},
    io::{AsyncRead, AsyncReadExt},
    stream::StreamExt,
};

const CACHE_DIR: &str = "cache";

/// Sync local cache folder with remote storage, and load all metadata entries from the cache.
pub async fn sync_and_load(dir: &PathBuf, storage: Arc<dyn BackupStorage>) -> Result<MetadataView> {
    let cache_dir = dir.join(CACHE_DIR);
    create_dir_all(&cache_dir).await?; // create if not present already

    // List cached metadata files.
    let local_entries = read_dir(&cache_dir)
        .await?
        .collect::<tokio::io::Result<Vec<_>>>()
        .await?;
    let local_hashes = local_entries
        .iter()
        .map(|e| {
            e.file_name()
                .into_string()
                .map_err(|s| anyhow!("into_string() failed for file name {:?}", s))
        })
        .collect::<Result<HashSet<_>>>()?;

    // List remote metadata files.
    let remote_file_handles = storage.list_metadata_files().await?;
    let remote_file_handle_by_hash: HashMap<_, _> = remote_file_handles
        .into_iter()
        .map(|file_handle| (file_handle.file_handle_hash(), file_handle))
        .collect();
    let remote_hashes: HashSet<_> = remote_file_handle_by_hash.keys().cloned().collect();

    // Sync local cache with remote metadata files.
    let stale_local_hashes = local_hashes.difference(&remote_hashes);
    let new_remote_hashes = remote_hashes.difference(&local_hashes);
    let up_to_date_local_hashes = local_hashes.intersection(&remote_hashes);

    for h in stale_local_hashes {
        remove_file(&cache_dir.join(&*h)).await?;
    }
    for h in new_remote_hashes.clone() {
        let file_handle = remote_file_handle_by_hash.get(&*h).unwrap();
        tokio::io::copy(
            &mut storage.open_for_read(file_handle).await?,
            &mut OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&cache_dir.join(&*h))
                .await?,
        )
        .await?;
    }

    // Load metadata from synced cache files.
    let mut metadata_vec = Vec::new();
    for h in new_remote_hashes.chain(up_to_date_local_hashes) {
        let cached_file = cache_dir.join(&*h);
        metadata_vec.extend(
            OpenOptions::new()
                .read(true)
                .open(&cached_file)
                .await?
                .load_metadata_lines()
                .await?
                .into_iter(),
        )
    }
    Ok(metadata_vec.into())
}

trait FileHandleHash {
    fn file_handle_hash(&self) -> String;
}

impl FileHandleHash for FileHandle {
    fn file_handle_hash(&self) -> String {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

#[async_trait]
trait LoadMetadataLines {
    async fn load_metadata_lines(&mut self) -> Result<Vec<Metadata>>;
}

#[async_trait]
impl<R: AsyncRead + Send + Unpin> LoadMetadataLines for R {
    async fn load_metadata_lines(&mut self) -> Result<Vec<Metadata>> {
        let mut buf = String::new();
        self.read_to_string(&mut buf).await?;
        Ok(buf
            .lines()
            .map(serde_json::from_str::<Metadata>)
            .collect::<Result<_, serde_json::error::Error>>()?)
    }
}
