// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::transaction::manifest::TransactionBackup,
    storage::{BackupStorage, FileHandle},
    utils::{read_record_bytes::ReadRecordBytes, GlobalRestoreOpt},
};
use anyhow::{ensure, Result};
use libra_types::{
    ledger_info::LedgerInfoWithSignatures,
    proof::{TransactionAccumulatorRangeProof, TransactionListProof},
    transaction::{Transaction, TransactionInfo, TransactionListWithProof, Version},
};
use libradb::backup::restore_handler::RestoreHandler;
use std::{cmp::min, sync::Arc};
use structopt::StructOpt;
use tokio::io::AsyncReadExt;

#[derive(StructOpt)]
pub struct TransactionRestoreOpt {
    #[structopt(long = "transaction-manifest")]
    pub manifest_handle: FileHandle,
}

pub struct TransactionRestoreController {
    storage: Arc<dyn BackupStorage>,
    restore_handler: Arc<RestoreHandler>,
    manifest_handle: FileHandle,
    target_version: Version,
}

impl TransactionRestoreController {
    pub fn new(
        opt: TransactionRestoreOpt,
        global_opt: GlobalRestoreOpt,
        storage: Arc<dyn BackupStorage>,
        restore_handler: Arc<RestoreHandler>,
    ) -> Self {
        Self {
            storage,
            restore_handler,
            manifest_handle: opt.manifest_handle,
            target_version: global_opt.target_version,
        }
    }

    pub async fn run(self) -> Result<()> {
        let mut manifest_bytes = Vec::new();
        self.storage
            .open_for_read(&self.manifest_handle)
            .await?
            .read_to_end(&mut manifest_bytes)
            .await?;
        let manifest: TransactionBackup = serde_json::from_slice(&manifest_bytes)?;
        manifest.verify()?;
        if self.target_version < manifest.first_version {
            println!(
                "Manifest {} skipped since its entirety is newer than target version {}.",
                self.manifest_handle, self.target_version,
            );
            return Ok(());
        }

        let mut first_chunk = true;
        for chunk in manifest.chunks {
            // verify
            let (txns, txn_infos) = self.read_chunk(chunk.transactions).await?;
            let (proof, ledger_info) = self.read_proof(chunk.proof).await?;
            ensure!(
                chunk.first_version + (txns.len() as Version) == chunk.last_version + 1,
                "Number of items in chunks doesn't match that in manifest. first_version: {}, last_version: {}, items in chunk: {}",
                chunk.first_version,
                chunk.last_version,
                txns.len(),
            );

            let txn_list_with_proof = TransactionListWithProof::new(
                txns,
                None,
                Some(chunk.first_version),
                TransactionListProof::new(proof, txn_infos),
            );
            txn_list_with_proof.verify(ledger_info.ledger_info(), Some(chunk.first_version))?;

            let last = min(self.target_version, chunk.last_version);
            // write to db
            if last >= chunk.first_version {
                self.restore_handler.save_transactions(
                    &txn_list_with_proof,
                    &ledger_info,
                    first_chunk,
                    (last - chunk.first_version + 1) as usize,
                )?;
                first_chunk = false;
            }

            if last < chunk.last_version {
                break;
            }
        }

        if self.target_version < manifest.last_version {
            println!(
                "Transactions newer than target version {} ignored.",
                self.target_version,
            )
        }

        Ok(())
    }
}

impl TransactionRestoreController {
    async fn read_chunk(
        &self,
        file_handle: FileHandle,
    ) -> Result<(Vec<Transaction>, Vec<TransactionInfo>)> {
        let mut file = self.storage.open_for_read(&file_handle).await?;
        let mut txns = Vec::new();
        let mut txn_infos = Vec::new();

        while let Some(record_bytes) = file.read_record_bytes().await? {
            let (txn, txn_info) = lcs::from_bytes(&record_bytes)?;
            txns.push(txn);
            txn_infos.push(txn_info);
        }

        Ok((txns, txn_infos))
    }

    async fn read_proof(
        &self,
        file_handle: FileHandle,
    ) -> Result<(TransactionAccumulatorRangeProof, LedgerInfoWithSignatures)> {
        let mut file = self.storage.open_for_read(&file_handle).await?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).await?;
        Ok(lcs::from_bytes(&bytes)?)
    }
}
