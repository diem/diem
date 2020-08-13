// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::epoch_ending::manifest::EpochEndingBackup,
    metrics::restore::{EPOCH_ENDING_EPOCH, EPOCH_ENDING_VERSION},
    storage::{BackupStorage, FileHandle},
    utils::{read_record_bytes::ReadRecordBytes, storage_ext::BackupStorageExt, GlobalRestoreOpt},
};
use anyhow::{anyhow, ensure, Result};
use libra_types::{
    ledger_info::LedgerInfoWithSignatures, transaction::Version, waypoint::Waypoint,
};
use libradb::backup::restore_handler::RestoreHandler;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct EpochEndingRestoreOpt {
    #[structopt(long = "epoch-ending-manifest")]
    pub manifest_handle: FileHandle,
}

pub struct EpochEndingRestoreController {
    storage: Arc<dyn BackupStorage>,
    restore_handler: Arc<RestoreHandler>,
    manifest_handle: FileHandle,
    target_version: Version,
}

impl EpochEndingRestoreController {
    pub fn new(
        opt: EpochEndingRestoreOpt,
        global_opt: GlobalRestoreOpt,
        storage: Arc<dyn BackupStorage>,
        restore_handler: Arc<RestoreHandler>,
    ) -> Self {
        Self {
            storage,
            restore_handler,
            manifest_handle: opt.manifest_handle,
            target_version: global_opt.target_version(),
        }
    }

    pub async fn run(self) -> Result<()> {
        println!(
            "Epoch ending restore started. Manifest: {}",
            self.manifest_handle
        );
        self.run_impl()
            .await
            .map_err(|e| anyhow!("Epoch ending restore failed: {}", e))?;
        println!("Epoch ending restore succeeded.");
        Ok(())
    }
}

impl EpochEndingRestoreController {
    async fn run_impl(self) -> Result<()> {
        let manifest: EpochEndingBackup =
            self.storage.load_json_file(&self.manifest_handle).await?;
        manifest.verify()?;

        let mut next_epoch = manifest.first_epoch;
        let mut waypoint_iter = manifest.waypoints.iter();

        for chunk in manifest.chunks {
            let lis = self.read_chunk(chunk.ledger_infos).await?;
            ensure!(
                chunk.first_epoch + lis.len() as u64 == chunk.last_epoch + 1,
                "Number of items in chunks doesn't match that in manifest. first_epoch: {}, last_epoch: {}, items in chunk: {}",
                chunk.first_epoch,
                chunk.last_epoch,
                lis.len(),
            );
            // verify
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
                // TODO: verify signature on li
                ensure!(
                    *wp_manifest == wp_li,
                    "Waypoints don't match. In manifest: {}, In chunk: {}",
                    wp_manifest,
                    wp_li,
                );
                next_epoch += 1;
            }

            let mut end = lis.len(); // To apply: "[0, end)"
            if let Some(_end) = lis
                .iter()
                .position(|li| li.ledger_info().version() > self.target_version)
            {
                println!(
                    "Ignoring epoch ending info beyond target_version. Epoch {} ends at {}, target_version: {}.",
                    lis[_end].ledger_info().epoch(),
                    lis[_end].ledger_info().version(),
                    self.target_version,
                );
                end = _end;
            }

            // write to db
            if end != 0 {
                self.restore_handler.save_ledger_infos(&lis[..end])?;
                EPOCH_ENDING_EPOCH.set(lis[end - 1].ledger_info().epoch() as i64);
                EPOCH_ENDING_VERSION.set(lis[end - 1].ledger_info().version() as i64);
            }

            // skip remaining chunks if beyond target_version
            if end < lis.len() {
                break;
            }
        }
        println!("Finished restoring epoch ending info.");

        Ok(())
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
