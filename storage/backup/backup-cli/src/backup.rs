// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{adapter::Adapter, FileHandle, ReadRecordBytes};
use anyhow::Result;
use bytes::Bytes;
use futures::{stream, stream::TryStreamExt};
use libra_crypto::HashValue;
use libra_types::{account_state_blob::AccountStateBlob, transaction::Version};
use tokio::io::{AsyncRead, AsyncReadExt};
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

pub async fn backup_account_state(
    client: &BackupServiceClient,
    version: Version,
    adapter: &impl Adapter,
    max_chunk_size: usize,
) -> Result<Vec<(FileHandle, FileHandle)>> {
    let mut chunk = vec![];
    let mut ret = vec![];

    let mut state_snapshot_stream = client.get_state_snapshot(version).await?;
    let mut prev_record_bytes: Option<Bytes> = None;
    while let Some(record_bytes) = state_snapshot_stream.read_record_bytes().await? {
        if chunk.len() + 4 + record_bytes.len() > max_chunk_size {
            assert!(chunk.len() <= max_chunk_size);
            let account_state_file = adapter
                .write_new_file(stream::once(async move { chunk }))
                .await?;

            let prb =
                prev_record_bytes.expect("max_chunk_size should be larger than account size.");
            let (prev_key, _): (HashValue, AccountStateBlob) = lcs::from_bytes(&prb)?;
            println!(
                "Reached max_chunk_size. Asking proof for key: {:?}",
                prev_key,
            );
            let proof_file = get_proof_and_write(client, adapter, prev_key, version).await?;
            ret.push((account_state_file, proof_file));
            chunk = vec![];
        }

        chunk.extend(&(record_bytes.len() as u32).to_be_bytes());
        chunk.extend(&record_bytes);
        prev_record_bytes = Some(record_bytes);
    }

    assert!(!chunk.is_empty());
    assert!(chunk.len() <= max_chunk_size);
    let account_state_file = adapter
        .write_new_file(stream::once(async move { chunk }))
        .await?;

    let prb = prev_record_bytes.expect("Should have at least one account.");
    let (prev_key, _): (HashValue, AccountStateBlob) = lcs::from_bytes(&prb)?;
    println!("Asking proof for last key: {:x}", prev_key);
    let proof_file = get_proof_and_write(client, adapter, prev_key, version).await?;
    ret.push((account_state_file, proof_file));

    Ok(ret)
}

async fn get_proof_and_write(
    client: &BackupServiceClient,
    adapter: &impl Adapter,
    key: HashValue,
    version: Version,
) -> Result<FileHandle> {
    let mut proof_bytes = Vec::new();
    client
        .get_account_range_proof(key, version)
        .await?
        .read_to_end(&mut proof_bytes)
        .await?;
    let file = adapter
        .write_new_file(stream::once(async move { proof_bytes }))
        .await?;
    Ok(file)
}
