// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_config::config::NodeConfig;
use libra_types::{
    account_address::AccountAddress, account_config::AccountResource,
    transaction::SignedTransaction, vm_error::VMStatus,
};
use scratchpad::SparseMerkleTree;
use std::convert::TryFrom;
use std::sync::Arc;
use storage_client::{StorageRead, VerifiedStateView};
use tokio::runtime::Handle;
use vm_runtime::{LibraVM, VMVerifier};

#[cfg(test)]
#[path = "unit_tests/vm_validator_test.rs"]
mod vm_validator_test;

#[async_trait::async_trait]
pub trait TransactionValidation: Send + Sync + Clone {
    type ValidationInstance: VMVerifier;
    /// Validate a txn from client
    async fn validate_transaction(&self, _txn: SignedTransaction) -> Result<Option<VMStatus>>;
}

#[derive(Clone)]
pub struct VMValidator {
    storage_read_client: Arc<dyn StorageRead>,
    rt_handle: Handle,
    vm: LibraVM,
}

impl VMValidator {
    pub fn new(
        config: &NodeConfig,
        storage_read_client: Arc<dyn StorageRead>,
        rt_handle: Handle,
    ) -> Self {
        VMValidator {
            storage_read_client,
            rt_handle,
            vm: LibraVM::new(&config.vm_config),
        }
    }
}

#[async_trait::async_trait]
impl TransactionValidation for VMValidator {
    type ValidationInstance = LibraVM;

    async fn validate_transaction(&self, txn: SignedTransaction) -> Result<Option<VMStatus>> {
        let (version, state_root) = self
            .storage_read_client
            .get_latest_state_root_async()
            .await?;
        let smt = SparseMerkleTree::new(state_root);
        let state_view = VerifiedStateView::new(
            Arc::clone(&self.storage_read_client),
            self.rt_handle.clone(),
            Some(version),
            state_root,
            &smt,
        );
        Ok(self.vm.validate_transaction(txn, &state_view))
    }
}

/// read account state
/// returns account's current sequence number and balance
pub async fn get_account_state(
    storage_read_client: Arc<dyn StorageRead>,
    address: AccountAddress,
) -> Result<(u64, u64)> {
    let account_state = storage_read_client
        .get_latest_account_state_async(address)
        .await?;
    Ok(if let Some(blob) = account_state {
        let account_resource = AccountResource::try_from(&blob)?;
        let sequence_number = account_resource.sequence_number();
        let balance = account_resource.balance();
        (sequence_number, balance)
    } else {
        (0, 0)
    })
}
