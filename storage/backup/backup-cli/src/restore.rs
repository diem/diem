// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    manifest::state_snapshot::StateSnapshotBackup,
    storage::{BackupStorage, FileHandle, FileHandleRef},
    ReadRecordBytes,
};
use anyhow::Result;
use libra_crypto::HashValue;
use libra_types::{account_state_blob::AccountStateBlob, proof::SparseMerkleRangeProof};
use libradb::LibraDB;
use std::path::Path;
use tokio::io::AsyncReadExt;

pub async fn restore_account_state<P>(
    storage: &dyn BackupStorage,
    manifest_handle: &FileHandleRef,
    version: u64,
    db_dir: P,
) -> Result<()>
where
    P: AsRef<Path> + Clone,
{
    let mut manifest_bytes = Vec::new();
    storage
        .open_for_read(manifest_handle)
        .await?
        .read_to_end(&mut manifest_bytes)
        .await?;
    let manifest: StateSnapshotBackup = serde_json::from_slice(&manifest_bytes)?;

    let libradb = LibraDB::open(db_dir, false /* read_only */, None /* pruner */)?;
    let mut receiver = libradb.get_state_restore_receiver(version, manifest.root_hash)?;

    for chunk in manifest.chunks {
        let blobs = read_account_state_chunk(storage, chunk.blobs).await?;
        let proof = read_proof(storage, chunk.proof).await?;

        receiver.add_chunk(blobs, proof)?;
    }

    receiver.finish()?;
    Ok(())
}

async fn read_account_state_chunk(
    storage: &dyn BackupStorage,
    file_handle: FileHandle,
) -> Result<Vec<(HashValue, AccountStateBlob)>> {
    let mut file = storage.open_for_read(&file_handle).await?;

    let mut chunk = vec![];

    while let Some(record_bytes) = file.read_record_bytes().await? {
        chunk.push(lcs::from_bytes(&record_bytes)?);
    }

    Ok(chunk)
}

async fn read_proof(
    storage: &dyn BackupStorage,
    file_handle: FileHandle,
) -> Result<SparseMerkleRangeProof> {
    let mut file = storage.open_for_read(&file_handle).await?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).await?;

    let proof = lcs::from_bytes(&buf)?;
    Ok(proof)
}
