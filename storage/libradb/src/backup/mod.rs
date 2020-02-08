// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod test;

use crate::{
    ledger_store::{LedgerStore, TransactionInfoIter},
    transaction_store::{TransactionIter, TransactionStore},
};
use anyhow::Result;
use libra_types::transaction::Version;
use std::sync::Arc;

/// `BackupHandler` provides functionalities for LibraDB data backup.
///
/// TODO: move the account state related code here.
pub struct BackupHandler {
    ledger_store: Arc<LedgerStore>,
    transaction_store: Arc<TransactionStore>,
}

impl BackupHandler {
    pub(crate) fn new(
        ledger_store: Arc<LedgerStore>,
        transaction_store: Arc<TransactionStore>,
    ) -> Self {
        Self {
            ledger_store,
            transaction_store,
        }
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

    /// Gets an iterator that yields a range of transaction infos.
    pub fn get_transaction_info_iter(
        &self,
        start_version: Version,
        num_transaction_infos: u64,
    ) -> Result<TransactionInfoIter> {
        self.ledger_store
            .get_transaction_info_iter(start_version, num_transaction_infos)
    }
}
