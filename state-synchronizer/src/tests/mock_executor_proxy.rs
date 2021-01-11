// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    executor_proxy::ExecutorProxyTrait, state_synchronizer::SynchronizationState,
    tests::mock_storage::MockStorage,
};
use anyhow::{format_err, Result};
use diem_infallible::RwLock;
use diem_types::{
    contract_event::ContractEvent, ledger_info::LedgerInfoWithSignatures,
    proof::TransactionListProof, transaction::TransactionListWithProof,
};
use std::sync::Arc;

pub(crate) type MockRpcHandler = Box<
    dyn Fn(TransactionListWithProof) -> Result<TransactionListWithProof> + Send + Sync + 'static,
>;

pub(crate) struct MockExecutorProxy {
    handler: MockRpcHandler,
    storage: Arc<RwLock<MockStorage>>,
}

impl MockExecutorProxy {
    pub(crate) fn new(handler: MockRpcHandler, storage: Arc<RwLock<MockStorage>>) -> Self {
        Self { handler, storage }
    }
}

impl ExecutorProxyTrait for MockExecutorProxy {
    fn get_local_storage_state(&self) -> Result<SynchronizationState> {
        Ok(self.storage.read().get_local_storage_state())
    }

    fn execute_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        intermediate_end_of_epoch_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<()> {
        self.storage.write().add_txns_with_li(
            txn_list_with_proof.transactions,
            ledger_info_with_sigs,
            intermediate_end_of_epoch_li,
        );
        Ok(())
    }

    fn get_chunk(
        &self,
        known_version: u64,
        limit: u64,
        target_version: u64,
    ) -> Result<TransactionListWithProof> {
        let start_version = known_version
            .checked_add(1)
            .ok_or_else(|| format_err!("Known version too high"))?;
        let txns = self
            .storage
            .read()
            .get_chunk(start_version, limit, target_version);
        let first_txn_version = txns.first().map(|_| start_version);
        let txns_with_proof = TransactionListWithProof::new(
            txns,
            None,
            first_txn_version,
            TransactionListProof::new_empty(),
        );
        (self.handler)(txns_with_proof)
    }

    fn get_epoch_proof(&self, epoch: u64) -> Result<LedgerInfoWithSignatures> {
        self.storage.read().get_epoch_changes(epoch)
    }

    fn get_epoch_ending_ledger_info(&self, version: u64) -> Result<LedgerInfoWithSignatures> {
        self.storage.read().get_epoch_ending_ledger_info(version)
    }

    fn get_version_timestamp(&self, _version: u64) -> Result<u64> {
        // Only used for logging purposes so no point in mocking
        Ok(0)
    }

    fn publish_on_chain_config_updates(&mut self, _events: Vec<ContractEvent>) -> Result<()> {
        Ok(())
    }
}
