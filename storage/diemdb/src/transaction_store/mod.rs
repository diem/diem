// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file defines transaction store APIs that are related to committed signed transactions.

use crate::{
    change_set::ChangeSet,
    errors::DiemDbError,
    schema::{
        transaction::TransactionSchema, transaction_by_account::TransactionByAccountSchema,
        transaction_by_hash::TransactionByHashSchema, write_set::WriteSetSchema,
    },
};
use anyhow::{anyhow, ensure, format_err, Result};
use diem_crypto::{hash::CryptoHash, HashValue};
use diem_types::{
    account_address::AccountAddress,
    block_metadata::BlockMetadata,
    transaction::{Transaction, Version},
    write_set::WriteSet,
};
use schemadb::{ReadOptions, SchemaIterator, DB};
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct TransactionStore {
    db: Arc<DB>,
}

impl TransactionStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    /// Gets the version of a transaction by the sender `address` and `sequence_number`.
    pub fn get_account_transaction_version(
        &self,
        address: AccountAddress,
        sequence_number: u64,
        ledger_version: Version,
    ) -> Result<Option<Version>> {
        if let Some(version) = self
            .db
            .get::<TransactionByAccountSchema>(&(address, sequence_number))?
        {
            if version <= ledger_version {
                return Ok(Some(version));
            }
        }

        Ok(None)
    }

    /// Gets the version of a transaction by its hash.
    pub fn get_transaction_version_by_hash(
        &self,
        hash: &HashValue,
        ledger_version: Version,
    ) -> Result<Option<Version>> {
        Ok(match self.db.get::<TransactionByHashSchema>(hash)? {
            Some(version) if version <= ledger_version => Some(version),
            _ => None,
        })
    }

    /// Gets an iterator that yields `(sequence_number, version)` for each
    /// transaction sent by an account, starting at `start_seq_num`, and returning
    /// at most `num_versions` results with `version <= ledger_version`.
    ///
    /// Guarantees that the returned sequence numbers are sequential, i.e.,
    /// `seq_num_{i} + 1 = seq_num_{i+1}`.
    pub fn get_account_transaction_version_iter(
        &self,
        address: AccountAddress,
        start_seq_num: u64,
        num_versions: u64,
        ledger_version: Version,
    ) -> Result<AccountTransactionVersionIter> {
        let mut iter = self
            .db
            .iter::<TransactionByAccountSchema>(ReadOptions::default())?;
        iter.seek(&(address, start_seq_num))?;
        Ok(AccountTransactionVersionIter {
            inner: iter,
            address,
            expected_next_seq_num: start_seq_num,
            end_seq_num: start_seq_num
                .checked_add(num_versions)
                .ok_or_else(|| format_err!("too many transactions requested"))?,
            prev_version: None,
            ledger_version,
        })
    }

    /// Get signed transaction given `version`
    pub fn get_transaction(&self, version: Version) -> Result<Transaction> {
        self.db
            .get::<TransactionSchema>(&version)?
            .ok_or_else(|| DiemDbError::NotFound(format!("Txn {}", version)).into())
    }

    /// Gets an iterator that yields `num_transactions` transactions starting from `start_version`.
    pub fn get_transaction_iter(
        &self,
        start_version: Version,
        num_transactions: usize,
    ) -> Result<TransactionIter> {
        let mut iter = self.db.iter::<TransactionSchema>(ReadOptions::default())?;
        iter.seek(&start_version)?;
        Ok(TransactionIter {
            inner: iter,
            expected_next_version: start_version,
            end_version: start_version
                .checked_add(num_transactions as u64)
                .ok_or_else(|| format_err!("too many transactions requested"))?,
        })
    }

    /// Get the first version that txn starts existent.
    pub fn get_first_txn_version(&self) -> Result<Option<Version>> {
        let mut iter = self.db.iter::<TransactionSchema>(Default::default())?;
        iter.seek_to_first();
        iter.next().map(|res| res.map(|(v, _)| v)).transpose()
    }

    /// Returns the block metadata carried on the block metadata transaction at or preceding
    /// `version`, together with the version of the block metadata transaction.
    /// Returns None if there's no such transaction at or preceding `version` (it's likely the genesis
    /// version 0).
    pub fn get_block_metadata(&self, version: Version) -> Result<Option<(Version, BlockMetadata)>> {
        // Must be larger than a block size, otherwise a NotFound error will be raised wrongly.
        const MAX_VERSIONS_TO_SEARCH: usize = 1000 * 100;

        // Linear search via `DB::rev_iter()` here, NOT expecting performance hit, due to the fact
        // that the iterator caches data block and that there are limited number of transactions in
        // each block.
        let mut iter = self.db.rev_iter::<TransactionSchema>(Default::default())?;
        iter.seek(&version)?;
        for res in iter.take(MAX_VERSIONS_TO_SEARCH) {
            let (v, txn) = res?;
            if let Transaction::BlockMetadata(block_meta) = txn {
                return Ok(Some((v, block_meta)));
            } else if v == 0 {
                return Ok(None);
            }
        }

        Err(DiemDbError::NotFound(format!("BlockMetadata preceding version {}", version)).into())
    }

    /// Save signed transaction at `version`
    pub fn put_transaction(
        &self,
        version: Version,
        transaction: &Transaction,
        cs: &mut ChangeSet,
    ) -> Result<()> {
        if let Transaction::UserTransaction(txn) = transaction {
            cs.batch.put::<TransactionByAccountSchema>(
                &(txn.sender(), txn.sequence_number()),
                &version,
            )?;
        }
        cs.batch
            .put::<TransactionByHashSchema>(&transaction.hash(), &version)?;
        cs.batch.put::<TransactionSchema>(&version, transaction)?;

        Ok(())
    }

    /// Get executed transaction vm output given `version`
    pub fn get_write_set(&self, version: Version) -> Result<WriteSet> {
        self.db
            .get::<WriteSetSchema>(&version)?
            .ok_or_else(|| DiemDbError::NotFound(format!("WriteSet at version {}", version)).into())
    }

    /// Get write sets in `[begin_version, end_version)` half-open range.
    ///
    /// N.b. an empty `Vec` is returned when `begin_version == end_version`
    pub fn get_write_sets(
        &self,
        begin_version: Version,
        end_version: Version,
    ) -> Result<Vec<WriteSet>> {
        if begin_version == end_version {
            return Ok(Vec::new());
        }
        ensure!(
            begin_version < end_version,
            "begin_version {} > end_version {}",
            begin_version,
            end_version
        );

        let mut iter = self.db.iter::<WriteSetSchema>(Default::default())?;
        iter.seek(&begin_version)?;

        let mut ret = Vec::with_capacity(end_version.saturating_sub(begin_version) as usize);
        for current_version in begin_version..end_version {
            let (version, write_set) = iter
                .next()
                .transpose()?
                .ok_or_else(|| anyhow!("Writeset missing for version {}", current_version))?;
            ensure!(
                version == current_version,
                "Write set missing for version {}, got version {}",
                current_version,
                version,
            );
            ret.push(write_set);
        }

        Ok(ret)
    }

    /// Get the first version that write set starts existent.
    pub fn get_first_write_set_version(&self) -> Result<Option<Version>> {
        let mut iter = self.db.iter::<WriteSetSchema>(Default::default())?;
        iter.seek_to_first();
        iter.next().map(|res| res.map(|(v, _)| v)).transpose()
    }

    /// Save executed transaction vm output given `version`
    pub fn put_write_set(
        &self,
        version: Version,
        write_set: &WriteSet,
        cs: &mut ChangeSet,
    ) -> Result<()> {
        cs.batch.put::<WriteSetSchema>(&version, write_set)
    }
}

pub struct TransactionIter<'a> {
    inner: SchemaIterator<'a, TransactionSchema>,
    expected_next_version: Version,
    end_version: Version,
}

impl<'a> TransactionIter<'a> {
    fn next_impl(&mut self) -> Result<Option<Transaction>> {
        if self.expected_next_version >= self.end_version {
            return Ok(None);
        }

        let ret = match self.inner.next().transpose()? {
            Some((version, transaction)) => {
                ensure!(
                    version == self.expected_next_version,
                    "Transaction versions are not consecutive.",
                );
                self.expected_next_version += 1;
                Some(transaction)
            }
            None => None,
        };

        Ok(ret)
    }
}

impl<'a> Iterator for TransactionIter<'a> {
    type Item = Result<Transaction>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl().transpose()
    }
}

// TODO(philiphayes): this will need to change to support CRSNs
// (Conflict-Resistant Sequence Numbers)[https://github.com/diem/dip/blob/main/dips/dip-168.md].
//
// It depends on the implementation details, but we'll probably index by _requested_
// transaction sequence number rather than committed account sequence number.
// This would mean the property: `seq_num_{i+1} == seq_num_{i} + 1` would no longer
// be guaranteed and the check should be removed.
//
// This index would also no longer iterate over an account's transactions in
// committed order, meaning the outer method would need to overread by
// `CRSN_WINDOW_SIZE`, sort by version, and take only `limit` entries to get
// at most `limit` transactions in committed order. Alternatively, add another
// index for scanning an accounts transactions in committed order, e.g.,
// `(AccountAddress, Version) -> SeqNum`.

pub struct AccountTransactionVersionIter<'a> {
    inner: SchemaIterator<'a, TransactionByAccountSchema>,
    address: AccountAddress,
    expected_next_seq_num: u64,
    end_seq_num: u64,
    prev_version: Option<Version>,
    ledger_version: Version,
}

impl<'a> AccountTransactionVersionIter<'a> {
    fn next_impl(&mut self) -> Result<Option<(u64, Version)>> {
        if self.expected_next_seq_num >= self.end_seq_num {
            return Ok(None);
        }

        Ok(match self.inner.next().transpose()? {
            Some(((address, seq_num), version)) => {
                // No more transactions sent by this account.
                if address != self.address {
                    return Ok(None);
                }

                // Ensure seq_num_{i+1} == seq_num_{i} + 1
                ensure!(
                    seq_num == self.expected_next_seq_num,
                    "DB corruption: account transactions sequence numbers are not contiguous: \
                     actual: {}, expected: {}",
                    seq_num,
                    self.expected_next_seq_num,
                );

                // Ensure version_{i+1} > version_{i}
                if let Some(prev_version) = self.prev_version {
                    ensure!(
                        prev_version < version,
                        "DB corruption: account transaction versions are not strictly increasing: \
                         previous version: {}, current version: {}",
                        prev_version,
                        version,
                    );
                }

                // No more transactions (in this view of the ledger).
                if version > self.ledger_version {
                    return Ok(None);
                }

                self.expected_next_seq_num += 1;
                self.prev_version = Some(version);
                Some((seq_num, version))
            }
            None => None,
        })
    }
}

impl<'a> Iterator for AccountTransactionVersionIter<'a> {
    type Item = Result<(u64, Version)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_impl().transpose()
    }
}

#[cfg(test)]
mod test;
