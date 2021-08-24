// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::{
        epoch_ending::restore::EpochHistoryRestoreController,
        state_snapshot::restore::{StateSnapshotRestoreController, StateSnapshotRestoreOpt},
        transaction::restore::TransactionRestoreBatchController,
    },
    metadata,
    metadata::{cache::MetadataCacheOpt, TransactionBackupMeta},
    metrics::restore::{
        COORDINATOR_FAIL_TS, COORDINATOR_START_TS, COORDINATOR_SUCC_TS, COORDINATOR_TARGET_VERSION,
    },
    storage::BackupStorage,
    utils::{unix_timestamp_sec, GlobalRestoreOptions, RestoreRunMode},
};
use anyhow::{bail, Result};
use diem_logger::prelude::*;
use diem_types::transaction::Version;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct RestoreCoordinatorOpt {
    #[structopt(flatten)]
    pub metadata_cache_opt: MetadataCacheOpt,
    #[structopt(
        long,
        help = "Replay all transactions, don't try to use a state snapshot."
    )]
    pub replay_all: bool,
}

pub struct RestoreCoordinator {
    storage: Arc<dyn BackupStorage>,
    global_opt: GlobalRestoreOptions,
    metadata_cache_opt: MetadataCacheOpt,
    replay_all: bool,
}

impl RestoreCoordinator {
    pub fn new(
        opt: RestoreCoordinatorOpt,
        global_opt: GlobalRestoreOptions,
        storage: Arc<dyn BackupStorage>,
    ) -> Self {
        Self {
            storage,
            global_opt,
            metadata_cache_opt: opt.metadata_cache_opt,
            replay_all: opt.replay_all,
        }
    }

    pub async fn run(self) -> Result<()> {
        info!("Restore coordinator started.");
        COORDINATOR_START_TS.set(unix_timestamp_sec());

        let ret = self.run_impl().await;

        if let Err(e) = &ret {
            error!(
                error = ?e,
                "Restore coordinator failed."
            );
            COORDINATOR_FAIL_TS.set(unix_timestamp_sec());
        } else {
            info!("Restore coordinator exiting with success.");
            COORDINATOR_SUCC_TS.set(unix_timestamp_sec());
        }

        ret
    }

    async fn run_impl(self) -> Result<()> {
        let metadata_view = metadata::cache::sync_and_load(
            &self.metadata_cache_opt,
            Arc::clone(&self.storage),
            self.global_opt.concurrent_downloads,
        )
        .await?;

        let transactions = metadata_view.select_transaction_backups(0, self.target_version())?;
        let actual_target_version = self.get_actual_target_version(&transactions)?;
        let epoch_endings = metadata_view.select_epoch_ending_backups(actual_target_version)?;
        let state_snapshot = if self.replay_all {
            None
        } else {
            metadata_view.select_state_snapshot(actual_target_version)?
        };
        let replay_transactions_from_version = match &state_snapshot {
            Some(b) => b.version + 1,
            None => 0,
        };
        COORDINATOR_TARGET_VERSION.set(actual_target_version as i64);
        info!("Planned to restore to version {}.", actual_target_version);
        let txn_resume_point = match self.global_opt.run_mode.as_ref() {
            RestoreRunMode::Restore { restore_handler } => {
                restore_handler.get_next_expected_transaction_version()?
            }
            RestoreRunMode::Verify => {
                info!("This is a dry run.");
                0
            }
        };
        if txn_resume_point > 0 {
            warn!(
                "DB has existing transactions, will skip transaction backups before version {}",
                txn_resume_point
            );
        }

        let epoch_history = Arc::new(
            EpochHistoryRestoreController::new(
                epoch_endings
                    .into_iter()
                    .map(|backup| backup.manifest)
                    .collect(),
                self.global_opt.clone(),
                self.storage.clone(),
            )
            .run()
            .await?,
        );

        if let Some(backup) = state_snapshot {
            StateSnapshotRestoreController::new(
                StateSnapshotRestoreOpt {
                    manifest_handle: backup.manifest,
                    version: backup.version,
                },
                self.global_opt.clone(),
                Arc::clone(&self.storage),
                Some(Arc::clone(&epoch_history)),
            )
            .run()
            .await?;
        }

        let txn_manifests = transactions
            .into_iter()
            .skip_while(|b| b.last_version < txn_resume_point)
            .map(|b| b.manifest)
            .collect();
        TransactionRestoreBatchController::new(
            self.global_opt,
            self.storage,
            txn_manifests,
            Some(replay_transactions_from_version),
            Some(epoch_history),
        )
        .run()
        .await?;

        Ok(())
    }
}

impl RestoreCoordinator {
    fn target_version(&self) -> Version {
        self.global_opt.target_version
    }

    fn get_actual_target_version(
        &self,
        transaction_backups: &[TransactionBackupMeta],
    ) -> Result<Version> {
        if let Some(b) = transaction_backups.last() {
            if b.last_version > self.target_version() {
                Ok(self.target_version())
            } else {
                warn!(
                    "Can't find transaction backup containing the target version, \
                    will restore as much as possible"
                );
                Ok(b.last_version)
            }
        } else {
            bail!("No transaction backup found.")
        }
    }
}
