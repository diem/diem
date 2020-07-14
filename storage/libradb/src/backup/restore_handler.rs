// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    change_set::ChangeSet, ledger_store::LedgerStore,
    schema::transaction_accumulator::TransactionAccumulatorSchema, state_store::StateStore,
    transaction_store::TransactionStore, LibraDB,
};
use anyhow::{ensure, Result};
use libra_crypto::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, HashValue};
use libra_jellyfish_merkle::{restore::JellyfishMerkleRestore, TreeReader, TreeWriter};
use libra_types::{
    ledger_info::LedgerInfoWithSignatures,
    proof::{definition::LeafCount, position::FrozenSubTreeIterator},
    transaction::{Transaction, TransactionInfo, Version, PRE_GENESIS_VERSION},
};
use schemadb::DB;
use std::sync::Arc;
use storage_interface::TreeState;

/// Provides functionalities for LibraDB data restore.
#[derive(Clone)]
pub struct RestoreHandler {
    db: Arc<DB>,
    pub libradb: Arc<LibraDB>,
    ledger_store: Arc<LedgerStore>,
    transaction_store: Arc<TransactionStore>,
    state_store: Arc<StateStore>,
}

impl RestoreHandler {
    pub(crate) fn new(
        db: Arc<DB>,
        libradb: Arc<LibraDB>,
        ledger_store: Arc<LedgerStore>,
        transaction_store: Arc<TransactionStore>,
        state_store: Arc<StateStore>,
    ) -> Self {
        Self {
            db,
            libradb,
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

    pub fn save_ledger_info_if_newer(&self, ledger_info: LedgerInfoWithSignatures) -> Result<()> {
        if let Some(latest_li) = self.ledger_store.get_latest_ledger_info_option() {
            if latest_li.ledger_info().version() >= ledger_info.ledger_info().version() {
                return Ok(());
            }
        }

        let mut cs = ChangeSet::new();
        self.ledger_store.put_ledger_info(&ledger_info, &mut cs)?;
        self.db.write_schemas(cs.batch)?;
        self.ledger_store.set_latest_ledger_info(ledger_info);
        Ok(())
    }

    pub fn confirm_or_save_frozen_subtrees(
        &self,
        num_leaves: LeafCount,
        frozen_subtrees: &[HashValue],
    ) -> Result<()> {
        let mut cs = ChangeSet::new();
        let positions: Vec<_> = FrozenSubTreeIterator::new(num_leaves).collect();

        ensure!(
            positions.len() == frozen_subtrees.len(),
            "Number of frozen subtree roots not expected. Expected: {}, actual: {}",
            positions.len(),
            frozen_subtrees.len(),
        );

        positions
            .iter()
            .zip(frozen_subtrees.iter().rev())
            .map(|(p, h)| {
                if let Some(_h) = self.db.get::<TransactionAccumulatorSchema>(&p)? {
                    ensure!(
                        h == &_h,
                        "Frozen subtree root does not match that already in DB. Provided: {}, in db: {}.",
                        h,
                        _h,
                    );
                } else {
                    cs.batch.put::<TransactionAccumulatorSchema>(p, h)?;
                }
                Ok(())
            })
            .collect::<Result<Vec<_>>>()?;
        self.db.write_schemas(cs.batch)
    }

    pub fn save_transactions(
        &self,
        first_version: Version,
        txns: &[Transaction],
        txn_infos: &[TransactionInfo],
    ) -> Result<()> {
        let mut cs = ChangeSet::new();
        for (idx, txn) in txns.iter().enumerate() {
            self.transaction_store
                .put_transaction(first_version + idx as Version, txn, &mut cs)?;
        }
        self.ledger_store
            .put_transaction_infos(first_version, txn_infos, &mut cs)?;

        self.db.write_schemas(cs.batch)
    }

    pub fn get_tree_state(&self, num_transactions: LeafCount) -> Result<TreeState> {
        let frozen_subtrees = self
            .ledger_store
            .get_frozen_subtree_hashes(num_transactions)?;
        let state_root_hash = if num_transactions == 0 {
            self.state_store
                .get_root_hash_option(PRE_GENESIS_VERSION)?
                .unwrap_or(*SPARSE_MERKLE_PLACEHOLDER_HASH)
        } else {
            self.state_store.get_root_hash(num_transactions - 1)?
        };

        Ok(TreeState::new(
            num_transactions,
            frozen_subtrees,
            state_root_hash,
        ))
    }
}
