// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file defines ledger store APIs that are related to the main ledger accumulator, from the
//! root(LedgerInfo) to leaf(TransactionInfo).

use crate::{
    change_set::ChangeSet,
    errors::LibraDbError,
    schema::{
        ledger_info::LedgerInfoSchema, transaction_accumulator::TransactionAccumulatorSchema,
        transaction_info::TransactionInfoSchema,
    },
};
use accumulator::{HashReader, MerkleAccumulator};
use crypto::{
    hash::{CryptoHash, TransactionAccumulatorHasher},
    HashValue,
};
use failure::prelude::*;
use itertools::Itertools;
use schemadb::{ReadOptions, DB};
use std::sync::Arc;
use types::{
    ledger_info::LedgerInfoWithSignatures,
    proof::{
        position::{FrozenSubTreeIterator, Position},
        AccumulatorProof,
    },
    transaction::{TransactionInfo, Version},
};

pub(crate) struct LedgerStore {
    db: Arc<DB>,
}

impl LedgerStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    /// Return the ledger infos with their least 2f+1 signatures starting from `start_version` to
    /// the most recent one.
    /// Note: ledger infos and signatures are only available at the last version of each earlier
    /// epoch and at the latest version of current epoch.
    #[cfg(test)]
    fn get_ledger_infos(&self, start_version: Version) -> Result<Vec<LedgerInfoWithSignatures>> {
        let mut iter = self.db.iter::<LedgerInfoSchema>(ReadOptions::default())?;
        iter.seek(&start_version)?;
        Ok(iter.map(|kv| Ok(kv?.1)).collect::<Result<Vec<_>>>()?)
    }

    pub fn get_latest_ledger_info_option(&self) -> Result<Option<LedgerInfoWithSignatures>> {
        let mut iter = self.db.iter::<LedgerInfoSchema>(ReadOptions::default())?;
        iter.seek_to_last();
        Ok(iter.next().transpose()?.map(|kv| kv.1))
    }

    pub fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures> {
        self.get_latest_ledger_info_option()?
            .ok_or_else(|| LibraDbError::NotFound(String::from("Genesis LedgerInfo")).into())
    }

    /// Get transaction info given `version`
    pub fn get_transaction_info(&self, version: Version) -> Result<TransactionInfo> {
        self.db
            .get::<TransactionInfoSchema>(&version)?
            .ok_or_else(|| format_err!("No TransactionInfo at version {}", version))
    }

    pub fn get_latest_transaction_info_option(&self) -> Result<Option<(Version, TransactionInfo)>> {
        let mut iter = self
            .db
            .iter::<TransactionInfoSchema>(ReadOptions::default())?;
        iter.seek_to_last();
        iter.next().transpose()
    }

    /// Get latest transaction info together with its version. Note that during node syncing, this
    /// version can be greater than what's in the latest LedgerInfo.
    pub fn get_latest_transaction_info(&self) -> Result<(Version, TransactionInfo)> {
        self.get_latest_transaction_info_option()?
            .ok_or_else(|| LibraDbError::NotFound(String::from("Genesis TransactionInfo.")).into())
    }

    /// Get transaction info at `version` with proof towards root of ledger at `ledger_version`.
    pub fn get_transaction_info_with_proof(
        &self,
        version: Version,
        ledger_version: Version,
    ) -> Result<(TransactionInfo, AccumulatorProof)> {
        Ok((
            self.get_transaction_info(version)?,
            self.get_transaction_proof(version, ledger_version)?,
        ))
    }

    /// Get proof for transaction at `version` towards root of ledger at `ledger_version`.
    pub fn get_transaction_proof(
        &self,
        version: Version,
        ledger_version: Version,
    ) -> Result<AccumulatorProof> {
        Accumulator::get_proof(self, ledger_version + 1 /* num_leaves */, version)
    }

    /// Write `txn_infos` to `batch`. Assigned `first_version` to the the version number of the
    /// first transaction, and so on.
    pub fn put_transaction_infos(
        &self,
        first_version: u64,
        txn_infos: &[TransactionInfo],
        cs: &mut ChangeSet,
    ) -> Result<HashValue> {
        // write txn_info
        (first_version..first_version + txn_infos.len() as u64)
            .zip_eq(txn_infos.iter())
            .map(|(version, txn_info)| cs.batch.put::<TransactionInfoSchema>(&version, txn_info))
            .collect::<Result<()>>()?;

        // write hash of txn_info into the accumulator
        let txn_hashes: Vec<HashValue> = txn_infos.iter().map(TransactionInfo::hash).collect();
        let (root_hash, writes) = Accumulator::append(
            self,
            first_version, /* num_existing_leaves */
            &txn_hashes,
        )?;
        writes
            .iter()
            .map(|(pos, hash)| cs.batch.put::<TransactionAccumulatorSchema>(pos, hash))
            .collect::<Result<()>>()?;
        Ok(root_hash)
    }

    /// Write `ledger_info` to `cs`.
    pub fn put_ledger_info(
        &self,
        ledger_info_with_sigs: &LedgerInfoWithSignatures,
        cs: &mut ChangeSet,
    ) -> Result<()> {
        cs.batch.put::<LedgerInfoSchema>(
            &ledger_info_with_sigs.ledger_info().version(),
            ledger_info_with_sigs,
        )
    }

    /// From left to right, get frozen subtree root hashes of the transaction accumulator.
    pub fn get_ledger_frozen_subtree_hashes(&self, version: Version) -> Result<Vec<HashValue>> {
        FrozenSubTreeIterator::new(version + 1)
            .map(|pos| {
                self.db
                    .get::<TransactionAccumulatorSchema>(&pos)?
                    .ok_or_else(|| {
                        LibraDbError::NotFound(format!(
                            "Txn Accumulator node at pos {}",
                            pos.to_inorder_index()
                        ))
                        .into()
                    })
            })
            .collect::<Result<Vec<_>>>()
    }
}

type Accumulator = MerkleAccumulator<LedgerStore, TransactionAccumulatorHasher>;

impl HashReader for LedgerStore {
    fn get(&self, position: Position) -> Result<HashValue> {
        self.db
            .get::<TransactionAccumulatorSchema>(&position)?
            .ok_or_else(|| format_err!("Does not exist."))
    }
}

#[cfg(test)]
mod ledger_info_test;
#[cfg(test)]
mod transaction_info_test;
