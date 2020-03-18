// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file defines ledger store APIs that are related to the main ledger accumulator, from the
//! root(LedgerInfo) to leaf(TransactionInfo).

use crate::{
    change_set::ChangeSet,
    errors::LibraDbError,
    schema::{
        epoch_by_version::EpochByVersionSchema, ledger_info::LedgerInfoSchema,
        transaction_accumulator::TransactionAccumulatorSchema,
        transaction_info::TransactionInfoSchema,
    },
};
use accumulator::{HashReader, MerkleAccumulator};
use anyhow::{ensure, format_err, Result};
use arc_swap::ArcSwap;
use itertools::Itertools;
use libra_crypto::{
    hash::{CryptoHash, TransactionAccumulatorHasher},
    HashValue,
};
use libra_types::{
    ledger_info::LedgerInfoWithSignatures,
    proof::{
        position::Position, AccumulatorConsistencyProof, TransactionAccumulatorProof,
        TransactionAccumulatorRangeProof,
    },
    transaction::{TransactionInfo, Version},
    validator_set::ValidatorSet,
};
use schemadb::{ReadOptions, SchemaIterator, DB};
use std::{ops::Deref, sync::Arc};

pub(crate) struct LedgerStore {
    db: Arc<DB>,

    /// We almost always need the latest ledger info and signatures to serve read requests, so we
    /// cache it in memory in order to avoid reading DB and deserializing the object frequently. It
    /// should be updated every time new ledger info and signatures are persisted.
    latest_ledger_info: ArcSwap<Option<LedgerInfoWithSignatures>>,
}

impl LedgerStore {
    pub fn new(db: Arc<DB>) -> Self {
        // Upon restart, read the latest ledger info and signatures and cache them in memory.
        let ledger_info = {
            let mut iter = db
                .iter::<LedgerInfoSchema>(ReadOptions::default())
                .expect("Constructing iterator should work.");
            iter.seek_to_last().expect("Unable to seek to last entry!");
            iter.next()
                .transpose()
                .expect("Reading latest ledger info from DB should work.")
                .map(|kv| kv.1)
        };

        Self {
            db,
            latest_ledger_info: ArcSwap::from(Arc::new(ledger_info)),
        }
    }

    pub fn get_epoch(&self, version: Version) -> Result<u64> {
        let mut iter = self
            .db
            .iter::<EpochByVersionSchema>(ReadOptions::default())?;
        // Search for the end of the previous epoch.
        iter.seek_for_prev(&version)?;
        let (epoch_end_version, epoch) = match iter.next().transpose()? {
            Some(x) => x,
            None => {
                // There should be a genesis LedgerInfo at version 0 (genesis only consists of one
                // transaction), so this normally doesn't happen. However this part of
                // implementation doesn't need to rely on this assumption.
                return Ok(0);
            }
        };
        ensure!(
            epoch_end_version <= version,
            "DB corruption: looking for epoch for version {}, got epoch {} ends at version {}",
            version,
            epoch,
            epoch_end_version
        );
        // If the obtained epoch ended before the given version, return epoch+1, otherwise
        // the given version is exactly the last version of the found epoch.
        Ok(if epoch_end_version < version {
            epoch + 1
        } else {
            epoch
        })
    }

    /// Returns the ledger infos reflecting epoch bumps with their 2f+1 signatures in
    /// [`start_epoch`, `end_epoch`). If there is no more than `limit` results, this function
    /// returns all of them, otherwise the first `limit` results are returned and a flag
    /// (when true) will be used to indicate the fact that there is more.
    pub fn get_first_n_epoch_change_ledger_infos(
        &self,
        start_epoch: u64,
        end_epoch: u64,
        limit: usize,
    ) -> Result<(Vec<LedgerInfoWithSignatures>, bool)> {
        let mut iter = self.db.iter::<LedgerInfoSchema>(ReadOptions::default())?;
        iter.seek(&start_epoch)?;

        let mut results = Vec::new();
        for res in iter {
            let (epoch, ledger_info_with_sigs) = res?;
            debug_assert_eq!(epoch, ledger_info_with_sigs.ledger_info().epoch());

            if epoch >= end_epoch {
                break;
            }
            if results.len() >= limit {
                return Ok((results, true));
            }

            ensure!(
                ledger_info_with_sigs
                    .ledger_info()
                    .next_validator_set()
                    .is_some(),
                "DB corruption: the last ledger info of epoch {} is missing next validator set",
                epoch,
            );
            results.push(ledger_info_with_sigs);
        }

        Ok((results, false))
    }

    pub fn get_latest_ledger_info_option(&self) -> Option<LedgerInfoWithSignatures> {
        let ledger_info_ptr = self.latest_ledger_info.load();
        let ledger_info: &Option<_> = ledger_info_ptr.deref();
        ledger_info.clone()
    }

    pub fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures> {
        self.get_latest_ledger_info_option()
            .ok_or_else(|| LibraDbError::NotFound(String::from("Genesis LedgerInfo")).into())
    }

    pub fn set_latest_ledger_info(&self, ledger_info_with_sigs: LedgerInfoWithSignatures) {
        self.latest_ledger_info
            .store(Arc::new(Some(ledger_info_with_sigs)));
    }

    /// Returns `None` if the DB is empty. Otherwise returns a tuple
    /// `(LedgerInfoWithSignatures, Option<ValidatorSet>)`: either the first element carries a
    /// validator set and the second element is `None`, or the second element carries a validator
    /// set.
    pub fn get_startup_info(
        &self,
    ) -> Result<Option<(LedgerInfoWithSignatures, Option<ValidatorSet>)>> {
        let latest_ledger_info_with_sigs = match self.get_latest_ledger_info_option() {
            Some(x) => x,
            None => return Ok(None),
        };

        if latest_ledger_info_with_sigs
            .ledger_info()
            .next_validator_set()
            .is_some()
        {
            return Ok(Some((latest_ledger_info_with_sigs, None)));
        }

        // If the latest LedgerInfo doesn't carry a validator set, we look for the previous
        // LedgerInfo which should always carry a validator set.
        let latest_epoch = latest_ledger_info_with_sigs.ledger_info().epoch();
        ensure!(
            latest_epoch > 0,
            "Genesis epoch should always carry a validator set.",
        );

        let prev_ledger_info_with_sigs = self
            .db
            .get::<LedgerInfoSchema>(&(latest_epoch - 1))?
            .ok_or_else(|| format_err!("At least one epoch change LedgerInfo must exist."))?;

        let latest_validator_set = prev_ledger_info_with_sigs
            .ledger_info()
            .next_validator_set()
            .ok_or_else(|| format_err!("All previous LedgerInfo should have a validator set."))?;
        Ok(Some((
            latest_ledger_info_with_sigs,
            Some(latest_validator_set.clone()),
        )))
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
        iter.seek_to_last()?;
        iter.next().transpose()
    }

    /// Get latest transaction info together with its version. Note that during node syncing, this
    /// version can be greater than what's in the latest LedgerInfo.
    pub fn get_latest_transaction_info(&self) -> Result<(Version, TransactionInfo)> {
        self.get_latest_transaction_info_option()?
            .ok_or_else(|| LibraDbError::NotFound(String::from("Genesis TransactionInfo.")).into())
    }

    /// Gets an iterator that yields `num_transaction_infos` transaction infos starting from
    /// `start_version`.
    pub fn get_transaction_info_iter(
        &self,
        start_version: Version,
        num_transaction_infos: u64,
    ) -> Result<TransactionInfoIter> {
        let mut iter = self
            .db
            .iter::<TransactionInfoSchema>(ReadOptions::default())?;
        iter.seek(&start_version)?;
        Ok(TransactionInfoIter {
            inner: iter,
            expected_next_version: start_version,
            end_version: start_version
                .checked_add(num_transaction_infos)
                .ok_or_else(|| format_err!("Too many transaction infos requested."))?,
        })
    }

    /// Get transaction info at `version` with proof towards root of ledger at `ledger_version`.
    pub fn get_transaction_info_with_proof(
        &self,
        version: Version,
        ledger_version: Version,
    ) -> Result<(TransactionInfo, TransactionAccumulatorProof)> {
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
    ) -> Result<TransactionAccumulatorProof> {
        Accumulator::get_proof(self, ledger_version + 1 /* num_leaves */, version)
    }

    /// Get proof for `num_txns` consecutive transactions starting from `start_version` towards
    /// root of ledger at `ledger_version`.
    pub fn get_transaction_range_proof(
        &self,
        start_version: Option<Version>,
        num_txns: u64,
        ledger_version: Version,
    ) -> Result<TransactionAccumulatorRangeProof> {
        Accumulator::get_range_proof(
            self,
            ledger_version + 1, /* num_leaves */
            start_version,
            num_txns,
        )
    }

    /// Gets proof that shows the ledger at `ledger_version` is consistent with the ledger at
    /// `client_known_version`.
    pub fn get_consistency_proof(
        &self,
        client_known_version: Version,
        ledger_version: Version,
    ) -> Result<AccumulatorConsistencyProof> {
        Accumulator::get_consistency_proof(self, ledger_version + 1, client_known_version + 1)
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
        let ledger_info = ledger_info_with_sigs.ledger_info();

        if ledger_info.next_validator_set().is_some() {
            // This is the last version of the current epoch, update the epoch by version index.
            cs.batch
                .put::<EpochByVersionSchema>(&ledger_info.version(), &ledger_info.epoch())?;
        }
        cs.batch.put::<LedgerInfoSchema>(
            &ledger_info_with_sigs.ledger_info().epoch(),
            ledger_info_with_sigs,
        )
    }

    /// From left to right, get frozen subtree root hashes of the transaction accumulator.
    pub fn get_ledger_frozen_subtree_hashes(&self, version: Version) -> Result<Vec<HashValue>> {
        Accumulator::get_frozen_subtree_hashes(self, version + 1)
    }
}

type Accumulator = MerkleAccumulator<LedgerStore, TransactionAccumulatorHasher>;

impl HashReader for LedgerStore {
    fn get(&self, position: Position) -> Result<HashValue> {
        self.db
            .get::<TransactionAccumulatorSchema>(&position)?
            .ok_or_else(|| format_err!("{} does not exist.", position))
    }
}

pub struct TransactionInfoIter<'a> {
    inner: SchemaIterator<'a, TransactionInfoSchema>,
    expected_next_version: Version,
    end_version: Version,
}

impl<'a> TransactionInfoIter<'a> {
    fn next_impl(&mut self) -> Result<Option<TransactionInfo>> {
        if self.expected_next_version >= self.end_version {
            return Ok(None);
        }

        let ret = match self.inner.next().transpose()? {
            Some((version, transaction_info)) => {
                ensure!(
                    version == self.expected_next_version,
                    "Transaction info versions are not consecutive.",
                );
                self.expected_next_version += 1;
                Some(transaction_info)
            }
            _ => None,
        };

        Ok(ret)
    }
}

impl<'a> Iterator for TransactionInfoIter<'a> {
    type Item = Result<TransactionInfo>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl().transpose()
    }
}

#[cfg(test)]
mod ledger_info_test;
#[cfg(test)]
mod transaction_info_test;
