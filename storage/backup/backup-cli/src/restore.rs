// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    manifest::state_snapshot::StateSnapshotBackup,
    storage::{BackupStorage, FileHandle},
    ReadRecordBytes,
};
use anyhow::Result;
use libra_crypto::HashValue;
use libra_types::{
    account_state_blob::AccountStateBlob, proof::SparseMerkleRangeProof, transaction::Version,
};
use libradb::LibraDB;
use std::{path::PathBuf, sync::Arc};
use structopt::StructOpt;
use tokio::io::AsyncReadExt;

#[derive(StructOpt)]
pub struct GlobalRestoreOpt {
    #[structopt(long = "target-db-dir", parse(from_os_str))]
    pub db_dir: PathBuf,
}

#[derive(StructOpt)]
pub struct StateSnapshotRestoreOpt {
    #[structopt(long = "state-manifest")]
    pub manifest_handle: FileHandle,
    #[structopt(long = "state-into-version")]
    pub version: Version,
}

pub struct StateSnapshotRestoreController {
    storage: Arc<dyn BackupStorage>,
    db: Arc<LibraDB>,
    version: Version,
    manifest_handle: FileHandle,
}

impl StateSnapshotRestoreController {
    pub fn new(
        opt: StateSnapshotRestoreOpt,
        storage: Arc<dyn BackupStorage>,
        db: Arc<LibraDB>,
    ) -> Self {
        Self {
            storage,
            db,
            version: opt.version,
            manifest_handle: opt.manifest_handle,
        }
    }

    pub async fn run(self) -> Result<()> {
        let mut manifest_bytes = Vec::new();
        self.storage
            .open_for_read(&self.manifest_handle)
            .await?
            .read_to_end(&mut manifest_bytes)
            .await?;
        let manifest: StateSnapshotBackup = serde_json::from_slice(&manifest_bytes)?;

        let mut receiver = self
            .db
            .get_state_restore_receiver(self.version, manifest.root_hash)?;

        for chunk in manifest.chunks {
            let blobs = self.read_account_state_chunk(chunk.blobs).await?;
            let proof = self.read_proof(chunk.proof).await?;

            receiver.add_chunk(blobs, proof)?;
        }

        receiver.finish()?;
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

    async fn read_proof(&self, file_handle: FileHandle) -> Result<SparseMerkleRangeProof> {
        let mut file = self.storage.open_for_read(&file_handle).await?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;

        let proof = lcs::from_bytes(&buf)?;
        Ok(proof)
    }
}
