// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::transaction::manifest::{TransactionBackup, TransactionChunk},
    storage::{BackupStorage, FileHandle},
    utils::{read_record_bytes::ReadRecordBytes, storage_ext::BackupStorageExt, GlobalRestoreOpt},
};
use anyhow::{ensure, Result};
use executor::Executor;
use executor_types::TransactionReplayer;
use libra_types::{
    ledger_info::LedgerInfoWithSignatures,
    proof::{TransactionAccumulatorRangeProof, TransactionListProof},
    transaction::{Transaction, TransactionInfo, TransactionListWithProof, Version},
};
use libra_vm::LibraVM;
use libradb::backup::restore_handler::RestoreHandler;
use std::{
    cmp::{max, min},
    sync::Arc,
};
use storage_interface::DbReaderWriter;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct TransactionRestoreOpt {
    #[structopt(long = "transaction-manifest")]
    pub manifest_handle: FileHandle,
    #[structopt(
        long = "replay-transaction-from-version",
        default_value = "Version::max_value()"
    )]
    pub replay_from_version: Version,
}

pub struct TransactionRestoreController {
    storage: Arc<dyn BackupStorage>,
    restore_handler: Arc<RestoreHandler>,
    manifest_handle: FileHandle,
    target_version: Version,
    replay_from_version: Version,
    state: State,
}

#[derive(Default)]
struct State {
    frozen_subtree_confirmed: bool,
    transaction_replayer: Option<Executor<LibraVM>>,
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
            replay_from_version: opt.replay_from_version,
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

    fn transaction_replayer(&mut self) -> Result<&mut Executor<LibraVM>> {
        if self.state.transaction_replayer.is_none() {
            let replayer = Executor::new_on_unbootstrapped_db(
                DbReaderWriter::from_arc(Arc::clone(&self.restore_handler.libradb)),
                self.restore_handler
                    .get_tree_state(self.replay_from_version)?,
            );
            self.state.transaction_replayer = Some(replayer);
        }
        Ok(self.state.transaction_replayer.as_mut().unwrap())
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
            if chunk_manifest.first_version > self.target_version {
                break;
            }

            let mut chunk = LoadedChunk::load(chunk_manifest, &self.storage).await?;
            self.maybe_save_frozen_subtrees(&chunk)?;

            let last = min(self.target_version, chunk.manifest.last_version);
            let first_to_replay = max(chunk.manifest.first_version, self.replay_from_version);

            // Transactions to save without replaying:
            if first_to_replay > chunk.manifest.first_version {
                let last_to_save = min(last, first_to_replay);
                let num_txns_to_save = (last_to_save - chunk.manifest.first_version + 1) as usize;
                self.restore_handler.save_transactions(
                    chunk.manifest.first_version,
                    &chunk.txns[..num_txns_to_save],
                    &chunk.txn_infos[..num_txns_to_save],
                )?;
                chunk.txns.drain(0..num_txns_to_save);
                chunk.txn_infos.drain(0..num_txns_to_save);
            }
            // Those to replay:
            if first_to_replay <= last {
                let num_to_replay = (last - first_to_replay + 1) as usize;
                chunk.txns.truncate(num_to_replay);
                chunk.txn_infos.truncate(num_to_replay);
                self.transaction_replayer()?.replay_chunk(
                    first_to_replay,
                    chunk.txns,
                    chunk.txn_infos,
                )?;
            }

            // Last chunk
            if chunk.manifest.last_version == manifest.last_version {
                self.restore_handler
                    .save_ledger_info_if_newer(chunk.ledger_info)?;
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
