// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::state_snapshot::manifest::StateSnapshotBackup,
    storage::{BackupStorage, FileHandle},
    utils::{read_record_bytes::ReadRecordBytes, storage_ext::BackupStorageExt, GlobalRestoreOpt},
};
use anyhow::Result;
use libra_crypto::HashValue;
use libra_types::{account_state_blob::AccountStateBlob, transaction::Version};
use libradb::backup::restore_handler::RestoreHandler;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct StateSnapshotRestoreOpt {
    #[structopt(long = "state-manifest")]
    pub manifest_handle: FileHandle,
    #[structopt(long = "state-into-version")]
    pub version: Version,
}

pub struct StateSnapshotRestoreController {
    storage: Arc<dyn BackupStorage>,
    restore_handler: Arc<RestoreHandler>,
    // State snapshot restore to this version
    version: Version,
    manifest_handle: FileHandle,
    // Global "target_version" for the entire restore process, if `version` is newer than this,
    // nothing will be done.
    target_version: Version,
}

impl StateSnapshotRestoreController {
    pub fn new(
        opt: StateSnapshotRestoreOpt,
        global_opt: GlobalRestoreOpt,
        storage: Arc<dyn BackupStorage>,
        restore_handler: Arc<RestoreHandler>,
    ) -> Self {
        Self {
            storage,
            restore_handler,
            version: opt.version,
            manifest_handle: opt.manifest_handle,
            target_version: global_opt.target_version,
        }
    }

    pub async fn run(self) -> Result<()> {
        if self.version > self.target_version {
            println!(
                "Trying to restore state snapshot to version {}, which is newer than the target version {}.",
                self.version,
                self.target_version,
            );
            return Ok(());
        }

        let manifest: StateSnapshotBackup =
            self.storage.load_json_file(&self.manifest_handle).await?;

        let mut receiver = self
            .restore_handler
            .get_state_restore_receiver(self.version, manifest.root_hash)?;

        for chunk in manifest.chunks {
            let blobs = self.read_account_state_chunk(chunk.blobs).await?;
            let proof = self.storage.load_lcs_file(&chunk.proof).await?;

            receiver.add_chunk(blobs, proof)?;
        }

        receiver.finish()?;
        println!("Finished restoring state snapshot.");
        Ok(())
    }
}

impl StateSnapshotRestoreController {
    async fn read_account_state_chunk(
        &self,
        file_handle: FileHandle,
    ) -> Result<Vec<(HashValue, AccountStateBlob)>> {
        let mut file = self.storage.open_for_read(&file_handle).await?;

        let mut chunk = vec![];

        while let Some(record_bytes) = file.read_record_bytes().await? {
            chunk.push(lcs::from_bytes(&record_bytes)?);
        }

        Ok(chunk)
    }
}
