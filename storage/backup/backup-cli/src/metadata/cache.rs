// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    metadata::{view::MetadataView, Metadata},
    metrics::metadata::{NUM_META_DOWNLOAD, NUM_META_FILES, NUM_META_MISS},
    storage::{BackupStorage, FileHandle},
    utils::stream::StreamX,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use diem_logger::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
    time::Instant,
};
use structopt::StructOpt;
use tokio::{
    fs::{create_dir_all, read_dir, remove_file, OpenOptions},
    io::{AsyncRead, AsyncReadExt},
    stream::StreamExt,
};

#[derive(StructOpt)]
pub struct MetadataCacheOpt {
    #[structopt(
        long = "metadata-cache-dir",
        parse(from_os_str),
        help = "Metadata cache dir."
    )]
    dir: Option<PathBuf>,
}

impl MetadataCacheOpt {
    // in cache we save things other than the cached files.
    const SUB_DIR: &'static str = "cache";

    fn cache_dir(&self) -> PathBuf {
        self.dir
            .clone()
            .unwrap_or_else(|| {
                let home_path: PathBuf = std::env::var_os("HOME")
                    .expect("Can't find home dir. Specify metadata cache path explicitly.")
                    .into();
                home_path.join("diem_backup_metadata")
            })
            .join(Self::SUB_DIR)
    }
}

/// Sync local cache folder with remote storage, and load all metadata entries from the cache.
pub async fn sync_and_load(
    opt: &MetadataCacheOpt,
    storage: Arc<dyn BackupStorage>,
) -> Result<MetadataView> {
    let timer = Instant::now();
    let cache_dir = opt.cache_dir();
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
    info!("Metadata files listed.");
    NUM_META_FILES.set(remote_hashes.len() as i64);

    // Sync local cache with remote metadata files.
    let stale_local_hashes = local_hashes.difference(&remote_hashes);
    let new_remote_hashes = remote_hashes.difference(&local_hashes).collect::<Vec<_>>();
    let up_to_date_local_hashes = local_hashes.intersection(&remote_hashes);

    for h in stale_local_hashes {
        remove_file(&cache_dir.join(&*h)).await?;
    }

    NUM_META_MISS.set(new_remote_hashes.len() as i64);
    NUM_META_DOWNLOAD.set(0);
    let futs = new_remote_hashes.iter().map(|h| {
        let fh_by_h_ref = &remote_file_handle_by_hash;
        let storage_ref = &storage;
        let cache_dir_ref = &cache_dir;

        async move {
            let file_handle = fh_by_h_ref.get(*h).expect("In map.");
            tokio::io::copy(
                &mut storage_ref.open_for_read(file_handle).await?,
                &mut OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&cache_dir_ref.join(*h))
                    .await?,
            )
            .await?;
            NUM_META_DOWNLOAD.inc();
            Ok(())
        }
    });
    futures::stream::iter(futs)
        .buffered_x(
            num_cpus::get() * 2, /* buffer size */
            num_cpus::get(),     /* concurrency */
        )
        .collect::<Result<Vec<_>>>()
        .await?;

    // Load metadata from synced cache files.
    let mut metadata_vec = Vec::new();
    for h in new_remote_hashes.into_iter().chain(up_to_date_local_hashes) {
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
    info!(
        "Metadata cache loaded in {:.2} seconds.",
        timer.elapsed().as_secs_f64()
    );
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
