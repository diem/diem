// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod adapter;

use crate::adapter::Adapter;
use anyhow::Result;
use futures::{stream, StreamExt};
use libra_crypto::HashValue;
use libra_types::transaction::Version;
use storage_client::{StorageRead, StorageReadServiceClient};

pub type FileHandle = String;

pub async fn backup_account_state(
    client: &StorageReadServiceClient,
    version: Version,
    adapter: &impl Adapter,
    max_chunk_size: usize,
) -> Result<Vec<(FileHandle, FileHandle)>> {
    let mut chunk = vec![];
    let mut ret = vec![];

    let mut account_stream = client.backup_account_state(version)?;
    let mut prev_key = None;
    while let Some(resp) = account_stream.next().await.transpose()? {
        let key = resp.account_key;
        let blob = resp.account_state_blob;
        println!("Backing up key: {:x}", key);

        let mut bytes = key.to_vec();
        let blob: Vec<u8> = blob.into();
        assert!(blob.len() <= std::u32::MAX as usize);
        let blob_len = blob.len() as u32;
        bytes.extend_from_slice(&blob_len.to_le_bytes());
        bytes.extend(blob);
        assert!(bytes.len() <= max_chunk_size);

        if chunk.len() + bytes.len() > max_chunk_size {
            assert!(chunk.len() <= max_chunk_size);
            let account_state_file = adapter
                .write_new_file(stream::once(async move { chunk }))
                .await?;

            let prev_key = prev_key.expect("max_chunk_size should be larger than account size.");
            println!(
                "Reached max_chunk_size. Asking proof for key: {:?}",
                prev_key,
            );
            let proof_file = get_proof_and_write(client, adapter, prev_key, version).await?;
            ret.push((account_state_file, proof_file));
            chunk = vec![];
        }

        chunk.extend(bytes);
        prev_key = Some(key);
    }

    assert!(!chunk.is_empty());
    assert!(chunk.len() <= max_chunk_size);
    let account_state_file = adapter
        .write_new_file(stream::once(async move { chunk }))
        .await?;

    let prev_key = prev_key.expect("Should have at least one account.");
    println!("Asking proof for last key: {:x}", prev_key);
    let proof_file = get_proof_and_write(client, adapter, prev_key, version).await?;
    ret.push((account_state_file, proof_file));

    Ok(ret)
}

async fn get_proof_and_write(
    client: &StorageReadServiceClient,
    adapter: &impl Adapter,
    key: HashValue,
    version: Version,
) -> Result<FileHandle> {
    let proof = client.get_account_state_range_proof(key, version).await?;
    let proof_bytes: Vec<u8> = lcs::to_bytes(&proof)?;
    let file = adapter
        .write_new_file(stream::once(async move { proof_bytes }))
        .await?;
    Ok(file)
}
