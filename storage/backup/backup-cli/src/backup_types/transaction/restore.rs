// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::transaction::manifest::{TransactionBackup, TransactionChunk},
    storage::{BackupStorage, FileHandle},
    utils::{read_record_bytes::ReadRecordBytes, storage_ext::BackupStorageExt, GlobalRestoreOpt},
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
    state: State,
}

#[derive(Default)]
struct State {
    frozen_subtree_confirmed: bool,
}

struct LoadedChunk {
    pub manifest: TransactionChunk,
    pub txns: Vec<Transaction>,
    pub txn_infos: Vec<TransactionInfo>,
    pub range_proof: TransactionAccumulatorRangeProof,
    pub ledger_info: LedgerInfoWithSignatures,
}

impl LoadedChunk {
    async fn load(manifest: TransactionChunk, storage: &Arc<dyn BackupStorage>) -> Result<Self> {
        let mut file = storage.open_for_read(&manifest.transactions).await?;
        let mut txns = Vec::new();
        let mut txn_infos = Vec::new();

        while let Some(record_bytes) = file.read_record_bytes().await? {
            let (txn, txn_info) = lcs::from_bytes(&record_bytes)?;
            txns.push(txn);
            txn_infos.push(txn_info);
        }

        ensure!(
            manifest.first_version + (txns.len() as Version) == manifest.last_version + 1,
            "Number of items in chunks doesn't match that in manifest. first_version: {}, last_version: {}, items in chunk: {}",
            manifest.first_version,
            manifest.last_version,
            txns.len(),
        );

        let (range_proof, ledger_info) = storage
            .load_lcs_file::<(TransactionAccumulatorRangeProof, LedgerInfoWithSignatures)>(
                &manifest.proof,
            )
            .await?;
        // TODO: verify signatures

        // make a `TransactionListWithProof` to reuse its verification code.
        let txn_list_with_proof = TransactionListWithProof::new(
            txns,
            None, /* events */
            Some(manifest.first_version),
            TransactionListProof::new(range_proof, txn_infos),
        );
        txn_list_with_proof.verify(ledger_info.ledger_info(), Some(manifest.first_version))?;
        // and disassemble it to get things back.
        let txns = txn_list_with_proof.transactions;
        let (range_proof, txn_infos) = txn_list_with_proof.proof.unpack();

        Ok(Self {
            manifest,
            txns,
            txn_infos,
            range_proof,
            ledger_info,
        })
    }
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
            state: State::default(),
        }
    }

    fn maybe_save_frozen_subtrees(&mut self, chunk: &LoadedChunk) -> Result<()> {
        if !self.state.frozen_subtree_confirmed {
            self.restore_handler.confirm_or_save_frozen_subtrees(
                chunk.manifest.first_version,
                chunk.range_proof.left_siblings(),
            )?;
            self.state.frozen_subtree_confirmed = true;
        }
        Ok(())
    }

    pub async fn run(mut self) -> Result<()> {
        let manifest: TransactionBackup =
            self.storage.load_json_file(&self.manifest_handle).await?;
        manifest.verify()?;
        if self.target_version < manifest.first_version {
            println!(
                "Manifest {} skipped since its entirety is newer than target version {}.",
                self.manifest_handle, self.target_version,
            );
            return Ok(());
        }

        for chunk_manifest in manifest.chunks {
            let chunk = LoadedChunk::load(chunk_manifest, &self.storage).await?;
            self.maybe_save_frozen_subtrees(&chunk)?;

            let last = min(self.target_version, chunk.manifest.last_version);
            let num_txns_to_save = (last - chunk.manifest.first_version + 1) as usize;
            // write to db
            if last >= chunk.manifest.first_version {
                self.restore_handler.save_transactions(
                    chunk.manifest.first_version,
                    &chunk.txns[..num_txns_to_save],
                    &chunk.txn_infos[..num_txns_to_save],
                )?;
            }

            if last < chunk.manifest.last_version {
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
