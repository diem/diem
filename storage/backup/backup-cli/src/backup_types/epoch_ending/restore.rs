// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::epoch_ending::manifest::EpochEndingBackup,
    metrics::{
        restore::{EPOCH_ENDING_EPOCH, EPOCH_ENDING_VERSION},
        verify::{VERIFY_EPOCH_ENDING_EPOCH, VERIFY_EPOCH_ENDING_VERSION},
    },
    storage::{BackupStorage, FileHandle, FileHandleRef},
    utils::{
        read_record_bytes::ReadRecordBytes, storage_ext::BackupStorageExt, stream::StreamX,
        GlobalRestoreOptions, RestoreRunMode,
    },
};
use anyhow::{anyhow, ensure, Result};
use diem_logger::prelude::*;
use diem_types::{
    epoch_change::Verifier,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    transaction::Version,
    waypoint::Waypoint,
};
use futures::StreamExt;
use std::{collections::HashMap, sync::Arc, time::Instant};
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
    trusted_waypoints: Arc<HashMap<Version, Waypoint>>,
}

impl EpochEndingRestoreController {
    pub fn new(
        opt: EpochEndingRestoreOpt,
        global_opt: GlobalRestoreOptions,
        storage: Arc<dyn BackupStorage>,
    ) -> Self {
        Self {
            storage,
            run_mode: global_opt.run_mode,
            manifest_handle: opt.manifest_handle,
            target_version: global_opt.target_version,
            trusted_waypoints: global_opt.trusted_waypoints,
        }
    }

    pub async fn preheat(self) -> PreheatedEpochEndingRestore {
        PreheatedEpochEndingRestore {
            preheat_result: self.preheat_impl().await,
            controller: self,
        }
    }

    pub async fn run(
        self,
        previous_epoch_ending_ledger_info: Option<&LedgerInfo>,
    ) -> Result<Vec<LedgerInfo>> {
        self.preheat()
            .await
            .run(previous_epoch_ending_ledger_info)
            .await
    }
}

impl EpochEndingRestoreController {
    fn name(&self) -> String {
        format!("epoch ending {}", self.run_mode.name())
    }

    async fn preheat_impl(&self) -> Result<EpochEndingRestorePreheatData> {
        let manifest: EpochEndingBackup =
            self.storage.load_json_file(&self.manifest_handle).await?;
        manifest.verify()?;

        let mut next_epoch = manifest.first_epoch;
        let mut waypoint_iter = manifest.waypoints.iter();

        let mut previous_li: Option<&LedgerInfoWithSignatures> = None;
        let mut ledger_infos = Vec::new();

        let mut past_target = false;
        for chunk in &manifest.chunks {
            if past_target {
                break;
            }

            let lis = self.read_chunk(&chunk.ledger_infos).await?;
            ensure!(
                chunk.first_epoch + lis.len() as u64 == chunk.last_epoch + 1,
                "Number of items in chunks doesn't match that in manifest. \
                first_epoch: {}, last_epoch: {}, items in chunk: {}",
                chunk.first_epoch,
                chunk.last_epoch,
                lis.len(),
            );

            for li in lis {
                if li.ledger_info().version() > self.target_version {
                    past_target = true;
                    break;
                }

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
                if let Some(wp_trusted) = self.trusted_waypoints.get(&wp_li.version()) {
                    ensure!(
                        *wp_trusted == wp_li,
                        "Waypoints don't match. In backup: {}, trusted: {}",
                        wp_li,
                        wp_trusted,
                    );
                } else if let Some(pre_li) = previous_li {
                    pre_li
                        .ledger_info()
                        .next_epoch_state()
                        .ok_or_else(|| {
                            anyhow!(
                                "Next epoch state not found from LI at epoch {}.",
                                pre_li.ledger_info().epoch()
                            )
                        })?
                        .verify(&li)?;
                }
                ledger_infos.push(li);
                previous_li = ledger_infos.last();
                next_epoch += 1;
            }
        }

        Ok(EpochEndingRestorePreheatData {
            manifest,
            ledger_infos,
        })
    }

    async fn read_chunk(
        &self,
        file_handle: &FileHandleRef,
    ) -> Result<Vec<LedgerInfoWithSignatures>> {
        let mut file = self.storage.open_for_read(file_handle).await?;
        let mut chunk = vec![];

        while let Some(record_bytes) = file.read_record_bytes().await? {
            chunk.push(bcs::from_bytes(&record_bytes)?);
        }

        Ok(chunk)
    }
}

struct EpochEndingRestorePreheatData {
    manifest: EpochEndingBackup,
    ledger_infos: Vec<LedgerInfoWithSignatures>,
}

pub struct PreheatedEpochEndingRestore {
    controller: EpochEndingRestoreController,
    preheat_result: Result<EpochEndingRestorePreheatData>,
}

impl PreheatedEpochEndingRestore {
    pub async fn run(
        self,
        previous_epoch_ending_ledger_info: Option<&LedgerInfo>,
    ) -> Result<Vec<LedgerInfo>> {
        let name = self.controller.name();
        info!(
            "{} started. Manifest: {}",
            name, self.controller.manifest_handle
        );
        let res = self
            .run_impl(previous_epoch_ending_ledger_info)
            .await
            .map_err(|e| anyhow!("{} failed: {}", name, e))?;
        info!("{} succeeded.", name);
        Ok(res)
    }
}

impl PreheatedEpochEndingRestore {
    async fn run_impl(
        self,
        previous_epoch_ending_ledger_info: Option<&LedgerInfo>,
    ) -> Result<Vec<LedgerInfo>> {
        let preheat_data = self
            .preheat_result
            .map_err(|e| anyhow!("Preheat failed: {}", e))?;

        let first_li = preheat_data
            .ledger_infos
            .first()
            .expect("Epoch ending backup can't be empty.");

        if let Some(li) = previous_epoch_ending_ledger_info {
            ensure!(
                li.next_block_epoch() == preheat_data.manifest.first_epoch,
                "Previous epoch ending LedgerInfo is not the one expected. \
                My first epoch: {}, previous LedgerInfo next_block_epoch: {}",
                preheat_data.manifest.first_epoch,
                li.next_block_epoch(),
            );
            // Waypoint has been verified in preheat if it's trusted, otherwise try to check
            // the signatures.
            if self
                .controller
                .trusted_waypoints
                .get(&first_li.ledger_info().version())
                .is_none()
            {
                li.next_epoch_state()
                    .ok_or_else(|| {
                        anyhow!("Previous epoch ending LedgerInfo doesn't end an epoch")
                    })?
                    .verify(first_li)?;
            }
        }

        let last_li = preheat_data
            .ledger_infos
            .last()
            .expect("Verified not empty.")
            .ledger_info();
        match self.controller.run_mode.as_ref() {
            RestoreRunMode::Restore { restore_handler } => {
                restore_handler.save_ledger_infos(&preheat_data.ledger_infos)?;

                EPOCH_ENDING_EPOCH.set(last_li.epoch() as i64);
                EPOCH_ENDING_VERSION.set(last_li.version() as i64);
            }
            RestoreRunMode::Verify => {
                VERIFY_EPOCH_ENDING_EPOCH.set(last_li.epoch() as i64);
                VERIFY_EPOCH_ENDING_VERSION.set(last_li.version() as i64);
            }
        };

        Ok(preheat_data
            .ledger_infos
            .into_iter()
            .map(|x| x.ledger_info().clone())
            .collect())
    }
}

/// Represents a history of epoch changes since epoch 0.
#[derive(Clone)]
pub struct EpochHistory {
    pub epoch_endings: Vec<LedgerInfo>,
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
            "{} started. Trying epoch endings starting from epoch 0, {} in total.",
            name,
            self.manifest_handles.len(),
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
        let timer = Instant::now();
        if self.manifest_handles.is_empty() {
            return Ok(EpochHistory {
                epoch_endings: Vec::new(),
            });
        }

        let futs_iter = self.manifest_handles.iter().map(|hdl| {
            EpochEndingRestoreController::new(
                EpochEndingRestoreOpt {
                    manifest_handle: hdl.clone(),
                },
                self.global_opt.clone(),
                self.storage.clone(),
            )
            .preheat()
        });
        let mut futs_stream = futures::stream::iter(futs_iter).buffered_x(
            self.global_opt.concurrent_downloads * 2, /* buffer size */
            self.global_opt.concurrent_downloads,     /* concurrency */
        );

        let mut next_epoch = 0u64;
        let mut previous_li = None;
        let mut epoch_endings = Vec::new();

        while let Some(preheated_restore) = futs_stream.next().await {
            let manifest_handle = preheated_restore.controller.manifest_handle.clone();
            let lis = preheated_restore.run(previous_li).await?;
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

            epoch_endings.extend(lis);
            previous_li = epoch_endings.last();
        }

        info!(
            "Epoch history recovered in {:.2} seconds",
            timer.elapsed().as_secs_f64()
        );
        Ok(EpochHistory { epoch_endings })
    }
}
