// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod test;

use crate::transaction_store::{TransactionIter, TransactionStore};
use anyhow::Result;
use libra_types::transaction::Version;
use std::sync::Arc;

/// `BackupHandler` provides functionalities for LibraDB data backup.
///
/// TODO: move the account state related code here.
pub struct BackupHandler {
    transaction_store: Arc<TransactionStore>,
}

impl BackupHandler {
    pub(crate) fn new(transaction_store: Arc<TransactionStore>) -> Self {
        Self { transaction_store }
    }

    /// Gets an iterator that yields a range of transactions.
    pub fn get_transaction_iter(
        &self,
        start_version: Version,
        num_transactions: u64,
    ) -> Result<TransactionIter> {
        self.transaction_store
            .get_transaction_iter(start_version, num_transactions)
    }
}
