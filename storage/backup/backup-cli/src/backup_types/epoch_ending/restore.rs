// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::epoch_ending::manifest::EpochEndingBackup,
    metrics::restore::{EPOCH_ENDING_EPOCH, EPOCH_ENDING_VERSION},
    storage::{BackupStorage, FileHandle},
    utils::{
        read_record_bytes::ReadRecordBytes, storage_ext::BackupStorageExt, GlobalRestoreOptions,
        RestoreRunMode,
    },
};
use anyhow::{anyhow, ensure, Result};
use libra_logger::prelude::*;
use libra_types::{
    epoch_change::Verifier,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::Version,
    waypoint::Waypoint,
};
use std::sync::Arc;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct EpochEndingRestoreOpt {
    #[structopt(long = "epoch-ending-manifest")]
    pub manifest_handle: FileHandle,
}

pub struct EpochEndingRestoreController {
    storage: Arc<dyn BackupStorage>,
    run_mode: Arc<RestoreRunMode>,
    manifest_handle: FileHandle,
    target_version: Version,
    previous_epoch_ending_ledger_info: Option<LedgerInfo>,
}

impl EpochEndingRestoreController {
    pub fn new(
        opt: EpochEndingRestoreOpt,
        global_opt: GlobalRestoreOptions,
        storage: Arc<dyn BackupStorage>,
        previous_epoch_ending_ledger_info: Option<LedgerInfo>,
    ) -> Self {
        Self {
            storage,
            run_mode: global_opt.run_mode,
            manifest_handle: opt.manifest_handle,
            target_version: global_opt.target_version,
            previous_epoch_ending_ledger_info,
        }
    }

    pub async fn run(self) -> Result<Vec<LedgerInfo>> {
        let name = self.name();
        info!("{} started. Manifest: {}", name, self.manifest_handle);
        let res = self
            .run_impl()
            .await
            .map_err(|e| anyhow!("{} failed: {}", name, e))?;
        info!("{} succeeded.", name);
        Ok(res)
    }
}

impl EpochEndingRestoreController {
    fn name(&self) -> String {
        format!("epoch ending {}", self.run_mode.name())
    }

    async fn run_impl(self) -> Result<Vec<LedgerInfo>> {
        let manifest: EpochEndingBackup =
            self.storage.load_json_file(&self.manifest_handle).await?;
        manifest.verify()?;

        let mut next_epoch = manifest.first_epoch;
        let mut waypoint_iter = manifest.waypoints.iter();

        let mut previous_li = self.previous_epoch_ending_ledger_info.clone();
        if let Some(li) = &previous_li {
            ensure!(
                li.next_block_epoch() == next_epoch,
                "Previous epoch ending LedgerInfo is not the one expected. \
                My first epoch: {}, previous LedgerInfo next_block_epoch: {}",
                next_epoch,
                li.next_block_epoch(),
            );
        }

        let mut output_lis = Vec::new();

        for chunk in manifest.chunks {
            let mut lis = self.read_chunk(chunk.ledger_infos).await?;
            ensure!(
                chunk.first_epoch + lis.len() as u64 == chunk.last_epoch + 1,
                "Number of items in chunks doesn't match that in manifest. first_epoch: {}, last_epoch: {}, items in chunk: {}",
                chunk.first_epoch,
                chunk.last_epoch,
                lis.len(),
            );
            // verify
            let mut previous_li_ref = previous_li.as_ref();
            for li in lis.iter() {
                ensure!(
                    li.ledger_info().epoch() == next_epoch,
                    "LedgerInfo epoch not expected. Expected: {}, actual: {}.",
                    li.ledger_info().epoch(),
                    next_epoch,
                );
                let wp_manifest = waypoint_iter.next().ok_or_else(|| {
                    anyhow!("More LedgerInfo's found than waypoints in manifest.")
                })?;
                let wp_li = Waypoint::new_epoch_boundary(li.ledger_info())?;
                ensure!(
                    *wp_manifest == wp_li,
                    "Waypoints don't match. In manifest: {}, In chunk: {}",
                    wp_manifest,
                    wp_li,
                );
                if let Some(pre_li) = &previous_li_ref {
                    pre_li
                        .next_epoch_state()
                        .ok_or_else(|| {
                            anyhow!(
                                "Next epoch state not found from LI at epoch {}.",
                                pre_li.epoch()
                            )
                        })?
                        .verify(li)?;
                }
                previous_li_ref = Some(li.ledger_info());
                next_epoch += 1;
            }
            previous_li = previous_li_ref.cloned();

            let mut end = lis.len(); // To apply: "[0, end)"
            if let Some(_end) = lis
                .iter()
                .position(|li| li.ledger_info().version() > self.target_version)
            {
                info!(
                    "Ignoring epoch ending info beyond target_version. Epoch {} ends at {}, target_version: {}.",
                    lis[_end].ledger_info().epoch(),
                    lis[_end].ledger_info().version(),
                    self.target_version,
                );
                end = _end;
            }
            let should_skip_next = lis.drain(end..).count() > 0;

            // write to db
            if !lis.is_empty() {
                match self.run_mode.as_ref() {
                    RestoreRunMode::Restore { restore_handler } => {
                        restore_handler.save_ledger_infos(&lis)?;
                        EPOCH_ENDING_EPOCH.set(
                            lis.last()
                                .expect("Verified not empty.")
                                .ledger_info()
                                .epoch() as i64,
                        );
                        EPOCH_ENDING_VERSION.set(
                            lis.last()
                                .expect("Verified not empty.")
                                .ledger_info()
                                .version() as i64,
                        );
                    }
                    RestoreRunMode::Verify => {
                        // add counters
                    }
                };
                output_lis.extend(lis.into_iter().map(|x| x.ledger_info().clone()));
            }

            // skip remaining chunks if beyond target_version
            if should_skip_next {
                break;
            }
        }

        Ok(output_lis)
    }

    async fn read_chunk(&self, file_handle: FileHandle) -> Result<Vec<LedgerInfoWithSignatures>> {
        let mut file = self.storage.open_for_read(&file_handle).await?;
        let mut chunk = vec![];

        while let Some(record_bytes) = file.read_record_bytes().await? {
            chunk.push(lcs::from_bytes(&record_bytes)?);
        }

        Ok(chunk)
    }
}

/// Represents a history of epoch changes since epoch 0.
#[derive(Clone)]
pub struct EpochHistory {
    epoch_endings: Vec<LedgerInfo>,
}

impl EpochHistory {
    pub fn verify_ledger_info(&self, li_with_sigs: &LedgerInfoWithSignatures) -> Result<()> {
        let epoch = li_with_sigs.ledger_info().epoch();
        ensure!(!self.epoch_endings.is_empty(), "Empty epoch history.",);
        ensure!(
            epoch <= self.epoch_endings.len() as u64,
            "History until epoch {} can't verify epoch {}",
            self.epoch_endings.len(),
            epoch,
        );
        if epoch == 0 {
            ensure!(
                li_with_sigs.ledger_info() == &self.epoch_endings[0],
                "Genesis epoch LedgerInfo info doesn't match.",
            );
            Ok(())
        } else {
            self.epoch_endings[epoch as usize - 1]
                .next_epoch_state()
                .ok_or_else(|| anyhow!("Shouldn't contain non- epoch bumping LIs."))?
                .verify(li_with_sigs)
        }
    }
}

pub struct EpochHistoryRestoreController {
    storage: Arc<dyn BackupStorage>,
    manifest_handles: Vec<FileHandle>,
    global_opt: GlobalRestoreOptions,
}

impl EpochHistoryRestoreController {
    pub fn new(
        manifest_handles: Vec<FileHandle>,
        global_opt: GlobalRestoreOptions,
        storage: Arc<dyn BackupStorage>,
    ) -> Self {
        Self {
            storage,
            manifest_handles,
            global_opt,
        }
    }

    pub async fn run(self) -> Result<EpochHistory> {
        let name = self.name();
        info!(
            "{} started. Trying epoch endings for epoch 0 till {} (inclusive)",
            name,
            self.manifest_handles.len() - 1,
        );
        let res = self
            .run_impl()
            .await
            .map_err(|e| anyhow!("{} failed: {}", name, e))?;
        info!("{} succeeded.", name);
        Ok(res)
    }
}

impl EpochHistoryRestoreController {
    fn name(&self) -> String {
        format!("epoch history {}", self.global_opt.run_mode.name())
    }

    async fn run_impl(self) -> Result<EpochHistory> {
        let mut next_epoch = 0u64;
        let mut previous_li = None;
        let mut epoch_endings = Vec::new();

        for manifest_handle in &self.manifest_handles {
            let lis = EpochEndingRestoreController::new(
                EpochEndingRestoreOpt {
                    manifest_handle: manifest_handle.clone(),
                },
                self.global_opt.clone(),
                self.storage.clone(),
                previous_li.clone(),
            )
            .run()
            .await?;

            ensure!(
                !lis.is_empty(),
                "No epochs restored from {}",
                manifest_handle,
            );
            for li in &lis {
                ensure!(
                    li.epoch() == next_epoch,
                    "Restored LedgerInfo has epoch {}, expecting {}.",
                    li.epoch(),
                    next_epoch,
                );
                ensure!(
                    li.ends_epoch(),
                    "LedgerInfo is not one at an epoch ending. epoch: {}",
                    li.epoch(),
                );
                next_epoch += 1;
            }

            previous_li = Some(lis.last().expect("Verified not empty.").clone());
            epoch_endings.extend(lis);
        }
        ensure!(
            !epoch_endings.is_empty(),
            "No epochs restored from {} manifests",
            self.manifest_handles.len(),
        );

        Ok(EpochHistory { epoch_endings })
    }
}
