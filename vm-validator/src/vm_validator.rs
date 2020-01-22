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

// #[cfg(test)]
// #[path = "unit_tests/vm_validator_test.rs"]
// mod vm_validator_test;

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
        let (version, state_root) = self.storage_read_client.get_latest_state_root().await?;
        let client = self.storage_read_client.clone();
        let rt_handle = self.rt_handle.clone();
        let vm = self.vm.clone();
        // We have to be careful here. The storage read client only exposes async functions but the
        // whole VM is synchronous and async/await isn't currently using in the VM. Due to this
        // there is a trick in the StateView impl which spawns a task on a runtime to actually do
        // the async read while using "block_on" to synchronously wait for the read to complete.
        // This is where things get tricky. If that task is spawned onto the same thread in the
        // pool as the task that is using "block_on" then there is a chance that the read will
        // never complete since it will be resource starved resulting in the "block_on" task
        // hanging forever.
        //
        // In order to work around this issue we can use "spawn_blocking" to move the task that is
        // using "block_on" to its own thread, where it won't interfere with have a chance to
        // starve other async tasks.
        tokio::task::spawn_blocking(move || {
            let smt = SparseMerkleTree::new(state_root);
            let state_view =
                VerifiedStateView::new(client, rt_handle, Some(version), state_root, &smt);

            vm.validate_transaction(txn, &state_view)
        })
        .await
        .map_err(Into::into)
    }
}

/// read account state
/// returns account's current sequence number and balance
pub async fn get_account_state(
    storage_read_client: Arc<dyn StorageRead>,
    address: AccountAddress,
) -> Result<(u64, u64)> {
    let account_state = storage_read_client
        .get_latest_account_state(address)
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
