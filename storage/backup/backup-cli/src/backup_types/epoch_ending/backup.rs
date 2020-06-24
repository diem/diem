// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::epoch_ending::manifest::{EpochEndingBackup, EpochEndingChunk},
    storage::{BackupHandleRef, BackupStorage, FileHandle, ShellSafeName},
    utils::{
        backup_service_client::BackupServiceClient, read_record_bytes::ReadRecordBytes,
        GlobalBackupOpt,
    },
};
use anyhow::{ensure, Result};
use libra_types::{ledger_info::LedgerInfoWithSignatures, waypoint::Waypoint};
use once_cell::sync::Lazy;
use std::{convert::TryInto, mem::size_of, str::FromStr, sync::Arc};
use structopt::StructOpt;
use tokio::io::AsyncWriteExt;

#[derive(StructOpt)]
pub struct EpochEndingBackupOpt {
    #[structopt(long = "start-epoch", help = "First epoch to be backed up.")]
    pub start_epoch: u64,

    #[structopt(
        long = "end-epoch",
        help = "Epoch before which epoch ending backup stops. Pass in the current open epoch to get all."
    )]
    pub end_epoch: u64,
}

pub struct EpochEndingBackupController {
    start_epoch: u64,
    end_epoch: u64,
    max_chunk_size: usize,
    client: Arc<BackupServiceClient>,
    storage: Arc<dyn BackupStorage>,
}

impl EpochEndingBackupController {
    pub fn new(
        opt: EpochEndingBackupOpt,
        global_opt: GlobalBackupOpt,
        client: Arc<BackupServiceClient>,
        storage: Arc<dyn BackupStorage>,
    ) -> Self {
        Self {
            start_epoch: opt.start_epoch,
            end_epoch: opt.end_epoch,
            max_chunk_size: global_opt.max_chunk_size,
            client,
            storage,
        }
    }

    pub async fn run(self) -> Result<FileHandle> {
        let backup_handle = self.storage.create_backup(&self.backup_name()).await?;

        let mut chunks = Vec::new();
        let mut waypoints = Vec::new();
        let mut chunk_bytes = Vec::new();

        let mut ledger_infos_file = self
            .client
            .get_epoch_ending_ledger_infos(self.start_epoch, self.end_epoch)
            .await?;
        let mut current_epoch: u64 = self.start_epoch;
        let mut chunk_first_epoch: u64 = self.start_epoch;

        while let Some(record_bytes) = ledger_infos_file.read_record_bytes().await? {
            if chunk_bytes.len() + size_of::<u32>() + record_bytes.len() > self.max_chunk_size {
                ensure!(
                    !chunk_bytes.is_empty(),
                    "max chunk size too small: {}",
                    self.max_chunk_size
                );
                assert!(chunk_bytes.len() <= self.max_chunk_size);
                println!("Reached max_chunk_size.");

                let chunk = self
                    .write_chunk(
                        &backup_handle,
                        &chunk_bytes,
                        chunk_first_epoch,
                        current_epoch - 1,
                    )
                    .await?;
                chunks.push(chunk);
                chunk_bytes = vec![];
                chunk_first_epoch = current_epoch;
            }

            waypoints.push(Self::get_waypoint(&record_bytes, current_epoch)?);
            chunk_bytes.extend(&(record_bytes.len() as u32).to_be_bytes());
            chunk_bytes.extend(&record_bytes);
            current_epoch += 1;
        }

        assert!(!chunk_bytes.is_empty());
        assert!(chunk_bytes.len() <= self.max_chunk_size);
        assert_eq!(current_epoch, self.end_epoch);
        println!("Last chunk.");
        let chunk = self
            .write_chunk(
                &backup_handle,
                &chunk_bytes,
                chunk_first_epoch,
                current_epoch - 1,
            )
            .await?;
        chunks.push(chunk);

        self.write_manifest(&backup_handle, waypoints, chunks).await
    }
}

impl EpochEndingBackupController {
    fn backup_name(&self) -> ShellSafeName {
        format!("epoch_ending_{}-", self.start_epoch)
            .try_into()
            .unwrap()
    }

    fn manifest_name() -> &'static ShellSafeName {
        static NAME: Lazy<ShellSafeName> =
            Lazy::new(|| ShellSafeName::from_str("epoch_ending.manifest").unwrap());
        &NAME
    }

    fn chunk_name(first_epoch: u64) -> ShellSafeName {
        format!("{}-.chunk", first_epoch).try_into().unwrap()
    }

    fn get_waypoint(record: &[u8], epoch: u64) -> Result<Waypoint> {
        let li: LedgerInfoWithSignatures = lcs::from_bytes(record)?;
        ensure!(
            li.ledger_info().epoch() == epoch,
            "Epoch not expected. expected: {}, actual: {}.",
            li.ledger_info().epoch(),
            epoch,
        );
        Waypoint::new_epoch_boundary(li.ledger_info())
    }

    async fn write_chunk(
        &self,
        backup_handle: &BackupHandleRef,
        chunk_bytes: &[u8],
        first_epoch: u64,
        last_epoch: u64,
    ) -> Result<EpochEndingChunk> {
        let (chunk_handle, mut chunk_file) = self
            .storage
            .create_for_write(backup_handle, &Self::chunk_name(first_epoch))
            .await?;
        chunk_file.write_all(&chunk_bytes).await?;
        Ok(EpochEndingChunk {
            first_epoch,
            last_epoch,
            ledger_infos: chunk_handle,
        })
    }

    async fn write_manifest(
        &self,
        backup_handle: &BackupHandleRef,
        waypoints: Vec<Waypoint>,
        chunks: Vec<EpochEndingChunk>,
    ) -> Result<FileHandle> {
        let manifest = EpochEndingBackup {
            first_epoch: self.start_epoch,
            last_epoch: self.end_epoch - 1,
            waypoints,
            chunks,
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
