// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::{
        epoch_ending::restore::EpochHistoryRestoreController,
        state_snapshot::restore::{StateSnapshotRestoreController, StateSnapshotRestoreOpt},
        transaction::restore::{TransactionRestoreController, TransactionRestoreOpt},
    },
    metadata,
    metadata::{cache::MetadataCacheOpt, TransactionBackupMeta},
    metrics::restore::COORDINATOR_TARGET_VERSION,
    storage::BackupStorage,
    utils::GlobalRestoreOpt,
};
use anyhow::{bail, Result};
use libra_logger::prelude::*;
use libra_types::transaction::Version;
use libradb::backup::restore_handler::RestoreHandler;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct RestoreCoordinatorOpt {
    #[structopt(flatten)]
    pub metadata_cache_opt: MetadataCacheOpt,
}

pub struct RestoreCoordinator {
    storage: Arc<dyn BackupStorage>,
    restore_handler: Arc<RestoreHandler>,
    global_opt: GlobalRestoreOpt,
    metadata_cache_opt: MetadataCacheOpt,
}

impl RestoreCoordinator {
    pub fn new(
        opt: RestoreCoordinatorOpt,
        global_opt: GlobalRestoreOpt,
        storage: Arc<dyn BackupStorage>,
        restore_handler: Arc<RestoreHandler>,
    ) -> Self {
        Self {
            storage,
            restore_handler,
            global_opt,
            metadata_cache_opt: opt.metadata_cache_opt,
        }
    }

    pub async fn run(self) -> Result<()> {
        info!("Restore coordinator started.");
        let metadata_view =
            metadata::cache::sync_and_load(&self.metadata_cache_opt, Arc::clone(&self.storage))
                .await?;

        let transactions = metadata_view.select_transaction_backups(self.target_version())?;
        let actual_target_version = self.get_actual_target_version(&transactions)?;
        let epoch_endings = metadata_view.select_epoch_ending_backups(actual_target_version)?;
        let state_snapshot = metadata_view.select_state_snapshot(actual_target_version)?;
        let replay_transactions_from_version = match &state_snapshot {
            Some(b) => b.version + 1,
            None => 0,
        };
        COORDINATOR_TARGET_VERSION.set(actual_target_version as i64);
        info!("Planned to restore to version {}.", actual_target_version);
        let txn_resume_point = self
            .restore_handler
            .get_next_expected_transaction_version()?;
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
                self.restore_handler.clone(),
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
                Arc::clone(&self.restore_handler),
                Some(Arc::clone(&epoch_history)),
            )
            .run()
            .await?;
        }

        for backup in transactions {
            if backup.last_version < txn_resume_point {
                info!("Skipping {} due to non-empty DB.", backup.manifest);
                continue;
            }

            TransactionRestoreController::new(
                TransactionRestoreOpt {
                    manifest_handle: backup.manifest,
                    replay_from_version: Some(replay_transactions_from_version),
                },
                self.global_opt.clone(),
                Arc::clone(&self.storage),
                Arc::clone(&self.restore_handler),
                Some(Arc::clone(&epoch_history)),
            )
            .run()
            .await?;
        }

        info!("Restore coordinator exiting with success.");
        Ok(())
    }
}

impl RestoreCoordinator {
    fn target_version(&self) -> Version {
        self.global_opt.target_version()
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
