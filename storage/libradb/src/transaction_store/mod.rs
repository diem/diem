// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file defines transaction store APIs that are related to committed signed transactions.

use crate::schema::transaction::TransactionSchema;
use crate::{
    change_set::ChangeSet, errors::LibraDbError,
    schema::transaction_by_account::TransactionByAccountSchema,
};
use failure::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    transaction::{SignedTransaction, Transaction, Version},
};
use schemadb::DB;
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
    pub fn get_transaction(&self, version: Version) -> Result<SignedTransaction> {
        let txn = self
            .db
            .get::<TransactionSchema>(&version)?
            .ok_or_else(|| LibraDbError::NotFound(format!("Txn {}", version)))?;

        match txn {
            Transaction::UserTransaction(user_txn) => Ok(user_txn),
            // TODO: support other variants after API change
            _ => unreachable!("Currently only supports user transactions."),
        }
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

#[cfg(test)]
mod test;
