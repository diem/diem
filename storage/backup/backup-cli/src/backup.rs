// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{adapter::Adapter, FileHandle};
use anyhow::{bail, Result};
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use futures::{stream, stream::TryStreamExt};
use libra_crypto::HashValue;
use libra_types::{account_state_blob::AccountStateBlob, transaction::Version};
use std::convert::TryInto;
use storage_client::StorageReadServiceClient;
use tokio::io::AsyncReadExt;
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

    async fn get(&self, path: &str) -> Result<impl AsyncReadExt> {
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

    async fn get_account_range_proof(
        &self,
        key: HashValue,
        version: Version,
    ) -> Result<impl AsyncReadExt> {
        self.get(&format!("state_range_proof/{}/{:x}", version, key))
            .await
    }

    async fn get_state_snapshot(&self, version: Version) -> Result<impl AsyncReadExt> {
        self.get(&format!("state_snapshot/{}", version)).await
    }
}

#[async_trait]
trait ReadRecordBytes {
    async fn read_full_buf_or_none(&mut self, buf: &mut BytesMut) -> Result<()>;
    async fn read_record_bytes(&mut self) -> Result<Option<Bytes>>;
}

#[async_trait]
impl<R: AsyncReadExt + Send + Unpin> ReadRecordBytes for R {
    async fn read_full_buf_or_none(&mut self, buf: &mut BytesMut) -> Result<()> {
        assert_eq!(buf.len(), 0);
        let n_expected = buf.capacity();

        loop {
            let n_read = self.read_buf(buf).await?;
            let n_read_total = buf.len();
            if n_read_total == n_expected {
                return Ok(());
            }
            if n_read == 0 {
                if n_read_total == 0 {
                    return Ok(());
                } else {
                    bail!(
                        "Hit EOF before filling the whole buffer, read {}, expected {}",
                        n_read_total,
                        n_expected
                    );
                }
            }
        }
    }

    async fn read_record_bytes(&mut self) -> Result<Option<Bytes>> {
        // read record size
        let mut size_buf = BytesMut::with_capacity(4);
        self.read_full_buf_or_none(&mut size_buf).await?;
        if size_buf.is_empty() {
            return Ok(None);
        }

        // read record
        let record_size = u32::from_be_bytes(size_buf.as_ref().try_into()?) as usize;
        let mut record_buf = BytesMut::with_capacity(record_size);
        self.read_full_buf_or_none(&mut record_buf).await?;
        if record_buf.is_empty() {
            bail!("Hit EOF when reading record.")
        }

        Ok(Some(record_buf.freeze()))
    }
}

pub async fn backup_account_state(
    _client: &StorageReadServiceClient,
    backup_service_client: &BackupServiceClient,
    version: Version,
    adapter: &impl Adapter,
    max_chunk_size: usize,
) -> Result<Vec<(FileHandle, FileHandle)>> {
    let mut chunk = vec![];
    let mut ret = vec![];

    let mut state_snapshot_stream = backup_service_client.get_state_snapshot(version).await?;
    let mut prev_record_bytes: Option<Bytes> = None;
    while let Some(record_bytes) = state_snapshot_stream.read_record_bytes().await? {
        if chunk.len() + 4 + record_bytes.len() > max_chunk_size {
            assert!(chunk.len() <= max_chunk_size);
            let account_state_file = adapter
                .write_new_file(stream::once(async move { chunk }))
                .await?;

            prev_record_bytes.expect("max_chunk_size should be larger than account size.");
            let (prev_key, _): (HashValue, AccountStateBlob) = lcs::from_bytes(&record_bytes)?;
            println!(
                "Reached max_chunk_size. Asking proof for key: {:?}",
                prev_key,
            );
            let proof_file =
                get_proof_and_write(backup_service_client, adapter, prev_key, version).await?;
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

    let prev_record_bytes = prev_record_bytes.expect("Should have at least one account.");
    let (prev_key, _): (HashValue, AccountStateBlob) = lcs::from_bytes(&prev_record_bytes)?;
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
    let mut proof_bytes = Vec::new();
    backup_service_client
        .get_account_range_proof(key, version)
        .await?
        .read_to_end(&mut proof_bytes)
        .await?;
    let file = adapter
        .write_new_file(stream::once(async move { proof_bytes }))
        .await?;
    Ok(file)
}
