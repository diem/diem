// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::*,
    create_access_path,
    data_cache::StateViewCache,
    diem_vm::{get_currency_info, DiemVMImpl},
    transaction_metadata::TransactionMetadata,
    VMValidator,
};
use diem_logger::prelude::*;
use diem_state_view::StateView;
use diem_types::{
    account_address::AccountAddress,
    account_config::{self, RoleId},
    on_chain_config::{DiemVersion, VMConfig, VMPublishingOption},
    transaction::{
        GovernanceRole, SignatureCheckedTransaction, SignedTransaction, TransactionPayload,
        VMValidatorResult,
    },
    vm_status::{StatusCode, VMStatus},
};
use move_core_types::{
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::IdentStr,
    move_resource::MoveResource,
};

use crate::logging::AdapterLogSchema;
use move_vm_types::gas_schedule::CostStrategy;

#[derive(Clone)]
pub struct DiemVMValidator(DiemVMImpl);

impl DiemVMValidator {
    pub fn new<S: StateView>(state: &S) -> Self {
        info!(
            AdapterLogSchema::new(state.id(), 0),
            "Adapter created for Validation"
        );
        Self(DiemVMImpl::new(state))
    }

    pub fn init_with_config(
        version: DiemVersion,
        on_chain_config: VMConfig,
        publishing_option: VMPublishingOption,
    ) -> Self {
        info!("Adapter restarted for Validation");
        DiemVMValidator(DiemVMImpl::init_with_config(
            version,
            on_chain_config,
            publishing_option,
        ))
    }

    fn verify_transaction_impl(
        &self,
        transaction: &SignatureCheckedTransaction,
        remote_cache: &StateViewCache,
        account_currency_symbol: &IdentStr,
    ) -> Result<(), VMStatus> {
        let txn_data = TransactionMetadata::new(transaction);
        let mut session = self.0.new_session(remote_cache);
        let log_context = AdapterLogSchema::new(remote_cache.id(), 0);
        let mut cost_strategy =
            CostStrategy::system(self.0.get_gas_schedule(&log_context)?, GasUnits::new(0));
        match transaction.payload() {
            TransactionPayload::Script(_script) => {
                self.0.check_gas(&txn_data, &log_context)?;
                self.0.run_script_prologue(
                    &mut session,
                    &mut cost_strategy,
                    &txn_data,
                    account_currency_symbol,
                    &log_context,
                )
            }
            TransactionPayload::Module(_module) => {
                self.0.check_gas(&txn_data, &log_context)?;
                self.0.run_module_prologue(
                    &mut session,
                    &mut cost_strategy,
                    &txn_data,
                    account_currency_symbol,
                    &log_context,
                )
            }
            TransactionPayload::WriteSet(_cs) => {
                self.0
                    .run_writeset_prologue(&mut session, &txn_data, &log_context)
            }
        }
    }
}

// VMValidator external API
impl VMValidator for DiemVMValidator {
    /// Determine if a transaction is valid. Will return `None` if the transaction is accepted,
    /// `Some(Err)` if the VM rejects it, with `Err` as an error code. Verification performs the
    /// following steps:
    /// 1. The signature on the `SignedTransaction` matches the public key included in the
    ///    transaction
    /// 2. The script to be executed is under given specific configuration.
    /// 3. Invokes `DiemAccount.prologue`, which checks properties such as the transaction has the
    /// right sequence number and the sender has enough balance to pay for the gas.
    /// TBD:
    /// 1. Transaction arguments matches the main function's type signature.
    ///    We don't check this item for now and would execute the check at execution time.
    fn validate_transaction(
        &self,
        transaction: SignedTransaction,
        state_view: &dyn StateView,
    ) -> VMValidatorResult {
        let data_cache = StateViewCache::new(state_view);
        let _timer = TXN_VALIDATION_SECONDS.start_timer();
        let gas_price = transaction.gas_unit_price();
        let currency_code =
            match account_config::from_currency_code_string(transaction.gas_currency_code()) {
                Ok(code) => code,
                Err(_) => {
                    return VMValidatorResult::new(
                        Some(StatusCode::INVALID_GAS_SPECIFIER),
                        gas_price,
                        GovernanceRole::NonGovernanceRole,
                    )
                }
            };

        let txn_sender = transaction.sender();
        let signature_verified_txn = if let Ok(t) = transaction.check_signature() {
            t
        } else {
            return VMValidatorResult::new(
                Some(StatusCode::INVALID_SIGNATURE),
                gas_price,
                GovernanceRole::NonGovernanceRole,
            );
        };

        let account_role = get_account_role(txn_sender, &data_cache);
        let normalized_gas_price = match get_currency_info(&currency_code, &data_cache) {
            Ok(info) => info.convert_to_xdx(gas_price),
            Err(err) => {
                return VMValidatorResult::new(
                    Some(err.status_code()),
                    gas_price,
                    GovernanceRole::NonGovernanceRole,
                )
            }
        };

        let res = match self.verify_transaction_impl(
            &signature_verified_txn,
            &data_cache,
            &currency_code,
        ) {
            Ok(_) => None,
            Err(err) => {
                if err.status_code() == StatusCode::SEQUENCE_NUMBER_TOO_NEW {
                    None
                } else {
                    Some(err)
                }
            }
        };

        // Increment the counter for transactions verified.
        let counter_label = match res {
            None => "success",
            Some(_) => "failure",
        };
        TRANSACTIONS_VALIDATED
            .with_label_values(&[counter_label])
            .inc();

        VMValidatorResult::new(
            res.map(|s| s.status_code()),
            normalized_gas_price,
            account_role,
        )
    }
}

fn get_account_role(sender: AccountAddress, remote_cache: &StateViewCache) -> GovernanceRole {
    let role_access_path = create_access_path(sender, RoleId::struct_tag());
    if let Ok(Some(blob)) = remote_cache.get(&role_access_path) {
        return bcs::from_bytes::<account_config::RoleId>(&blob)
            .map(|role_id| GovernanceRole::from_role_id(role_id.role_id()))
            .unwrap_or(GovernanceRole::NonGovernanceRole);
    }
    GovernanceRole::NonGovernanceRole
}
