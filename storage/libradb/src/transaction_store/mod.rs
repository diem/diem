// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This file defines transaction store APIs that are related to committed signed transactions.

use super::schema::signed_transaction::*;
use crate::{change_set::ChangeSet, errors::LibraDbError};
use failure::prelude::*;
use schemadb::DB;
use std::sync::Arc;
use types::transaction::{SignedTransaction, Version};

pub(crate) struct TransactionStore {
    db: Arc<DB>,
}

impl TransactionStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
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
        cs.batch
            .put::<SignedTransactionSchema>(&version, signed_transaction)
    }
}

#[cfg(test)]
mod test;
