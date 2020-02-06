// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file defines transaction store APIs that are related to committed signed transactions.

use crate::schema::transaction::TransactionSchema;
use crate::{
    change_set::ChangeSet, errors::LibraDbError,
    schema::transaction_by_account::TransactionByAccountSchema,
};
use anyhow::{format_err, Result};
use libra_types::{
    account_address::AccountAddress,
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
            end_version: start_version
                .checked_add(num_transactions)
                .ok_or_else(|| format_err!("Too many transactions requested."))?,
        })
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
    end_version: Version,
}

impl<'a> TransactionIter<'a> {
    fn next_impl(&mut self) -> Result<Option<Transaction>> {
        match self.inner.next().transpose()? {
            Some((version, transaction)) if version < self.end_version => Ok(Some(transaction)),
            _ => Ok(None),
        }
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
