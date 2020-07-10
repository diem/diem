// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::*, create_access_path, data_cache::StateViewCache, libra_vm::LibraVMImpl,
    transaction_metadata::TransactionMetadata, VMValidator,
};
use libra_state_view::StateView;
use libra_types::{
    account_address::AccountAddress,
    account_config::{self, RoleId},
    on_chain_config::{LibraVersion, VMConfig},
    transaction::{
        SignatureCheckedTransaction, SignedTransaction, TransactionPayload, VMValidatorResult,
    },
    vm_status::{convert_prologue_runtime_error, StatusCode, VMStatus},
};
use move_core_types::{
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::IdentStr,
    move_resource::MoveResource,
};

use move_vm_types::gas_schedule::CostStrategy;

/// Any transation sent from an account with a role id below this cutoff will be priorited over
/// other transactions.
const PRIORITIZED_TRANSACTION_ROLE_CUTOFF: u64 = 5;

#[derive(Clone)]
pub struct LibraVMValidator(LibraVMImpl);

impl LibraVMValidator {
    pub fn new<S: StateView>(state: &S) -> Self {
        Self(LibraVMImpl::new(state))
    }

    pub fn init_with_config(version: LibraVersion, on_chain_config: VMConfig) -> Self {
        LibraVMValidator(LibraVMImpl::init_with_config(version, on_chain_config))
    }

    fn verify_transaction_impl(
        &self,
        transaction: &SignatureCheckedTransaction,
        remote_cache: &StateViewCache,
        account_currency_symbol: &IdentStr,
    ) -> Result<(), VMStatus> {
        let txn_data = TransactionMetadata::new(transaction);
        let mut session = self.0.new_session(remote_cache);
        let mut cost_strategy = CostStrategy::system(self.0.get_gas_schedule()?, GasUnits::new(0));
        match transaction.payload() {
            TransactionPayload::Script(script) => {
                self.0.check_gas(&txn_data)?;
                self.0.is_allowed_script(script)?;
                self.0.run_prologue(
                    &mut session,
                    &mut cost_strategy,
                    &txn_data,
                    account_currency_symbol,
                )
            }
            TransactionPayload::Module(_module) => {
                self.0.check_gas(&txn_data)?;
                self.0.is_allowed_module(&txn_data, remote_cache)?;
                self.0.run_prologue(
                    &mut session,
                    &mut cost_strategy,
                    &txn_data,
                    account_currency_symbol,
                )
            }
            TransactionPayload::WriteSet(_cs) => {
                self.0.run_writeset_prologue(&mut session, &txn_data)
            }
        }
    }
}

// VMValidator external API
impl VMValidator for LibraVMValidator {
    /// Determine if a transaction is valid. Will return `None` if the transaction is accepted,
    /// `Some(Err)` if the VM rejects it, with `Err` as an error code. Verification performs the
    /// following steps:
    /// 1. The signature on the `SignedTransaction` matches the public key included in the
    ///    transaction
    /// 2. The script to be executed is under given specific configuration.
    /// 3. Invokes `LibraAccount.prologue`, which checks properties such as the transaction has the
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
                        Some(VMStatus::Error(StatusCode::INVALID_GAS_SPECIFIER)),
                        gas_price,
                        false,
                    )
                }
            };

        let txn_sender = transaction.sender();
        let signature_verified_txn = if let Ok(t) = transaction.check_signature() {
            t
        } else {
            return VMValidatorResult::new(
                Some(VMStatus::Error(StatusCode::INVALID_SIGNATURE)),
                gas_price,
                false,
            );
        };

        let is_prioritized_txn = is_prioritized_txn(txn_sender, &data_cache);
        let normalized_gas_price = match normalize_gas_price(gas_price, &currency_code, &data_cache)
        {
            Ok(price) => price,
            Err(err) => return VMValidatorResult::new(Some(err), gas_price, false),
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
                    Some(convert_prologue_runtime_error(err))
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

        VMValidatorResult::new(res, normalized_gas_price, is_prioritized_txn)
    }
}

fn is_prioritized_txn(sender: AccountAddress, remote_cache: &StateViewCache) -> bool {
    let role_access_path = create_access_path(sender, RoleId::struct_tag());
    if let Ok(Some(blob)) = remote_cache.get(&role_access_path) {
        return lcs::from_bytes::<account_config::RoleId>(&blob)
            .map(|role_id| role_id.role_id() < PRIORITIZED_TRANSACTION_ROLE_CUTOFF)
            .unwrap_or(false);
    }
    false
}

fn normalize_gas_price(
    gas_price: u64,
    currency_code: &IdentStr,
    remote_cache: &StateViewCache,
) -> Result<u64, VMStatus> {
    let currency_info_path =
        account_config::CurrencyInfoResource::resource_path_for(currency_code.to_owned());
    if let Ok(Some(blob)) = remote_cache.get(&currency_info_path) {
        let x = lcs::from_bytes::<account_config::CurrencyInfoResource>(&blob)
            .map_err(|_| VMStatus::Error(StatusCode::CURRENCY_INFO_DOES_NOT_EXIST))?;
        Ok(x.convert_to_lbr(gas_price))
    } else {
        Err(VMStatus::Error(StatusCode::MISSING_DATA))
    }
}
