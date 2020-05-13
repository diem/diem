// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{adapter::Adapter, FileHandle};
use anyhow::Result;
use bytes::Bytes;
use futures::{stream, StreamExt};
use libra_crypto::HashValue;
use libra_types::transaction::Version;
use storage_client::{StorageRead, StorageReadServiceClient};

pub struct BackupServiceClient {
    port: u16,
    client: reqwest::Client,
}

impl BackupServiceClient {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            client: reqwest::Client::new(),
        }
    }

    async fn get_account_range_proof(&self, key: HashValue, version: Version) -> Result<Bytes> {
        Ok(self
            .client
            .get(&format!(
                "http://localhost:{}/state_range_proof/{}/{:x}",
                self.port, version, key,
            ))
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?)
    }
}

pub async fn backup_account_state(
    client: &StorageReadServiceClient,
    backup_service_client: &BackupServiceClient,
    version: Version,
    adapter: &impl Adapter,
    max_chunk_size: usize,
) -> Result<Vec<(FileHandle, FileHandle)>> {
    let mut chunk = vec![];
    let mut ret = vec![];

    let mut account_stream = client.backup_account_state(version).await?;
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
            let proof_file =
                get_proof_and_write(backup_service_client, adapter, prev_key, version).await?;
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
    let proof_file = get_proof_and_write(backup_service_client, adapter, prev_key, version).await?;
    ret.push((account_state_file, proof_file));

    Ok(ret)
}

async fn get_proof_and_write(
    backup_service_client: &BackupServiceClient,
    adapter: &impl Adapter,
    key: HashValue,
    version: Version,
) -> Result<FileHandle> {
    let proof_bytes = backup_service_client
        .get_account_range_proof(key, version)
        .await?;
    let file = adapter
        .write_new_file(stream::once(async move { proof_bytes.to_vec() }))
        .await?;
    Ok(file)
}
