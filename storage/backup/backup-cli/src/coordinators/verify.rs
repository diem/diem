// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::{
        epoch_ending::restore::EpochHistoryRestoreController,
        state_snapshot::restore::{StateSnapshotRestoreController, StateSnapshotRestoreOpt},
        transaction::restore::{TransactionRestoreController, TransactionRestoreOpt},
    },
    metadata,
    metadata::cache::MetadataCacheOpt,
    storage::BackupStorage,
    utils::{GlobalRestoreOptions, RestoreRunMode},
};
use anyhow::Result;
use libra_logger::prelude::*;
use libra_types::transaction::Version;
use std::sync::Arc;

pub struct VerifyCoordinator {
    storage: Arc<dyn BackupStorage>,
    metadata_cache_opt: MetadataCacheOpt,
}

impl VerifyCoordinator {
    pub fn new(
        storage: Arc<dyn BackupStorage>,
        metadata_cache_opt: MetadataCacheOpt,
    ) -> Result<Self> {
        Ok(Self {
            storage,
            metadata_cache_opt,
        })
    }

    pub async fn run(self) -> Result<()> {
        info!("Verify coordinator started.");

        let metadata_view =
            metadata::cache::sync_and_load(&self.metadata_cache_opt, Arc::clone(&self.storage))
                .await?;
        let ver_max = Version::max_value();
        let state_snapshot = metadata_view.select_state_snapshot(ver_max)?;
        let transactions = metadata_view.select_transaction_backups(ver_max)?;
        let epoch_endings = metadata_view.select_epoch_ending_backups(ver_max)?;

        let global_opt = GlobalRestoreOptions {
            target_version: ver_max,
            run_mode: Arc::new(RestoreRunMode::Verify),
        };

        let epoch_history = Arc::new(
            EpochHistoryRestoreController::new(
                epoch_endings
                    .into_iter()
                    .map(|backup| backup.manifest)
                    .collect(),
                global_opt.clone(),
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
                global_opt.clone(),
                Arc::clone(&self.storage),
                Some(Arc::clone(&epoch_history)),
            )
            .run()
            .await?;
        }

        for backup in transactions {
            TransactionRestoreController::new(
                TransactionRestoreOpt {
                    manifest_handle: backup.manifest,
                    replay_from_version: None,
                },
                global_opt.clone(),
                Arc::clone(&self.storage),
                Some(Arc::clone(&epoch_history)),
            )
            .run()
            .await?;
        }

        info!("Verify coordinator exiting with success.");
        Ok(())
    }
}
