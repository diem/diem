// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file defines ledger store APIs that are related to the main ledger accumulator, from the
//! root(LedgerInfo) to leaf(TransactionInfo).

use crate::schema::epoch_by_version::EpochByVersionSchema;
use crate::{
    change_set::ChangeSet,
    errors::LibraDbError,
    schema::{
        ledger_info::LedgerInfoSchema, transaction_accumulator::TransactionAccumulatorSchema,
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
    crypto_proxies::LedgerInfoWithSignatures,
    proof::{
        position::Position, AccumulatorConsistencyProof, TransactionAccumulatorProof,
        TransactionAccumulatorRangeProof,
    },
    transaction::{TransactionInfo, Version},
};
use schemadb::{ReadOptions, DB};
use std::{ops::Deref, sync::Arc};

pub(crate) struct LedgerStore {
    db: Arc<DB>,

    /// We almost always need the latest ledger info and signatures to serve read requests, so we
    /// cache it in memory in order to avoid reading DB and deserializing the object frequently. It
    /// should be updated every time new ledger info and signatures are persisted.
    latest_ledger_info: ArcSwap<Option<LedgerInfoWithSignatures>>,
}

// TODO: Either implement an iteration API to allow a very old client to loop through a long history
// or guarantee that there is always a recent enough waypoint and client knows to boot from there.
const MAX_NUM_EPOCH_CHANGE_LEDGER_INFO: usize = 100;

impl LedgerStore {
    pub fn new(db: Arc<DB>) -> Self {
        // Upon restart, read the latest ledger info and signatures and cache them in memory.
        let ledger_info = {
            let mut iter = db
                .iter::<LedgerInfoSchema>(ReadOptions::default())
                .expect("Constructing iterator should work.");
            iter.seek_to_last();
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

    /// Return the ledger infos reflecting epoch bumps with their 2f+1 signatures
    /// in [`start_epoch`, `end_epoch`).
    pub fn get_epoch_change_ledger_infos(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<Vec<LedgerInfoWithSignatures>> {
        let mut iter = self.db.iter::<LedgerInfoSchema>(ReadOptions::default())?;
        iter.seek(&start_epoch)?;

        let mut result = Vec::new();
        for res in iter {
            let (_epoch, ledger_info_with_sigs) = res?;
            if ledger_info_with_sigs.ledger_info().epoch() >= end_epoch {
                break;
            }
            if result.len() >= MAX_NUM_EPOCH_CHANGE_LEDGER_INFO {
                return Err(LibraDbError::TooManyRequested(
                    MAX_NUM_EPOCH_CHANGE_LEDGER_INFO as u64 + 1,
                    MAX_NUM_EPOCH_CHANGE_LEDGER_INFO as u64,
                )
                .into());
            }
            if ledger_info_with_sigs
                .ledger_info()
                .next_validator_set()
                .is_none()
            {
                return Err((LibraDbError::NotFound(format!(
                    "Epoch {} change proof",
                    ledger_info_with_sigs.ledger_info().epoch()
                )))
                .into());
            }
            result.push(ledger_info_with_sigs);
        }

        Ok(result)
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

#[cfg(test)]
mod ledger_info_test;
#[cfg(test)]
mod transaction_info_test;
