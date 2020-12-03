// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::transaction::manifest::{TransactionBackup, TransactionChunk},
    metadata::Metadata,
    storage::{BackupHandleRef, BackupStorage, FileHandle, ShellSafeName},
    utils::{
        backup_service_client::BackupServiceClient, read_record_bytes::ReadRecordBytes,
        should_cut_chunk, storage_ext::BackupStorageExt, GlobalBackupOpt,
    },
};
use anyhow::{anyhow, Result};
use diem_logger::prelude::*;
use diem_types::transaction::Version;
use once_cell::sync::Lazy;
use std::{convert::TryInto, str::FromStr, sync::Arc};
use structopt::StructOpt;
use tokio::io::AsyncWriteExt;

#[derive(StructOpt)]
pub struct TransactionBackupOpt {
    #[structopt(long = "start-version", help = "First transaction to backup.")]
    pub start_version: u64,

    #[structopt(long = "num_transactions", help = "Number of transactions to backup")]
    pub num_transactions: usize,
}

pub struct TransactionBackupController {
    start_version: u64,
    num_transactions: usize,
    max_chunk_size: usize,
    client: Arc<BackupServiceClient>,
    storage: Arc<dyn BackupStorage>,
}

impl TransactionBackupController {
    pub fn new(
        opt: TransactionBackupOpt,
        global_opt: GlobalBackupOpt,
        client: Arc<BackupServiceClient>,
        storage: Arc<dyn BackupStorage>,
    ) -> Self {
        Self {
            start_version: opt.start_version,
            num_transactions: opt.num_transactions,
            max_chunk_size: global_opt.max_chunk_size,
            client,
            storage,
        }
    }

    pub async fn run(self) -> Result<FileHandle> {
        info!(
            "Transaction backup started, starting from version {}, for {} transactions in total.",
            self.start_version, self.num_transactions,
        );
        let ret = self
            .run_impl()
            .await
            .map_err(|e| anyhow!("Transaction backup failed: {}", e))?;
        info!("Transaction backup succeeded. Manifest: {}", ret);
        Ok(ret)
    }
}

impl TransactionBackupController {
    async fn run_impl(self) -> Result<FileHandle> {
        let backup_handle = self
            .storage
            .create_backup_with_random_suffix(&self.backup_name())
            .await?;

        let mut chunks = Vec::new();
        let mut chunk_bytes = Vec::new();

        let mut transactions_file = self
            .client
            .get_transactions(self.start_version, self.num_transactions)
            .await?;
        let mut current_ver: u64 = self.start_version;
        let mut chunk_first_ver: u64 = self.start_version;

        while let Some(record_bytes) = transactions_file.read_record_bytes().await? {
            if should_cut_chunk(&chunk_bytes, &record_bytes, self.max_chunk_size) {
                let chunk = self
                    .write_chunk(
                        &backup_handle,
                        &chunk_bytes,
                        chunk_first_ver,
                        current_ver - 1,
                    )
                    .await?;
                chunks.push(chunk);
                chunk_bytes = vec![];
                chunk_first_ver = current_ver;
            }

            chunk_bytes.extend(&(record_bytes.len() as u32).to_be_bytes());
            chunk_bytes.extend(&record_bytes);
            current_ver += 1;
        }

        assert!(!chunk_bytes.is_empty());
        assert_eq!(
            current_ver,
            self.start_version + self.num_transactions as u64
        );
        let chunk = self
            .write_chunk(
                &backup_handle,
                &chunk_bytes,
                chunk_first_ver,
                current_ver - 1,
            )
            .await?;
        chunks.push(chunk);

        self.write_manifest(&backup_handle, self.start_version, current_ver - 1, chunks)
            .await
    }

    fn backup_name(&self) -> String {
        format!("transaction_{}-", self.start_version)
    }

    fn manifest_name() -> &'static ShellSafeName {
        static NAME: Lazy<ShellSafeName> =
            Lazy::new(|| ShellSafeName::from_str("transaction.manifest").unwrap());
        &NAME
    }

    fn chunk_name(first_ver: Version) -> ShellSafeName {
        format!("{}-.chunk", first_ver).try_into().unwrap()
    }

    fn chunk_proof_name(first_ver: u64, last_ver: Version) -> ShellSafeName {
        format!("{}-{}.proof", first_ver, last_ver)
            .try_into()
            .unwrap()
    }

    async fn write_chunk(
        &self,
        backup_handle: &BackupHandleRef,
        chunk_bytes: &[u8],
        first_version: u64,
        last_version: u64,
    ) -> Result<TransactionChunk> {
        let (proof_handle, mut proof_file) = self
            .storage
            .create_for_write(
                backup_handle,
                &Self::chunk_proof_name(first_version, last_version),
            )
            .await?;
        tokio::io::copy(
            &mut self
                .client
                .get_transaction_range_proof(first_version, last_version)
                .await?,
            &mut proof_file,
        )
        .await?;
        proof_file.shutdown().await?;

        let (chunk_handle, mut chunk_file) = self
            .storage
            .create_for_write(backup_handle, &Self::chunk_name(first_version))
            .await?;
        chunk_file.write_all(&chunk_bytes).await?;
        chunk_file.shutdown().await?;

        Ok(TransactionChunk {
            first_version,
            last_version,
            transactions: chunk_handle,
            proof: proof_handle,
        })
    }

    async fn write_manifest(
        &self,
        backup_handle: &BackupHandleRef,
        first_version: Version,
        last_version: Version,
        chunks: Vec<TransactionChunk>,
    ) -> Result<FileHandle> {
        let manifest = TransactionBackup {
            first_version,
            last_version,
            chunks,
        };
        let (manifest_handle, mut manifest_file) = self
            .storage
            .create_for_write(&backup_handle, Self::manifest_name())
            .await?;
        manifest_file
            .write_all(&serde_json::to_vec(&manifest)?)
            .await?;
        manifest_file.shutdown().await?;

        let metadata =
            Metadata::new_transaction_backup(first_version, last_version, manifest_handle.clone());
        self.storage
            .save_metadata_line(&metadata.name(), &metadata.to_text_line()?)
            .await?;

        Ok(manifest_handle)
    }
}
