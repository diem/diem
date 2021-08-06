// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::*, create_access_path, data_cache::StateViewCache, diem_vm::DiemVMImpl,
    transaction_metadata::TransactionMetadata, VMValidator,
};
use diem_logger::prelude::*;
use diem_state_view::StateView;
use diem_types::{
    account_address::AccountAddress,
    account_config::{self, CurrencyInfoResource, RoleId},
    on_chain_config::{DiemVersion, VMConfig, VMPublishingOption, DIEM_VERSION_2, DIEM_VERSION_3},
    transaction::{
        GovernanceRole, SignatureCheckedTransaction, SignedTransaction, TransactionPayload,
        VMValidatorResult,
    },
    vm_status::{StatusCode, VMStatus},
};
use move_core_types::{
    identifier::{IdentStr, Identifier},
    move_resource::MoveStructType,
    resolver::MoveResolver,
};
use move_vm_runtime::session::Session;

use crate::logging::AdapterLogSchema;

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
        let _timer = TXN_VALIDATION_SECONDS.start_timer();
        let txn_sender = transaction.sender();
        let log_context = AdapterLogSchema::new(state_view.id(), 0);

        let txn = if let Ok(t) = transaction.check_signature() {
            t
        } else {
            return VMValidatorResult::error(StatusCode::INVALID_SIGNATURE);
        };

        let remote_cache = StateViewCache::new(state_view);
        let account_role = get_account_role(txn_sender, &remote_cache);
        let mut session = self.0.new_session(&remote_cache);

        let (status, normalized_gas_price) = match validate_signature_checked_transaction(
            &self.0,
            &mut session,
            &txn,
            &remote_cache,
            true,
            &log_context,
        ) {
            Ok((price, _)) => (None, price),
            Err(err) => (Some(err.status_code()), 0),
        };

        // Increment the counter for transactions verified.
        let counter_label = match status {
            None => "success",
            Some(_) => "failure",
        };
        TRANSACTIONS_VALIDATED
            .with_label_values(&[counter_label])
            .inc();

        VMValidatorResult::new(status, normalized_gas_price, account_role)
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

pub(crate) fn validate_signature_checked_transaction<S: MoveResolver>(
    vm: &DiemVMImpl,
    session: &mut Session<S>,
    transaction: &SignatureCheckedTransaction,
    remote_cache: &S,
    allow_too_new: bool,
    log_context: &AdapterLogSchema,
) -> Result<(u64, Identifier), VMStatus> {
    if transaction.is_multi_agent() && vm.get_diem_version()? < DIEM_VERSION_3 {
        // Multi agent is not allowed under this version
        return Err(VMStatus::Error(StatusCode::FEATURE_UNDER_GATING));
    }

    if transaction.contains_duplicate_signers() {
        return Err(VMStatus::Error(StatusCode::SIGNERS_CONTAIN_DUPLICATES));
    }

    let gas_price = transaction.gas_unit_price();
    let currency_code_string = transaction.gas_currency_code();
    let currency_code = match account_config::from_currency_code_string(currency_code_string) {
        Ok(code) => code,
        Err(_) => {
            return Err(VMStatus::Error(StatusCode::INVALID_GAS_SPECIFIER));
        }
    };

    let normalized_gas_price = match get_currency_info(&currency_code, remote_cache) {
        Ok(info) => info.convert_to_xdx(gas_price),
        Err(err) => {
            return Err(err);
        }
    };

    let txn_data = TransactionMetadata::new(transaction);
    let prologue_status = match transaction.payload() {
        TransactionPayload::Script(_) => {
            vm.check_gas(&txn_data, log_context)?;
            vm.run_script_prologue(session, &txn_data, &currency_code, log_context)
        }
        TransactionPayload::ScriptFunction(_) => {
            // gate the behavior until the Diem version is ready
            if vm.get_diem_version()? < DIEM_VERSION_2 {
                return Err(VMStatus::Error(StatusCode::FEATURE_UNDER_GATING));
            }
            // NOTE: Script and ScriptFunction shares the same prologue
            vm.check_gas(&txn_data, log_context)?;
            vm.run_script_prologue(session, &txn_data, &currency_code, log_context)
        }
        TransactionPayload::Module(_module) => {
            vm.check_gas(&txn_data, log_context)?;
            vm.run_module_prologue(session, &txn_data, &currency_code, log_context)
        }
        TransactionPayload::WriteSet(_cs) => {
            vm.run_writeset_prologue(session, &txn_data, log_context)
        }
    };

    if let Err(err) = prologue_status {
        // Accept "future" sequence numbers during the validation phase so that multiple
        // transactions from the same account can be in mempool together.
        if !allow_too_new || err.status_code() != StatusCode::SEQUENCE_NUMBER_TOO_NEW {
            return Err(err);
        }
    }
    Ok((normalized_gas_price, currency_code))
}

fn get_currency_info<S: MoveResolver>(
    currency_code: &IdentStr,
    remote_cache: &S,
) -> Result<CurrencyInfoResource, VMStatus> {
    if let Ok(Some(blob)) = remote_cache.get_resource(
        &account_config::diem_root_address(),
        &CurrencyInfoResource::struct_tag_for(currency_code.to_owned()),
    ) {
        let x = bcs::from_bytes::<CurrencyInfoResource>(&blob)
            .map_err(|_| VMStatus::Error(StatusCode::CURRENCY_INFO_DOES_NOT_EXIST))?;
        Ok(x)
    } else {
        Err(VMStatus::Error(StatusCode::CURRENCY_INFO_DOES_NOT_EXIST))
    }
}
