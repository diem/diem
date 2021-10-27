// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_state_view::StateViewId;
use diem_types::{
    account_address::AccountAddress,
    account_config::{AccountResource, AccountSequenceInfo},
    account_state::AccountState,
    on_chain_config::{DiemVersion, OnChainConfigPayload, VMConfig, VMPublishingOption},
    protocol_spec::DpnProto,
    transaction::{SignedTransaction, VMValidatorResult},
};
use diem_vm::DiemVM;
use fail::fail_point;
use scratchpad::SparseMerkleTree;
use std::{convert::TryFrom, sync::Arc};
use storage_interface::{state_view::VerifiedStateView, DbReader};

#[cfg(test)]
#[path = "unit_tests/vm_validator_test.rs"]
mod vm_validator_test;

pub trait TransactionValidation: Send + Sync + Clone {
    type ValidationInstance: diem_vm::VMValidator;

    /// Validate a txn from client
    fn validate_transaction(&self, _txn: SignedTransaction) -> Result<VMValidatorResult>;

    /// Restart the transaction validation instance
    fn restart(&mut self, config: OnChainConfigPayload) -> Result<()>;

    /// Notify about new commit
    fn notify_commit(&mut self);
}

fn latest_state_view(db_reader: &Arc<dyn DbReader<DpnProto>>) -> VerifiedStateView<DpnProto> {
    let (version, state_root) = db_reader.get_latest_state_root().expect("Should not fail.");

    VerifiedStateView::new(
        StateViewId::TransactionValidation {
            base_version: version,
        },
        Arc::clone(db_reader),
        Some(version),
        state_root,
        SparseMerkleTree::new(state_root),
    )
}

pub struct VMValidator {
    db_reader: Arc<dyn DbReader<DpnProto>>,
    cached_state_view: VerifiedStateView<DpnProto>,
    vm: DiemVM,
}

impl Clone for VMValidator {
    fn clone(&self) -> Self {
        Self::new(self.db_reader.clone())
    }
}

impl VMValidator {
    pub fn new(db_reader: Arc<dyn DbReader<DpnProto>>) -> Self {
        let cached_state_view = latest_state_view(&db_reader);

        let vm = DiemVM::new_for_validation(&cached_state_view);
        VMValidator {
            db_reader,
            cached_state_view,
            vm,
        }
    }
}

impl TransactionValidation for VMValidator {
    type ValidationInstance = DiemVM;

    fn validate_transaction(&self, txn: SignedTransaction) -> Result<VMValidatorResult> {
        fail_point!("vm_validator::validate_transaction", |_| {
            Err(anyhow::anyhow!(
                "Injected error in vm_validator::validate_transaction"
            ))
        });
        use diem_vm::VMValidator;

        Ok(self.vm.validate_transaction(txn, &self.cached_state_view))
    }

    fn restart(&mut self, config: OnChainConfigPayload) -> Result<()> {
        self.notify_commit();
        let vm_config = config.get::<VMConfig>()?;
        let version = config.get::<DiemVersion>()?;
        let publishing_option = config.get::<VMPublishingOption>()?;

        self.vm = DiemVM::init_with_config(version, vm_config, publishing_option);
        Ok(())
    }

    fn notify_commit(&mut self) {
        self.cached_state_view = latest_state_view(&self.db_reader);
    }
}

/// returns account's sequence number from storage
pub fn get_account_sequence_number(
    storage: &dyn DbReader<DpnProto>,
    address: AccountAddress,
) -> Result<AccountSequenceInfo> {
    fail_point!("vm_validator::get_account_sequence_number", |_| {
        Err(anyhow::anyhow!(
            "Injected error in get_account_sequence_number"
        ))
    });
    match storage.get_latest_account_state(address)? {
        Some(blob) => {
            if let Ok(Some(crsn)) = AccountState::try_from(&blob)?.get_crsn_resource() {
                return Ok(AccountSequenceInfo::CRSN {
                    min_nonce: crsn.min_nonce(),
                    size: crsn.size(),
                });
            }
            Ok(AccountSequenceInfo::Sequential(
                AccountResource::try_from(&blob)?.sequence_number(),
            ))
        }
        None => Ok(AccountSequenceInfo::Sequential(0)),
    }
}
