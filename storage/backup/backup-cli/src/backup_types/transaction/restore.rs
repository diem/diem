// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backup_types::{
        epoch_ending::restore::EpochHistory,
        transaction::manifest::{TransactionBackup, TransactionChunk},
    },
    metrics::{
        restore::{TRANSACTION_REPLAY_VERSION, TRANSACTION_SAVE_VERSION},
        verify::VERIFY_TRANSACTION_VERSION,
    },
    storage::{BackupStorage, FileHandle},
    utils::{
        read_record_bytes::ReadRecordBytes, storage_ext::BackupStorageExt, stream::StreamX,
        GlobalRestoreOptions, RestoreRunMode,
    },
};
use anyhow::{anyhow, bail, ensure, Result};
use diem_logger::prelude::*;
use diem_types::{
    contract_event::ContractEvent,
    ledger_info::LedgerInfoWithSignatures,
    proof::{TransactionAccumulatorRangeProof, TransactionListProof},
    transaction::{Transaction, TransactionInfo, TransactionListWithProof, Version},
};
use diem_vm::DiemVM;
use executor::Executor;
use executor_types::TransactionReplayer;
use futures::StreamExt;
use std::{
    cmp::{max, min},
    sync::Arc,
};
use storage_interface::DbReaderWriter;
use structopt::StructOpt;
use tokio::io::BufReader;

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
    run_mode: Arc<RestoreRunMode>,
    manifest_handle: FileHandle,
    target_version: Version,
    replay_from_version: Version,
    epoch_history: Option<Arc<EpochHistory>>,
    state: State,
}

#[allow(dead_code)]
struct LoadedChunk {
    pub manifest: TransactionChunk,
    pub txns: Vec<Transaction>,
    pub txn_infos: Vec<TransactionInfo>,
    pub event_vecs: Vec<Vec<ContractEvent>>,
    pub range_proof: TransactionAccumulatorRangeProof,
    pub ledger_info: LedgerInfoWithSignatures,
}

impl LoadedChunk {
    async fn load(
        manifest: TransactionChunk,
        storage: &Arc<dyn BackupStorage>,
        epoch_history: Option<&Arc<EpochHistory>>,
    ) -> Result<Self> {
        let mut file = BufReader::new(storage.open_for_read(&manifest.transactions).await?);
        let mut txns = Vec::new();
        let mut txn_infos = Vec::new();
        let mut event_vecs = Vec::new();

        while let Some(record_bytes) = file.read_record_bytes().await? {
            let (txn, txn_info, events) = bcs::from_bytes(&record_bytes)?;
            txns.push(txn);
            txn_infos.push(txn_info);
            event_vecs.push(events);
        }

        ensure!(
            manifest.first_version + (txns.len() as Version) == manifest.last_version + 1,
            "Number of items in chunks doesn't match that in manifest. first_version: {}, last_version: {}, items in chunk: {}",
            manifest.first_version,
            manifest.last_version,
            txns.len(),
        );

        let (range_proof, ledger_info) = storage
            .load_bcs_file::<(TransactionAccumulatorRangeProof, LedgerInfoWithSignatures)>(
                &manifest.proof,
            )
            .await?;
        if let Some(epoch_history) = epoch_history {
            epoch_history.verify_ledger_info(&ledger_info)?;
        }

        // make a `TransactionListWithProof` to reuse its verification code.
        let txn_list_with_proof = TransactionListWithProof::new(
            txns,
            Some(event_vecs),
            Some(manifest.first_version),
            TransactionListProof::new(range_proof, txn_infos),
        );
        txn_list_with_proof.verify(ledger_info.ledger_info(), Some(manifest.first_version))?;
        // and disassemble it to get things back.
        let txns = txn_list_with_proof.transactions;
        let (range_proof, txn_infos) = txn_list_with_proof.proof.unpack();
        let event_vecs = txn_list_with_proof.events.expect("unknown to be Some.");

        Ok(Self {
            manifest,
            txns,
            txn_infos,
            event_vecs,
            range_proof,
            ledger_info,
        })
    }
}

impl TransactionRestoreController {
    pub fn new(
        opt: TransactionRestoreOpt,
        global_opt: GlobalRestoreOptions,
        storage: Arc<dyn BackupStorage>,
        epoch_history: Option<Arc<EpochHistory>>,
    ) -> Self {
        Self {
            storage,
            run_mode: global_opt.run_mode,
            replay_from_version: opt.replay_from_version(),
            manifest_handle: opt.manifest_handle,
            target_version: global_opt.target_version,
            epoch_history,
            state: State::default(),
        }
    }

    pub async fn preheat(mut self) -> PreheatedTransactionRestore {
        PreheatedTransactionRestore {
            preheat_result: self.preheat_impl().await,
            controller: self,
        }
    }

    pub async fn run(self) -> Result<()> {
        self.preheat().await.run().await
    }
}

impl TransactionRestoreController {
    fn name(&self) -> String {
        format!("transaction {}", self.run_mode.name())
    }

    async fn preheat_impl(&mut self) -> Result<TransactionRestorePreheatData> {
        let manifest: TransactionBackup =
            self.storage.load_json_file(&self.manifest_handle).await?;
        manifest.verify()?;

        let mut loaded_chunks = Vec::new();
        for chunk_manifest in &manifest.chunks {
            if chunk_manifest.first_version > self.target_version {
                break;
            }

            loaded_chunks.push(
                LoadedChunk::load(
                    chunk_manifest.clone(),
                    &self.storage,
                    self.epoch_history.as_ref(),
                )
                .await?,
            )
        }

        Ok(TransactionRestorePreheatData {
            manifest,
            loaded_chunks,
        })
    }

    fn maybe_restore_transactions(
        &mut self,
        mut chunk: LoadedChunk,
        last: u64,
        first_to_replay: u64,
    ) -> Result<()> {
        if let RestoreRunMode::Restore { restore_handler } = self.run_mode.as_ref() {
            // Transactions to save without replaying:
            if first_to_replay > chunk.manifest.first_version {
                let last_to_save = min(last, first_to_replay - 1);
                let num_txns_to_save = (last_to_save - chunk.manifest.first_version + 1) as usize;
                restore_handler.save_transactions(
                    chunk.manifest.first_version,
                    &chunk.txns[..num_txns_to_save],
                    &chunk.txn_infos[..num_txns_to_save],
                    &chunk.event_vecs[..num_txns_to_save],
                )?;
                chunk.txns.drain(0..num_txns_to_save);
                chunk.txn_infos.drain(0..num_txns_to_save);
                chunk.event_vecs.drain(0..num_txns_to_save);
                TRANSACTION_SAVE_VERSION.set(last_to_save as i64);
            }
            // Those to replay:
            if first_to_replay <= last {
                // ditch those beyond the target version
                let num_to_replay = (last - first_to_replay + 1) as usize;
                chunk.txns.truncate(num_to_replay);
                chunk.txn_infos.truncate(num_to_replay);

                // replay in batches
                info!("Replaying transactions {} to {}.", first_to_replay, last);
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
                    TRANSACTION_REPLAY_VERSION.set(current_version as i64 - 1);
                }
            }
        } else {
            VERIFY_TRANSACTION_VERSION.set(last as i64);
        }

        Ok(())
    }

    fn maybe_save_frozen_subtrees(&mut self, chunk: &LoadedChunk) -> Result<()> {
        if let RestoreRunMode::Restore { restore_handler } = self.run_mode.as_ref() {
            if !self.state.frozen_subtree_confirmed {
                restore_handler.confirm_or_save_frozen_subtrees(
                    chunk.manifest.first_version,
                    chunk.range_proof.left_siblings(),
                )?;
                self.state.frozen_subtree_confirmed = true;
            }
        }

        Ok(())
    }

    fn transaction_replayer(&mut self, first_version: Version) -> Result<&mut Executor<DiemVM>> {
        if self.state.transaction_replayer.is_none() {
            if let RestoreRunMode::Restore { restore_handler } = self.run_mode.as_ref() {
                let replayer = Executor::new_on_unbootstrapped_db(
                    DbReaderWriter::from_arc(Arc::clone(&restore_handler.diemdb)),
                    restore_handler.get_tree_state(first_version)?,
                );
                self.state.transaction_replayer = Some(replayer);
            } else {
                bail!("Trying to construct TransactionReplayer under Verify mode.");
            }
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

#[derive(Default)]
struct State {
    frozen_subtree_confirmed: bool,
    transaction_replayer: Option<Executor<DiemVM>>,
}

struct TransactionRestorePreheatData {
    manifest: TransactionBackup,
    loaded_chunks: Vec<LoadedChunk>,
}

pub struct PreheatedTransactionRestore {
    controller: TransactionRestoreController,
    preheat_result: Result<TransactionRestorePreheatData>,
}

impl PreheatedTransactionRestore {
    pub async fn run(self) -> Result<()> {
        let name = self.controller.name();
        info!(
            "{} started. Manifest: {}",
            name, self.controller.manifest_handle
        );
        self.run_impl()
            .await
            .map_err(|e| anyhow!("{} failed: {}", name, e))?;
        info!("{} succeeded.", name);
        Ok(())
    }

    pub fn get_last_version(&self) -> Result<Version> {
        Ok(min(
            self.preheat_result
                .as_ref()
                .map_err(|e| anyhow!("Preheat failed: {}", e))?
                .manifest
                .last_version,
            self.controller.target_version,
        ))
    }
}

impl PreheatedTransactionRestore {
    async fn run_impl(mut self) -> Result<()> {
        let preheat_data = self
            .preheat_result
            .map_err(|e| anyhow!("Preheat failed: {}", e))?;

        for chunk in preheat_data.loaded_chunks {
            self.controller.maybe_save_frozen_subtrees(&chunk)?;

            let last = min(self.controller.target_version, chunk.manifest.last_version);
            let first_to_replay = max(
                chunk.manifest.first_version,
                self.controller.replay_from_version,
            );

            self.controller
                .maybe_restore_transactions(chunk, last, first_to_replay)?;
        }

        if self.controller.target_version < preheat_data.manifest.last_version {
            warn!(
                "Transactions newer than target version {} ignored.",
                self.controller.target_version,
            )
        }

        Ok(())
    }
}

/// Takes a series of transaction backup manifests, preheat in parallel, then execute in order.
pub struct TransactionRestoreBatchController {
    global_opt: GlobalRestoreOptions,
    storage: Arc<dyn BackupStorage>,
    manifest_handles: Vec<FileHandle>,
    replay_from_version: Option<Version>,
    epoch_history: Option<Arc<EpochHistory>>,
}

impl TransactionRestoreBatchController {
    pub fn new(
        global_opt: GlobalRestoreOptions,
        storage: Arc<dyn BackupStorage>,
        manifest_handles: Vec<FileHandle>,
        replay_from_version: Option<Version>,
        epoch_history: Option<Arc<EpochHistory>>,
    ) -> Self {
        Self {
            global_opt,
            storage,
            manifest_handles,
            replay_from_version,
            epoch_history,
        }
    }

    pub async fn run(self) -> Result<()> {
        let timer = std::time::Instant::now();

        let futs_iter = self.manifest_handles.clone().into_iter().map(|hdl| {
            let _global_opt = self.global_opt.clone();
            let _epoch_history = self.epoch_history.clone();
            let _storage = self.storage.clone();
            let _replay_from_version = self.replay_from_version;
            async move {
                // Use `spawn()` so it's (most likely) off the main thread.
                // There's a lot of deserialization / verification happening that's CPU
                // intensive, hence the `spawn()`
                tokio::spawn(
                    TransactionRestoreController::new(
                        TransactionRestoreOpt {
                            manifest_handle: hdl,
                            replay_from_version: _replay_from_version,
                        },
                        _global_opt,
                        _storage,
                        _epoch_history,
                    )
                    .preheat(),
                )
                .await
                .expect("Failed to spawn task.")
            }
        });

        let mut futs_stream = futures::stream::iter(futs_iter).buffered_x(
            // more buffer here because the preheat of txns is heavy and variates more in execution
            // time
            self.global_opt.concurrent_downloads * 3,
            self.global_opt.concurrent_downloads, /* concurrency */
        );
        while let Some(preheated_txn_restore) = futs_stream.next().await {
            let v = preheated_txn_restore.get_last_version();
            preheated_txn_restore.run().await?;
            debug!(
                "Accumulative TPS: {:.0}",
                (v? + 1) as f64 / timer.elapsed().as_secs_f64()
            );
        }

        Ok(())
    }
}
