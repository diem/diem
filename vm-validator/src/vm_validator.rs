// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{convert::TryFrom, sync::Arc};

use anyhow::Result;

use libra_types::{
    account_address::AccountAddress,
    account_config::AccountResource,
    on_chain_config::OnChainConfigPayload,
    transaction::{SignedTransaction, VMValidatorResult},
};
use libra_vm::LibraVM;
use scratchpad::SparseMerkleTree;
use storage_interface::{state_view::VerifiedStateView, DbReader};

#[cfg(test)]
#[path = "unit_tests/vm_validator_test.rs"]
mod vm_validator_test;

pub trait TransactionValidation: Send + Sync + Clone {
    type ValidationInstance: libra_vm::VMValidator;

    /// Validate a txn from client
    fn validate_transaction(&self, _txn: SignedTransaction) -> Result<VMValidatorResult>;

    /// Restart the transaction validation instance
    fn restart(&mut self, config: OnChainConfigPayload) -> Result<()>;
}

#[derive(Clone)]
pub struct VMValidator {
    db_reader: Arc<dyn DbReader>,
    vm: LibraVM,
}

impl VMValidator {
    pub fn new(db_reader: Arc<dyn DbReader>) -> Self {
        let mut vm = LibraVM::new();
        let (version, state_root) = db_reader.get_latest_state_root().expect("Should not fail.");
        let smt = SparseMerkleTree::new(state_root);
        let state_view =
            VerifiedStateView::new(Arc::clone(&db_reader), Some(version), state_root, &smt);

        vm.load_configs(&state_view);
        VMValidator { db_reader, vm }
    }
}

impl TransactionValidation for VMValidator {
    type ValidationInstance = LibraVM;

    fn validate_transaction(&self, txn: SignedTransaction) -> Result<VMValidatorResult> {
        use libra_vm::VMValidator;

        let (version, state_root) = self.db_reader.get_latest_state_root()?;
        let db_reader = Arc::clone(&self.db_reader);
        let vm = self.vm.clone();

        let smt = SparseMerkleTree::new(state_root);
        let state_view = VerifiedStateView::new(db_reader, Some(version), state_root, &smt);

        Ok(vm.validate_transaction(txn, &state_view))
    }

    fn restart(&mut self, config: OnChainConfigPayload) -> Result<()> {
        let vm_config = config.vm_config()?;
        let version = config.libra_version()?;

        self.vm = LibraVM::init_with_config(version, vm_config);
        Ok(())
    }
}

/// returns account's sequence number from storage
pub fn get_account_sequence_number(storage: &dyn DbReader, address: AccountAddress) -> Result<u64> {
    match storage.get_latest_account_state(address)? {
        Some(blob) => Ok(AccountResource::try_from(&blob)?.sequence_number()),
        None => Ok(0),
    }
}
