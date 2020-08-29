// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::transaction::manifest::{TransactionBackup, TransactionChunk},
    storage::{BackupStorage, FileHandle},
    utils::{read_record_bytes::ReadRecordBytes, storage_ext::BackupStorageExt, GlobalRestoreOpt},
};
use anyhow::{anyhow, ensure, Result};
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
        long = "replay-transactions-from-version",
        help = "Transactions with this version and above will be replayed so state and events are \
        gonna pop up. Requires state at the version right before this to exist, either by \
        recovering a state snapshot, or previous transaction replay."
    )]
    pub replay_from_version: Option<Version>,
}

impl TransactionRestoreOpt {
    pub fn replay_from_version(&self) -> Version {
        self.replay_from_version.unwrap_or(Version::max_value())
    }
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
            replay_from_version: opt.replay_from_version(),
            manifest_handle: opt.manifest_handle,
            target_version: global_opt.target_version(),
            state: State::default(),
        }
    }

    pub async fn run(self) -> Result<()> {
        println!(
            "Transaction restore started. Manifest: {}",
            self.manifest_handle
        );
        self.run_impl()
            .await
            .map_err(|e| anyhow!("Transaction restore failed: {}", e))?;
        println!("Transaction restore succeeded.");
        Ok(())
    }
}

impl TransactionRestoreController {
    async fn run_impl(mut self) -> Result<()> {
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
                let last_to_save = min(last, first_to_replay - 1);
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
                // ditch those beyond the target version
                let num_to_replay = (last - first_to_replay + 1) as usize;
                chunk.txns.truncate(num_to_replay);
                chunk.txn_infos.truncate(num_to_replay);

                // replay in batches
                println!("Replaying transactions {} to {}.", first_to_replay, last);
                #[cfg(not(test))]
                const BATCH_SIZE: usize = 10000;
                #[cfg(test)]
                const BATCH_SIZE: usize = 2;

                let mut current_version = first_to_replay;
                while !chunk.txns.is_empty() {
                    let this_batch_size = min(BATCH_SIZE, chunk.txns.len());
                    self.transaction_replayer(current_version)?.replay_chunk(
                        current_version,
                        chunk.txns.drain(0..this_batch_size).collect::<Vec<_>>(),
                        chunk
                            .txn_infos
                            .drain(0..this_batch_size)
                            .collect::<Vec<_>>(),
                    )?;
                    current_version += this_batch_size as u64;
                }
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

    fn transaction_replayer(&mut self, first_version: Version) -> Result<&mut Executor<LibraVM>> {
        if self.state.transaction_replayer.is_none() {
            let replayer = Executor::new_on_unbootstrapped_db(
                DbReaderWriter::from_arc(Arc::clone(&self.restore_handler.libradb)),
                self.restore_handler.get_tree_state(first_version)?,
            );
            self.state.transaction_replayer = Some(replayer);
        } else {
            assert_eq!(
                self.state
                    .transaction_replayer
                    .as_ref()
                    .unwrap()
                    .expecting_version(),
                first_version
            );
        }
        Ok(self.state.transaction_replayer.as_mut().unwrap())
    }
}
