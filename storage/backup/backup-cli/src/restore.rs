// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    storage::{local_fs::LocalFs, BackupStorage, FileHandle},
    ReadRecordBytes,
};
use anyhow::Result;
use libra_crypto::HashValue;
use libra_types::{account_state_blob::AccountStateBlob, proof::SparseMerkleRangeProof};
use libradb::LibraDB;
use std::path::Path;
use tokio::io::AsyncReadExt;

pub async fn restore_account_state<P, I>(
    version: u64,
    root_hash: HashValue,
    db_dir: P,
    iter: I,
) -> Result<()>
where
    P: AsRef<Path> + Clone,
    I: Iterator<Item = (FileHandle, FileHandle)>,
{
    let libradb = LibraDB::open(db_dir, false /* read_only */, None /* pruner */)?;
    let mut receiver = libradb.get_state_restore_receiver(version, root_hash)?;

    for (chunk_handle, proof_handle) in iter {
        let chunk = read_account_state_chunk(chunk_handle).await?;
        let proof = read_proof(proof_handle).await?;

        receiver.add_chunk(chunk, proof)?;
    }

    receiver.finish()?;
    Ok(())
}

async fn read_account_state_chunk(
    file_handle: FileHandle,
) -> Result<Vec<(HashValue, AccountStateBlob)>> {
    let mut file = LocalFs::open_for_read(&file_handle).await?;

    let mut chunk = vec![];

    while let Some(record_bytes) = file.read_record_bytes().await? {
        chunk.push(lcs::from_bytes(&record_bytes)?);
    }

    Ok(chunk)
}

async fn read_proof(file_handle: FileHandle) -> Result<SparseMerkleRangeProof> {
    let mut file = LocalFs::open_for_read(&file_handle).await?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).await?;

    let proof = lcs::from_bytes(&buf)?;
    Ok(proof)
}
