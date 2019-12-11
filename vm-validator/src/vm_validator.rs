// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Error, Result};
use futures::future::{err, ok, Future};
use libra_config::config::NodeConfig;
use libra_types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    account_config::get_account_resource_or_default,
    get_with_proof::{RequestItem, ResponseItem},
    transaction::SignedTransaction,
    vm_error::VMStatus,
};
use scratchpad::SparseMerkleTree;
use std::sync::Arc;
use storage_client::{StorageRead, VerifiedStateView};
use vm_runtime::{LibraVM, VMVerifier};

#[cfg(test)]
#[path = "unit_tests/vm_validator_test.rs"]
mod vm_validator_test;

pub trait TransactionValidation: Send + Sync {
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
        // TODO: For transaction validation, there are two options to go:
        // 1. Trust storage: there is no need to get root hash from storage here. We will
        // create another struct similar to `VerifiedStateView` that implements `StateView`
        // but does not do verification.
        // 2. Don't trust storage. This requires more work:
        // 1) AC must have validator set information
        // 2) Get state_root from transaction info which can be verified with signatures of
        // validator set.
        // 3) Create VerifiedStateView with verified state
        // root.

        // Just ask something from storage. It doesn't matter what it is -- we just need the
        // transaction info object in account state proof which contains the state root hash.
        let address = AccountAddress::new([0xff; ADDRESS_LENGTH]);
        let item = RequestItem::GetAccountState { address };

        match self.storage_read_client.retrieve_items(vec![item]) {
            Ok((mut items, ledger_info_with_sigs)) => {
                if items.len() != 1 {
                    return Box::new(err(format_err!(
                        "Unexpected number of items ({}).",
                        items.len()
                    )));
                }

                match items.remove(0) {
                    ResponseItem::GetAccountState {
                        account_state_with_proof,
                    } => {
                        let transaction_info = account_state_with_proof.proof.transaction_info();
                        let state_root = transaction_info.state_root_hash();
                        let smt = SparseMerkleTree::new(state_root);
                        let state_view = VerifiedStateView::new(
                            Arc::clone(&self.storage_read_client),
                            Some(ledger_info_with_sigs.ledger_info().version()),
                            state_root,
                            &smt,
                        );
                        Box::new(ok(self.vm.validate_transaction(txn, &state_view)))
                    }
                    _ => panic!("Unexpected item in response."),
                }
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
    let req_item = RequestItem::GetAccountState { address };
    let (response_items, _) = storage_read_client
        .retrieve_items_async(vec![req_item])
        .await?;
    let account_state = match &response_items[0] {
        ResponseItem::GetAccountState {
            account_state_with_proof,
        } => &account_state_with_proof.blob,
        _ => bail!("Not account state response."),
    };
    let account_resource = get_account_resource_or_default(account_state)?;
    let sequence_number = account_resource.sequence_number();
    let balance = account_resource.balance();
    Ok((sequence_number, balance))
}
