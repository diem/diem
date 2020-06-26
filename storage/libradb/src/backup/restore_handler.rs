// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ledger_store::LedgerStore, state_store::StateStore, transaction_store::TransactionStore,
};
use anyhow::Result;
use jellyfish_merkle::{restore::JellyfishMerkleRestore, TreeReader, TreeWriter};
use libra_crypto::HashValue;
use libra_types::transaction::Version;
use std::sync::Arc;

/// Provides functionalities for LibraDB data restore.
#[derive(Clone)]
pub struct RestoreHandler {
    ledger_store: Arc<LedgerStore>,
    transaction_store: Arc<TransactionStore>,
    state_store: Arc<StateStore>,
}

impl RestoreHandler {
    pub(crate) fn new(
        ledger_store: Arc<LedgerStore>,
        transaction_store: Arc<TransactionStore>,
        state_store: Arc<StateStore>,
    ) -> Self {
        Self {
            ledger_store,
            transaction_store,
            state_store,
        }
    }

    pub fn get_state_restore_receiver(
        &self,
        version: Version,
        expected_root_hash: HashValue,
    ) -> Result<JellyfishMerkleRestore<impl TreeReader + TreeWriter>> {
        JellyfishMerkleRestore::new(&*self.state_store, version, expected_root_hash)
    }
}
