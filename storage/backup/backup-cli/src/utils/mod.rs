// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod backup_service_client;
pub mod read_record_bytes;
pub mod storage_ext;

#[cfg(test)]
pub mod test_utils;

use anyhow::{anyhow, Result};
use libra_types::transaction::Version;
use std::{
    mem::size_of,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use tokio::fs::metadata;

#[derive(Clone, StructOpt)]
pub struct GlobalBackupOpt {
    #[structopt(
        long = "max-chunk-size",
        default_value = "1073741824",
        help = "Maximum chunk file size in bytes."
    )]
    pub max_chunk_size: usize,
}

#[derive(Clone, StructOpt)]
pub struct GlobalRestoreOpt {
    #[structopt(long = "target-db-dir", parse(from_os_str))]
    pub db_dir: PathBuf,
    #[structopt(
        long,
        help = "Content newer than this version will not be recovered to DB, \
        defaulting to the largest version possible, meaning recover everything in the backups."
    )]
    pub target_version: Option<Version>,
}

impl GlobalRestoreOpt {
    pub fn target_version(&self) -> Version {
        self.target_version.unwrap_or(Version::max_value())
    }
}

pub(crate) fn should_cut_chunk(chunk: &[u8], record: &[u8], max_chunk_size: usize) -> bool {
    !chunk.is_empty() && chunk.len() + record.len() + size_of::<u32>() > max_chunk_size
}

// TODO: use Path::exists() when Rust 1.5 stabilizes.
pub(crate) async fn path_exists(path: &PathBuf) -> bool {
    metadata(&path).await.is_ok()
}

pub(crate) trait PathToString {
    fn path_to_string(&self) -> Result<String>;
}

impl<T: AsRef<Path>> PathToString for T {
    fn path_to_string(&self) -> Result<String> {
        self.as_ref()
            .to_path_buf()
            .into_os_string()
            .into_string()
            .map_err(|s| anyhow!("into_string failed for OsString '{:?}'", s))
    }
}
