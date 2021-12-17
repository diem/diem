// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{ExecutedTrees, StateComputeResult, TransactionData};
use anyhow::{ensure, Result};
use diem_crypto::hash::{CryptoHash, TransactionAccumulatorHasher};
use diem_types::{
    contract_event::ContractEvent,
    epoch_state::EpochState,
    ledger_info::LedgerInfoWithSignatures,
    on_chain_config,
    proof::accumulator::InMemoryAccumulator,
    transaction::{Transaction, TransactionInfo, TransactionStatus, TransactionToCommit},
};
use std::sync::Arc;

#[derive(Default)]
pub struct ExecutedChunk {
    pub status: Vec<TransactionStatus>,
    pub to_commit: Vec<(Transaction, TransactionData)>,
    pub result_view: ExecutedTrees,
    /// If set, this is the new epoch info that should be changed to if this is committed.
    pub next_epoch_state: Option<EpochState>,
    pub ledger_info: Option<LedgerInfoWithSignatures>,
}

impl ExecutedChunk {
    pub fn new_empty(result_view: ExecutedTrees) -> Self {
        Self {
            result_view,
            ..Default::default()
        }
    }

    pub fn reconfig_suffix(&self) -> Self {
        assert!(self.next_epoch_state.is_some());
        Self {
            result_view: self.result_view.clone(),
            next_epoch_state: self.next_epoch_state.clone(),
            ..Default::default()
        }
    }

    pub fn transactions_to_commit(&self) -> Result<Vec<TransactionToCommit>> {
        self.to_commit
            .iter()
            .map(|(txn, txn_data)| {
                Ok(TransactionToCommit::new(
                    txn.clone(),
                    txn_data.account_blobs().clone(),
                    Some(txn_data.jf_node_hashes().clone()),
                    txn_data.write_set().clone(),
                    txn_data.events().to_vec(),
                    txn_data.gas_used(),
                    txn_data.status().as_kept_status()?,
                ))
            })
            .collect()
    }

    pub fn transactions(&self) -> Vec<Transaction> {
        self.to_commit.iter().map(|(txn, _)| txn).cloned().collect()
    }

    pub fn events_to_commit(&self) -> Vec<ContractEvent> {
        self.to_commit
            .iter()
            .map(|(_, txn_data)| txn_data.events())
            .flatten()
            .cloned()
            .collect()
    }

    pub fn has_reconfiguration(&self) -> bool {
        self.next_epoch_state.is_some()
    }

    pub fn ensure_transaction_infos_match(
        &self,
        transaction_infos: &[TransactionInfo],
    ) -> Result<()> {
        ensure!(
            self.to_commit.len() == transaction_infos.len(),
            "Lengths don't match. {} vs {}",
            self.to_commit.len(),
            transaction_infos.len(),
        );
        let txn_info_hashes = self
            .to_commit
            .iter()
            .map(|(_, txn_data)| txn_data.txn_info_hash().unwrap())
            .collect::<Vec<_>>();
        let expected_txn_info_hashes = transaction_infos
            .iter()
            .map(CryptoHash::hash)
            .collect::<Vec<_>>();
        ensure!(
            txn_info_hashes == expected_txn_info_hashes,
            "Transaction infos don't match",
        );
        Ok(())
    }

    pub fn maybe_select_chunk_ending_ledger_info(
        &self,
        verified_target_li: &LedgerInfoWithSignatures,
        epoch_change_li: Option<&LedgerInfoWithSignatures>,
    ) -> Result<Option<LedgerInfoWithSignatures>> {
        let result_accumulator = self.result_view.txn_accumulator();

        if verified_target_li.ledger_info().version() + 1 == result_accumulator.num_leaves() {
            // If the chunk corresponds to the target LI, the target LI can be added to storage.
            ensure!(
                verified_target_li
                    .ledger_info()
                    .transaction_accumulator_hash()
                    == result_accumulator.root_hash(),
                "Root hash in target ledger info does not match local computation."
            );
            Ok(Some(verified_target_li.clone()))
        } else if let Some(epoch_change_li) = epoch_change_li {
            // If the epoch change LI is present, it must match the version of the chunk:

            // Verify that the given ledger info corresponds to the new accumulator.
            ensure!(
                epoch_change_li.ledger_info().transaction_accumulator_hash()
                    == result_accumulator.root_hash(),
                "Root hash of a given epoch LI does not match local computation."
            );
            ensure!(
                epoch_change_li.ledger_info().version() + 1 == result_accumulator.num_leaves(),
                "Version of a given epoch LI does not match local computation."
            );
            ensure!(
                epoch_change_li.ledger_info().ends_epoch(),
                "Epoch change LI does not carry validator set"
            );
            ensure!(
                epoch_change_li.ledger_info().next_epoch_state() == self.next_epoch_state.as_ref(),
                "New validator set of a given epoch LI does not match local computation"
            );
            Ok(Some(epoch_change_li.clone()))
        } else {
            ensure!(
                self.next_epoch_state.is_none(),
                "End of epoch chunk based on local computation but no EoE LedgerInfo provided."
            );
            Ok(None)
        }
    }

    pub fn combine(self, rhs: Self) -> Result<Self> {
        let mut to_commit = self.to_commit;
        to_commit.extend(rhs.to_commit.into_iter());
        let mut status = self.status;
        status.extend(rhs.status.into_iter());

        Ok(Self {
            status,
            to_commit,
            result_view: rhs.result_view,
            next_epoch_state: rhs.next_epoch_state,
            ledger_info: rhs.ledger_info,
        })
    }

    pub fn as_state_compute_result(
        &self,
        parent_accumulator: &Arc<InMemoryAccumulator<TransactionAccumulatorHasher>>,
    ) -> StateComputeResult {
        let new_epoch_event_key = on_chain_config::new_epoch_event_key();
        let txn_accu = self.result_view.txn_accumulator();

        let mut transaction_info_hashes = Vec::new();
        let mut reconfig_events = Vec::new();

        for (_, txn_data) in &self.to_commit {
            transaction_info_hashes.push(txn_data.txn_info_hash().expect("Txn to be kept."));
            reconfig_events.extend(
                txn_data
                    .events()
                    .iter()
                    .filter(|e| *e.key() == new_epoch_event_key)
                    .cloned(),
            )
        }

        StateComputeResult::new(
            txn_accu.root_hash(),
            txn_accu.frozen_subtree_roots().clone(),
            txn_accu.num_leaves(),
            parent_accumulator.frozen_subtree_roots().clone(),
            parent_accumulator.num_leaves(),
            self.next_epoch_state.clone(),
            self.status.clone(),
            transaction_info_hashes,
            reconfig_events,
        )
    }
}
