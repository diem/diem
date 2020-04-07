// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures::executor::block_on;
use libra_types::{
    account_address::AccountAddress,
    account_config::AccountResource,
    transaction::{SignedTransaction, VMValidatorResult},
};
use libra_vm::{LibraVM, VMVerifier};
use scratchpad::SparseMerkleTree;
use std::{convert::TryFrom, sync::Arc};
use storage_client::{StorageRead, SyncStorageClient, VerifiedStateView};
use tokio::runtime::Handle;

#[cfg(test)]
#[path = "unit_tests/vm_validator_test.rs"]
mod vm_validator_test;

#[async_trait::async_trait]
pub trait TransactionValidation: Send + Sync + Clone {
    type ValidationInstance: VMVerifier;
    /// Validate a txn from client
    async fn validate_transaction(&self, _txn: SignedTransaction) -> Result<VMValidatorResult>;
}

#[derive(Clone)]
pub struct VMValidator {
    storage_read_client: Arc<dyn StorageRead>,
    rt_handle: Handle,
    vm: LibraVM,
}

impl VMValidator {
    pub fn new(storage_read_client: Arc<dyn StorageRead>, rt_handle: Handle) -> Self {
        let mut vm = LibraVM::new();
        let client = storage_read_client.clone();
        let (version, state_root) =
            block_on(rt_handle.spawn(async move { client.get_latest_state_root().await }))
                .expect("Block error")
                .expect("Failed to get the latest state");
        let smt = SparseMerkleTree::new(state_root);
        let state_view = VerifiedStateView::new(
            Arc::new(SyncStorageClient::new(
                storage_read_client.clone(),
                rt_handle.clone(),
            )),
            Some(version),
            state_root,
            &smt,
        );

        vm.load_configs(&state_view);
        VMValidator {
            storage_read_client,
            rt_handle,
            vm,
        }
    }
}

#[async_trait::async_trait]
impl TransactionValidation for VMValidator {
    type ValidationInstance = LibraVM;

    async fn validate_transaction(&self, txn: SignedTransaction) -> Result<VMValidatorResult> {
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
            let state_view = VerifiedStateView::new(
                Arc::new(SyncStorageClient::new(client, rt_handle)),
                Some(version),
                state_root,
                &smt,
            );

            vm.validate_transaction(txn, &state_view)
        })
        .await
        .map_err(Into::into)
    }
}

/// returns account's sequence number from storage
pub async fn get_account_sequence_number(
    storage_read_client: Arc<dyn StorageRead>,
    address: AccountAddress,
) -> Result<u64> {
    match storage_read_client
        .get_latest_account_state(address)
        .await?
    {
        Some(blob) => Ok(AccountResource::try_from(&blob)?.sequence_number()),
        None => Ok(0),
    }
}
