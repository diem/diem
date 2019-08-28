// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file defines transaction store APIs that are related to committed signed transactions.

use super::schema::signed_transaction::*;
use crate::{
    change_set::ChangeSet, errors::LibraDbError,
    schema::transaction_by_account::TransactionByAccountSchema,
};
use failure::prelude::*;
use schemadb::DB;
use std::sync::Arc;
use types::{
    account_address::AccountAddress,
    transaction::{SignedTransaction, Version},
};

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
        self.db
            .get::<SignedTransactionSchema>(&version)?
            .ok_or_else(|| LibraDbError::NotFound(format!("Txn {}", version)).into())
    }

    /// Save signed transaction at `version`
    pub fn put_transaction(
        &self,
        version: Version,
        signed_transaction: &SignedTransaction,
        cs: &mut ChangeSet,
    ) -> Result<()> {
        cs.batch.put::<TransactionByAccountSchema>(
            &(
                signed_transaction.sender(),
                signed_transaction.sequence_number(),
            ),
            &version,
        )?;
        cs.batch
            .put::<SignedTransactionSchema>(&version, signed_transaction)?;

        Ok(())
    }
}

#[cfg(test)]
mod test;
