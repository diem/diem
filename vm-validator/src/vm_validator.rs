// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use futures::future::{err, ok, Future};
use libra_config::config::NodeConfig;
use libra_types::{
    account_address::AccountAddress, account_config::get_account_resource_or_default,
    transaction::SignedTransaction, vm_error::VMStatus,
};
use scratchpad::SparseMerkleTree;
use std::sync::Arc;
use storage_client::{StorageRead, VerifiedStateView};
use vm_runtime::{LibraVM, VMVerifier};

#[cfg(test)]
#[path = "unit_tests/vm_validator_test.rs"]
mod vm_validator_test;

pub trait TransactionValidation: Send + Sync + Clone {
    type ValidationInstance: VMVerifier;
    /// Validate a txn from client
    fn validate_transaction(
        &self,
        _txn: SignedTransaction,
    ) -> Box<dyn Future<Item = Option<VMStatus>, Error = Error> + Send>;
}

#[derive(Clone)]
pub struct VMValidator {
    storage_read_client: Arc<dyn StorageRead>,
    vm: LibraVM,
}

impl VMValidator {
    pub fn new(config: &NodeConfig, storage_read_client: Arc<dyn StorageRead>) -> Self {
        VMValidator {
            storage_read_client,
            vm: LibraVM::new(&config.vm_config),
        }
    }
}

impl TransactionValidation for VMValidator {
    type ValidationInstance = LibraVM;

    fn validate_transaction(
        &self,
        txn: SignedTransaction,
    ) -> Box<dyn Future<Item = Option<VMStatus>, Error = Error> + Send> {
        match self.storage_read_client.get_latest_state_root() {
            Ok((version, state_root)) => {
                let smt = SparseMerkleTree::new(state_root);
                let state_view = VerifiedStateView::new(
                    Arc::clone(&self.storage_read_client),
                    Some(version),
                    state_root,
                    &smt,
                );
                Box::new(ok(self.vm.validate_transaction(txn, &state_view)))
            }
            Err(e) => Box::new(err(e)),
        }
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
    let account_resource = get_account_resource_or_default(&account_state)?;
    let sequence_number = account_resource.sequence_number();
    let balance = account_resource.balance();
    Ok((sequence_number, balance))
}
