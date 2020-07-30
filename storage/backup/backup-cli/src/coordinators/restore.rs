// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::{
        epoch_ending::restore::{EpochEndingRestoreController, EpochEndingRestoreOpt},
        state_snapshot::restore::{StateSnapshotRestoreController, StateSnapshotRestoreOpt},
        transaction::restore::{TransactionRestoreController, TransactionRestoreOpt},
    },
    metadata,
    metadata::cache::MetadataCacheOpt,
    storage::BackupStorage,
    utils::GlobalRestoreOpt,
};
use anyhow::Result;
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
        let metadata_view =
            metadata::cache::sync_and_load(&self.metadata_cache_opt, Arc::clone(&self.storage))
                .await?;

        let epoch_endings = metadata_view.select_epoch_ending_backups(self.target_version())?;
        let state_snapshot = metadata_view.select_state_snapshot(self.target_version())?;
        let transactions = metadata_view.select_transaction_backups(self.target_version())?;

        if transactions
            .last()
            .map_or(true, |b| b.last_version < self.target_version())
        {
            println!(
                "Warning: Can't find transaction backup that contains the target version, \
            will restore as much as possible"
            );
        }
        let replay_transactions_from_version = match &state_snapshot {
            Some(b) => b.version + 1,
            None => {
                println!(
                    "Warning: Can't find usable state snapshot, \
                will replay transactions from the beginning."
                );
                0
            }
        };

        for backup in epoch_endings {
            EpochEndingRestoreController::new(
                EpochEndingRestoreOpt {
                    manifest_handle: backup.manifest,
                },
                self.global_opt.clone(),
                Arc::clone(&self.storage),
                Arc::clone(&self.restore_handler),
            )
            .run()
            .await?;
        }

        if let Some(backup) = state_snapshot {
            StateSnapshotRestoreController::new(
                StateSnapshotRestoreOpt {
                    manifest_handle: backup.manifest,
                    version: backup.version,
                },
                self.global_opt.clone(),
                Arc::clone(&self.storage),
                Arc::clone(&self.restore_handler),
            )
            .run()
            .await?;
        }

        for backup in transactions {
            TransactionRestoreController::new(
                TransactionRestoreOpt {
                    manifest_handle: backup.manifest,
                    replay_from_version: Some(replay_transactions_from_version),
                },
                self.global_opt.clone(),
                Arc::clone(&self.storage),
                Arc::clone(&self.restore_handler),
            )
            .run()
            .await?;
        }

        println!("Restore finished.");
        Ok(())
    }
}

impl RestoreCoordinator {
    fn target_version(&self) -> Version {
        self.global_opt.target_version()
    }
}
