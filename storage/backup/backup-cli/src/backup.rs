// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    storage::{BackupHandleRef, BackupStorage, FileHandle},
    ReadRecordBytes,
};
use anyhow::Result;
use bytes::Bytes;
use futures::stream::TryStreamExt;
use libra_crypto::HashValue;
use libra_types::{account_state_blob::AccountStateBlob, transaction::Version};
use std::mem::size_of;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio_util::compat::FuturesAsyncReadCompatExt;

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

    async fn get(&self, path: &str) -> Result<impl AsyncRead> {
        Ok(self
            .client
            .get(&format!("http://localhost:{}/{}", self.port, path))
            .send()
            .await?
            .error_for_status()?
            .bytes_stream()
            .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
            .into_async_read()
            .compat())
    }

    pub async fn get_latest_state_root(&self) -> Result<(Version, HashValue)> {
        let mut buf = Vec::new();
        self.get("latest_state_root")
            .await?
            .read_to_end(&mut buf)
            .await?;
        Ok(lcs::from_bytes(&buf)?)
    }

    async fn get_account_range_proof(
        &self,
        key: HashValue,
        version: Version,
    ) -> Result<impl AsyncRead> {
        self.get(&format!("state_range_proof/{}/{:x}", version, key))
            .await
    }

    async fn get_state_snapshot(&self, version: Version) -> Result<impl AsyncRead> {
        self.get(&format!("state_snapshot/{}", version)).await
    }
}

fn backup_name(version: Version) -> String {
    format!("state_ver_{}", version)
}

fn chunk_name(first_idx: usize) -> String {
    format!("{}-.chunk", first_idx)
}

fn chunk_proof_name(first_idx: usize, last_idx: usize) -> String {
    format!("{}-{}.proof", first_idx, last_idx)
}

pub async fn backup_account_state(
    client: &BackupServiceClient,
    version: Version,
    storage: &impl BackupStorage,
    max_chunk_size: usize,
) -> Result<Vec<(FileHandle, FileHandle)>> {
    let backup_handle = storage.create_backup(&backup_name(version)).await?;

    let mut chunk_bytes = vec![];
    let mut ret = vec![];
    let mut current_idx: usize = 0;
    let mut chunk_first_idx: usize = 0;

    let mut state_snapshot_file = client.get_state_snapshot(version).await?;
    let mut prev_record_bytes: Option<Bytes> = None;
    while let Some(record_bytes) = state_snapshot_file.read_record_bytes().await? {
        if chunk_bytes.len() + size_of::<u32>() + record_bytes.len() > max_chunk_size {
            assert!(chunk_bytes.len() <= max_chunk_size);
            println!("Reached max_chunk_size.");

            let (chunk_handle, proof_handle) = write_chunk(
                client,
                version,
                storage,
                &backup_handle,
                &chunk_bytes,
                chunk_first_idx,
                current_idx,
                &prev_record_bytes.expect("max_chunk_size should be larger than account size."),
            )
            .await?;
            ret.push((chunk_handle, proof_handle));
            chunk_bytes = vec![];
            chunk_first_idx = current_idx + 1;
        }

        current_idx += 1;
        chunk_bytes.extend(&(record_bytes.len() as u32).to_be_bytes());
        chunk_bytes.extend(&record_bytes);
        prev_record_bytes = Some(record_bytes);
    }

    assert!(!chunk_bytes.is_empty());
    assert!(chunk_bytes.len() <= max_chunk_size);
    println!("Last chunk.");
    let (chunk_handle, proof_handle) = write_chunk(
        client,
        version,
        storage,
        &backup_handle,
        &chunk_bytes,
        chunk_first_idx,
        current_idx,
        &prev_record_bytes.expect("Should have at least one account."),
    )
    .await?;
    ret.push((chunk_handle, proof_handle));

    Ok(ret)
}

async fn write_chunk(
    client: &BackupServiceClient,
    version: u64,
    storage: &impl BackupStorage,
    backup_handle: &BackupHandleRef,
    chunk_bytes: &[u8],
    chunk_first_idx: usize,
    current_idx: usize,
    prev_record_bytes: &Bytes,
) -> Result<(FileHandle, FileHandle)> {
    let (prev_key, _): (HashValue, AccountStateBlob) = lcs::from_bytes(prev_record_bytes)?;
    println!("Asking proof for key: {:?}", prev_key,);
    let (chunk_handle, mut chunk_file) = storage
        .create_for_write(&backup_handle, &chunk_name(chunk_first_idx))
        .await?;
    chunk_file.write_all(&chunk_bytes).await?;
    let (proof_handle, mut proof_file) = storage
        .create_for_write(
            &backup_handle,
            &chunk_proof_name(chunk_first_idx, current_idx),
        )
        .await?;
    tokio::io::copy(
        &mut client.get_account_range_proof(prev_key, version).await?,
        &mut proof_file,
    )
    .await?;
    Ok((chunk_handle, proof_handle))
}
