// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::{
        epoch_ending::restore::EpochHistory, state_snapshot::manifest::StateSnapshotBackup,
    },
    metrics::{
        restore::{
            STATE_SNAPSHOT_LEAF_INDEX, STATE_SNAPSHOT_TARGET_LEAF_INDEX, STATE_SNAPSHOT_VERSION,
        },
        verify::{
            VERIFY_STATE_SNAPSHOT_LEAF_INDEX, VERIFY_STATE_SNAPSHOT_TARGET_LEAF_INDEX,
            VERIFY_STATE_SNAPSHOT_VERSION,
        },
    },
    storage::{BackupStorage, FileHandle},
    utils::{
        read_record_bytes::ReadRecordBytes, storage_ext::BackupStorageExt, GlobalRestoreOptions,
        RestoreRunMode,
    },
};
use anyhow::{anyhow, ensure, Result};
use diem_crypto::HashValue;
use diem_logger::prelude::*;
use diem_types::{
    account_state_blob::AccountStateBlob, ledger_info::LedgerInfoWithSignatures,
    proof::TransactionInfoWithProof, transaction::Version,
};
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
    run_mode: Arc<RestoreRunMode>,
    /// State snapshot restores to this version.
    version: Version,
    manifest_handle: FileHandle,
    /// Global "target_version" for the entire restore process, if `version` is newer than this,
    /// nothing will be done, otherwise, this has no effect.
    target_version: Version,
    epoch_history: Option<Arc<EpochHistory>>,
}

impl StateSnapshotRestoreController {
    pub fn new(
        opt: StateSnapshotRestoreOpt,
        global_opt: GlobalRestoreOptions,
        storage: Arc<dyn BackupStorage>,
        epoch_history: Option<Arc<EpochHistory>>,
    ) -> Self {
        Self {
            storage,
            run_mode: global_opt.run_mode,
            version: opt.version,
            manifest_handle: opt.manifest_handle,
            target_version: global_opt.target_version,
            epoch_history,
        }
    }

    pub async fn run(self) -> Result<()> {
        let name = self.name();
        info!("{} started. Manifest: {}", name, self.manifest_handle);
        self.run_impl()
            .await
            .map_err(|e| anyhow!("{} failed: {}", name, e))?;
        info!("{} succeeded.", name);
        Ok(())
    }
}

impl StateSnapshotRestoreController {
    fn name(&self) -> String {
        format!("state snapshot {}", self.run_mode.name())
    }

    async fn run_impl(self) -> Result<()> {
        if self.version > self.target_version {
            warn!(
                "Trying to restore state snapshot to version {}, which is newer than the target version {}, skipping.",
                self.version,
                self.target_version,
            );
            return Ok(());
        }

        let manifest: StateSnapshotBackup =
            self.storage.load_json_file(&self.manifest_handle).await?;
        let (txn_info_with_proof, li): (TransactionInfoWithProof, LedgerInfoWithSignatures) =
            self.storage.load_lcs_file(&manifest.proof).await?;
        txn_info_with_proof.verify(li.ledger_info(), manifest.version)?;
        ensure!(
            txn_info_with_proof.transaction_info().state_root_hash() == manifest.root_hash,
            "Root hash mismatch with that in proof. root hash: {}, expected: {}",
            manifest.root_hash,
            txn_info_with_proof.transaction_info().state_root_hash(),
        );
        if let Some(epoch_history) = self.epoch_history.as_ref() {
            epoch_history.verify_ledger_info(&li)?;
        }

        let mut receiver = self
            .run_mode
            .get_state_restore_receiver(self.version, manifest.root_hash)?;

        let (ver_gauge, tgt_leaf_idx, leaf_idx) = if self.run_mode.is_verify() {
            (
                &VERIFY_STATE_SNAPSHOT_VERSION,
                &VERIFY_STATE_SNAPSHOT_TARGET_LEAF_INDEX,
                &VERIFY_STATE_SNAPSHOT_LEAF_INDEX,
            )
        } else {
            (
                &STATE_SNAPSHOT_VERSION,
                &STATE_SNAPSHOT_TARGET_LEAF_INDEX,
                &STATE_SNAPSHOT_LEAF_INDEX,
            )
        };

        // FIXME update counters
        ver_gauge.set(self.version as i64);
        tgt_leaf_idx.set(manifest.chunks.last().map_or(0, |c| c.last_idx as i64));
        for chunk in manifest.chunks {
            let blobs = self.read_account_state_chunk(chunk.blobs).await?;
            let proof = self.storage.load_lcs_file(&chunk.proof).await?;

            receiver.add_chunk(blobs, proof)?;
            leaf_idx.set(chunk.last_idx as i64);
        }

        receiver.finish()?;
        Ok(())
    }

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
