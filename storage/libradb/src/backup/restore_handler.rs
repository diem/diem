// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    change_set::ChangeSet,
    ledger_store::{Accumulator as TransactionAccumulator, LedgerStore},
    schema::transaction_accumulator::TransactionAccumulatorSchema,
    state_store::StateStore,
    transaction_store::TransactionStore,
};
use anyhow::{anyhow, ensure, Result};
use libra_crypto::HashValue;
use libra_jellyfish_merkle::{restore::JellyfishMerkleRestore, TreeReader, TreeWriter};
use libra_types::{
    ledger_info::LedgerInfoWithSignatures,
    proof::definition::LeafCount,
    transaction::{TransactionListWithProof, Version},
};
use schemadb::DB;
use std::{borrow::Borrow, sync::Arc};

/// Provides functionalities for LibraDB data restore.
#[derive(Clone)]
pub struct RestoreHandler {
    db: Arc<DB>,
    ledger_store: Arc<LedgerStore>,
    transaction_store: Arc<TransactionStore>,
    state_store: Arc<StateStore>,
}

impl RestoreHandler {
    pub(crate) fn new(
        db: Arc<DB>,
        ledger_store: Arc<LedgerStore>,
        transaction_store: Arc<TransactionStore>,
        state_store: Arc<StateStore>,
    ) -> Self {
        Self {
            db,
            ledger_store,
            transaction_store,
            state_store,
        }
    }

    pub fn get_state_restore_receiver(
        &self,
        version: Version,
        expected_root_hash: HashValue,
    ) -> Result<JellyfishMerkleRestore<impl TreeReader + TreeWriter>> {
        JellyfishMerkleRestore::new(&*self.state_store, version, expected_root_hash)
    }

    pub fn save_ledger_infos(&self, ledger_infos: &[LedgerInfoWithSignatures]) -> Result<()> {
        ensure!(!ledger_infos.is_empty(), "No LedgerInfos to save.");

        let mut cs = ChangeSet::new();
        ledger_infos
            .iter()
            .map(|li| self.ledger_store.put_ledger_info(li, &mut cs))
            .collect::<Result<Vec<_>>>()?;
        self.db.write_schemas(cs.batch)?;

        if let Some(li) = self.ledger_store.get_latest_ledger_info_option() {
            if li.ledger_info().epoch() > ledger_infos.last().unwrap().ledger_info().epoch() {
                // No need to update latest ledger info.
                return Ok(());
            }
        }

        self.ledger_store
            .set_latest_ledger_info(ledger_infos.last().unwrap().clone());
        Ok(())
    }

    pub fn save_transactions(
        &self,
        txn_list_with_proof: &TransactionListWithProof,
        ledger_info: &LedgerInfoWithSignatures,
        save_left_siblings: bool,
    ) -> Result<()> {
        // TODO: check signatures
        let first_version = txn_list_with_proof
            .first_transaction_version
            .ok_or_else(|| anyhow!("Transaction list is empty."))?;

        if save_left_siblings {
            let mut cs = ChangeSet::new();
            let (left_sibling_positions, _) = TransactionAccumulator::get_range_proof_positions(
                self.ledger_store.borrow(),
                ledger_info.ledger_info().version() + 1,
                txn_list_with_proof.first_transaction_version,
                txn_list_with_proof.transactions.len() as LeafCount,
            )?;

            ensure!(
                left_sibling_positions.len() == txn_list_with_proof.proof.left_siblings().len(),
                "Number of left siblings not expected. Expected: {}, actual: {}",
                left_sibling_positions.len(),
                txn_list_with_proof.proof.left_siblings().len(),
            );

            left_sibling_positions
                .iter()
                .zip(txn_list_with_proof.proof.left_siblings())
                .map(|(p, h)| {
                    if self.db.get::<TransactionAccumulatorSchema>(&p)?.is_none() {
                        cs.batch.put::<TransactionAccumulatorSchema>(p, h)?;
                    }
                    Ok(())
                })
                .collect::<Result<Vec<_>>>()?;
            self.db.write_schemas(cs.batch)?;
        }

        let mut cs = ChangeSet::new();
        let mut version = first_version;
        for txn in &txn_list_with_proof.transactions {
            self.transaction_store
                .put_transaction(version, txn, &mut cs)?;
            version += 1;
        }
        self.ledger_store.put_transaction_infos(
            first_version,
            txn_list_with_proof.proof.transaction_infos(),
            &mut cs,
        )?;

        self.db.write_schemas(cs.batch)
    }
}
