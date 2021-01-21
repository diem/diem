// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod backup_service_client;
pub mod read_record_bytes;
pub mod storage_ext;
pub(crate) mod stream;

#[cfg(test)]
pub mod test_utils;

use anyhow::{anyhow, Result};
use diem_config::config::RocksdbConfig;
use diem_crypto::HashValue;
use diem_infallible::duration_since_epoch;
use diem_jellyfish_merkle::{restore::JellyfishMerkleRestore, NodeBatch, TreeWriter};
use diem_types::{transaction::Version, waypoint::Waypoint};
use diemdb::{backup::restore_handler::RestoreHandler, DiemDB, GetRestoreHandler};
use std::{
    collections::HashMap,
    convert::TryFrom,
    mem::size_of,
    path::{Path, PathBuf},
    sync::Arc,
};
use structopt::StructOpt;
use tokio::fs::metadata;

#[derive(Clone, StructOpt)]
pub struct GlobalBackupOpt {
    // Defaults to 128MB, so concurrent chunk downloads won't take up too much memory.
    #[structopt(
        long = "max-chunk-size",
        default_value = "134217728",
        help = "Maximum chunk file size in bytes."
    )]
    pub max_chunk_size: usize,
}

#[derive(Clone, StructOpt)]
pub struct RocksdbOpt {
    // using a smaller value than a node since we don't care much about reading performance
    // in this tool.
    #[structopt(long, default_value = "1000")]
    max_open_files: i32,
    // using the same default with a node (1GB).
    #[structopt(long, default_value = "1073741824")]
    max_total_wal_size: u64,
}

impl From<RocksdbOpt> for RocksdbConfig {
    fn from(opt: RocksdbOpt) -> Self {
        Self {
            max_open_files: opt.max_open_files,
            max_total_wal_size: opt.max_total_wal_size,
        }
    }
}

impl Default for RocksdbOpt {
    fn default() -> Self {
        Self::from_iter(vec!["exe"])
    }
}

#[derive(Clone, StructOpt)]
pub struct GlobalRestoreOpt {
    #[structopt(long, help = "Dry run without writing data to DB.")]
    pub dry_run: bool,

    #[structopt(
        long = "target-db-dir",
        parse(from_os_str),
        conflicts_with = "dry-run",
        required_unless = "dry-run"
    )]
    pub db_dir: Option<PathBuf>,

    #[structopt(
        long,
        help = "Content newer than this version will not be recovered to DB, \
        defaulting to the largest version possible, meaning recover everything in the backups."
    )]
    pub target_version: Option<Version>,

    #[structopt(flatten)]
    pub trusted_waypoints: TrustedWaypointOpt,

    #[structopt(flatten)]
    pub rocksdb_opt: RocksdbOpt,
}

pub enum RestoreRunMode {
    Restore { restore_handler: RestoreHandler },
    Verify,
}

struct MockTreeWriter;

impl TreeWriter for MockTreeWriter {
    fn write_node_batch(&self, _node_batch: &NodeBatch) -> Result<()> {
        Ok(())
    }
}

impl RestoreRunMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Restore { restore_handler: _ } => "restore",
            Self::Verify => "verify",
        }
    }

    pub fn is_verify(&self) -> bool {
        match self {
            Self::Restore { restore_handler: _ } => false,
            Self::Verify => true,
        }
    }

    pub fn get_state_restore_receiver(
        &self,
        version: Version,
        expected_root_hash: HashValue,
    ) -> Result<JellyfishMerkleRestore> {
        match self {
            Self::Restore { restore_handler } => {
                restore_handler.get_state_restore_receiver(version, expected_root_hash)
            }
            Self::Verify => JellyfishMerkleRestore::new_overwrite(
                Arc::new(MockTreeWriter),
                version,
                expected_root_hash,
            ),
        }
    }
}

#[derive(Clone)]
pub struct GlobalRestoreOptions {
    pub target_version: Version,
    pub trusted_waypoints: Arc<HashMap<Version, Waypoint>>,
    pub run_mode: Arc<RestoreRunMode>,
}

impl TryFrom<GlobalRestoreOpt> for GlobalRestoreOptions {
    type Error = anyhow::Error;

    fn try_from(opt: GlobalRestoreOpt) -> Result<Self> {
        let target_version = opt.target_version.unwrap_or(Version::max_value());
        let run_mode = if let Some(db_dir) = &opt.db_dir {
            let restore_handler = Arc::new(DiemDB::open(
                db_dir,
                false, /* read_only */
                None,  /* pruner */
                opt.rocksdb_opt.into(),
            )?)
            .get_restore_handler();
            RestoreRunMode::Restore { restore_handler }
        } else {
            RestoreRunMode::Verify
        };
        Ok(Self {
            target_version,
            trusted_waypoints: Arc::new(opt.trusted_waypoints.verify()?),
            run_mode: Arc::new(run_mode),
        })
    }
}

#[derive(Clone, Default, StructOpt)]
pub struct TrustedWaypointOpt {
    #[structopt(
        long,
        help = "(multiple) When provided, an epoch ending LedgerInfo at the waypoint version will be \
        checked against the hash in the waypoint, but signatures on it are NOT checked. \
        Use this for two purposes: \
        1. set the genesis or the latest waypoint to confirm the backup is compatible. \
        2. set waypoints at versions where writeset transactions were used to overwrite the \
        validator set, so that the signature check is skipped. \
        N.B. LedgerInfos are verified only when restoring / verifying the epoch ending backups, \
        i.e. they are NOT checked at all when doing one-shot restoring of the transaction \
        and state backups."
    )]
    pub trust_waypoint: Vec<Waypoint>,
}

impl TrustedWaypointOpt {
    pub fn verify(self) -> Result<HashMap<Version, Waypoint>> {
        let mut trusted_waypoints = HashMap::new();
        for w in self.trust_waypoint {
            trusted_waypoints
                .insert(w.version(), w)
                .map_or(Ok(()), |w| {
                    Err(anyhow!("Duplicated waypoints at version {}", w.version()))
                })?;
        }
        Ok(trusted_waypoints)
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

pub(crate) fn unix_timestamp_sec() -> i64 {
    duration_since_epoch().as_secs() as i64
}
