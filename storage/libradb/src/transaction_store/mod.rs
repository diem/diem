// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file defines transaction store APIs that are related to committed signed transactions.

use crate::{
    change_set::ChangeSet,
    errors::LibraDbError,
    schema::{transaction::TransactionSchema, transaction_by_account::TransactionByAccountSchema},
};
use anyhow::{ensure, format_err, Result};
use libra_types::{
    account_address::AccountAddress,
    block_metadata::BlockMetadata,
    transaction::{Transaction, Version},
};
use schemadb::{SchemaIterator, DB};
use std::sync::Arc;

pub(crate) struct TransactionStore {
    db: Arc<DB>,
}

impl TransactionStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    /// Gets the version of a transaction by the sender `address` and `sequence_number`.
    pub fn lookup_transaction_by_account(
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

    /// Get signed transaction given `version`
    pub fn get_transaction(&self, version: Version) -> Result<Transaction> {
        self.db
            .get::<TransactionSchema>(&version)?
            .ok_or_else(|| LibraDbError::NotFound(format!("Txn {}", version)).into())
    }

    /// Gets an iterator that yields `num_transactions` transactions starting from `start_version`.
    pub fn get_transaction_iter(
        &self,
        start_version: Version,
        num_transactions: u64,
    ) -> Result<TransactionIter> {
        let mut iter = self.db.iter::<TransactionSchema>(Default::default())?;
        iter.seek(&start_version)?;
        Ok(TransactionIter {
            inner: iter,
            expected_next_version: start_version,
            end_version: start_version
                .checked_add(num_transactions)
                .ok_or_else(|| format_err!("Too many transactions requested."))?,
        })
    }

    /// Returns the block metadata carried on the block metadata transaction at or preceding
    /// `version`, together with the version of the block metadata transaction.
    /// Returns None if there's no such transaction at or preceding `version` (it's likely the genesis
    /// version 0).
    pub fn get_block_metadata(&self, version: Version) -> Result<Option<(Version, BlockMetadata)>> {
        // Maximum TPS from benchmark is around 1000.
        const MAX_VERSIONS_TO_SEARCH: usize = 1000 * 3;

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

        Err(LibraDbError::NotFound(format!("BlockMetadata preceding version {}", version)).into())
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
        cs.batch.put::<TransactionSchema>(&version, &transaction)?;

        Ok(())
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

#[cfg(test)]
mod test;
