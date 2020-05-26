// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    manifest::state_snapshot::{StateSnapshotBackup, StateSnapshotChunk},
    storage::{BackupHandleRef, BackupStorage, FileHandle},
    ReadRecordBytes,
};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use futures::stream::TryStreamExt;
use libra_crypto::HashValue;
use libra_types::{
    account_state_blob::AccountStateBlob, ledger_info::LedgerInfoWithSignatures,
    proof::TransactionInfoWithProof, transaction::Version,
};
use std::{mem::size_of, sync::Arc};
use structopt::StructOpt;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio_util::compat::FuturesAsyncReadCompatExt;

#[derive(StructOpt)]
pub struct BackupServiceClientOpt {
    #[structopt(
        long = "backup-service-port",
        about = "Backup service port. The service must listen on localhost."
    )]
    pub port: u16,
}

pub struct BackupServiceClient {
    port: u16,
    client: reqwest::Client,
}

impl BackupServiceClient {
    pub fn new_with_opt(opt: BackupServiceClientOpt) -> Self {
        Self::new(opt.port)
    }

    pub fn new(port: u16) -> Self {
        Self {
            port,
            client: reqwest::Client::builder()
                .no_proxy()
                .build()
                .expect("Http client should build."),
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

    async fn get_state_root_proof(&self, version: Version) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.get(&format!("state_root_proof/{}", version))
            .await?
            .read_to_end(&mut buf)
            .await?;
        Ok(buf)
    }
}

#[derive(StructOpt)]
pub struct GlobalBackupOpt {
    #[structopt(long = "max-chunk-size", about = "Maximum chunk file size in bytes.")]
    pub max_chunk_size: usize,
}

#[derive(StructOpt)]
pub struct StateSnapshotBackupOpt {
    #[structopt(
        long = "state-version",
        about = "Version at which a state snapshot to be taken."
    )]
    pub version: Version,
}

pub struct StateSnapshotBackupController {
    version: Version,
    max_chunk_size: usize,
    client: Arc<BackupServiceClient>,
    storage: Arc<dyn BackupStorage>,
}

impl StateSnapshotBackupController {
    pub fn new(
        opt: StateSnapshotBackupOpt,
        global_opt: GlobalBackupOpt,
        client: Arc<BackupServiceClient>,
        storage: Arc<dyn BackupStorage>,
    ) -> Self {
        Self {
            version: opt.version,
            max_chunk_size: global_opt.max_chunk_size,
            client,
            storage,
        }
    }

    pub async fn run(self) -> Result<FileHandle> {
        let backup_handle = self.storage.create_backup(&self.backup_name()).await?;

        let mut chunks = vec![];

        let mut state_snapshot_file = self.client.get_state_snapshot(self.version).await?;
        let mut prev_record_bytes = state_snapshot_file
            .read_record_bytes()
            .await?
            .ok_or_else(|| anyhow!("State is empty."))?;
        let mut chunk_bytes = (prev_record_bytes.len() as u32).to_be_bytes().to_vec();
        chunk_bytes.extend(&prev_record_bytes);
        let mut chunk_first_key = Self::parse_key(&prev_record_bytes)?;
        let mut current_idx: usize = 0;
        let mut chunk_first_idx: usize = 0;

        while let Some(record_bytes) = state_snapshot_file.read_record_bytes().await? {
            if chunk_bytes.len() + size_of::<u32>() + record_bytes.len() > self.max_chunk_size {
                assert!(chunk_bytes.len() <= self.max_chunk_size);
                println!("Reached max_chunk_size.");

                let chunk = self
                    .write_chunk(
                        &backup_handle,
                        &chunk_bytes,
                        chunk_first_idx,
                        current_idx,
                        chunk_first_key,
                        Self::parse_key(&prev_record_bytes)?,
                    )
                    .await?;
                chunks.push(chunk);
                chunk_bytes = vec![];
                chunk_first_idx = current_idx + 1;
                chunk_first_key = Self::parse_key(&record_bytes)?;
            }

            current_idx += 1;
            chunk_bytes.extend(&(record_bytes.len() as u32).to_be_bytes());
            chunk_bytes.extend(&record_bytes);
            prev_record_bytes = record_bytes;
        }

        assert!(!chunk_bytes.is_empty());
        assert!(chunk_bytes.len() <= self.max_chunk_size);
        println!("Last chunk.");
        let chunk = self
            .write_chunk(
                &backup_handle,
                &chunk_bytes,
                chunk_first_idx,
                current_idx,
                chunk_first_key,
                Self::parse_key(&prev_record_bytes)?,
            )
            .await?;
        chunks.push(chunk);

        self.write_manifest(&backup_handle, chunks).await
    }
}

impl StateSnapshotBackupController {
    fn backup_name(&self) -> String {
        format!("state_ver_{}", self.version)
    }

    fn manifest_name() -> &'static str {
        "state.manifest"
    }

    fn proof_name() -> &'static str {
        "state.proof"
    }

    fn chunk_name(first_idx: usize) -> String {
        format!("{}-.chunk", first_idx)
    }

    fn chunk_proof_name(first_idx: usize, last_idx: usize) -> String {
        format!("{}-{}.proof", first_idx, last_idx)
    }

    fn parse_key(record: &Bytes) -> Result<HashValue> {
        let (key, _): (HashValue, AccountStateBlob) = lcs::from_bytes(record)?;
        Ok(key)
    }

    async fn write_chunk(
        &self,
        backup_handle: &BackupHandleRef,
        chunk_bytes: &[u8],
        first_idx: usize,
        last_idx: usize,
        first_key: HashValue,
        last_key: HashValue,
    ) -> Result<StateSnapshotChunk> {
        println!("Asking proof for key: {:?}", last_key);
        let (chunk_handle, mut chunk_file) = self
            .storage
            .create_for_write(backup_handle, &Self::chunk_name(first_idx))
            .await?;
        chunk_file.write_all(&chunk_bytes).await?;
        let (proof_handle, mut proof_file) = self
            .storage
            .create_for_write(backup_handle, &Self::chunk_proof_name(first_idx, last_idx))
            .await?;
        tokio::io::copy(
            &mut self
                .client
                .get_account_range_proof(last_key, self.version)
                .await?,
            &mut proof_file,
        )
        .await?;

        Ok(StateSnapshotChunk {
            first_idx,
            last_idx,
            first_key,
            last_key,
            blobs: chunk_handle,
            proof: proof_handle,
        })
    }

    async fn write_manifest(
        &self,
        backup_handle: &BackupHandleRef,
        chunks: Vec<StateSnapshotChunk>,
    ) -> Result<FileHandle> {
        let proof_bytes = self.client.get_state_root_proof(self.version).await?;
        let (txn_info, ledger_info): (TransactionInfoWithProof, LedgerInfoWithSignatures) =
            lcs::from_bytes(&proof_bytes)?;
        assert_eq!(ledger_info.ledger_info().version(), self.version);

        let (proof_handle, mut proof_file) = self
            .storage
            .create_for_write(&backup_handle, Self::proof_name())
            .await?;
        proof_file.write_all(&proof_bytes).await?;

        let manifest = StateSnapshotBackup {
            version: self.version,
            root_hash: txn_info.transaction_info().state_root_hash(),
            chunks,
            proof: proof_handle,
        };

        let (manifest_handle, mut manifest_file) = self
            .storage
            .create_for_write(&backup_handle, Self::manifest_name())
            .await?;
        manifest_file
            .write_all(&serde_json::to_vec(&manifest)?)
            .await?;

        Ok(manifest_handle)
    }
}
