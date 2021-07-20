use crate::{counters::*, create_access_path, data_cache::StateViewCache};
use anyhow::Result;
use diem_state_view::StateView;
use diem_types::{
    account_address::AccountAddress,
    account_config::{self, CurrencyInfoResource, RoleId},
    transaction::{
        GovernanceRole, SignatureCheckedTransaction, SignedTransaction, VMValidatorResult,
    },
    vm_status::{StatusCode, VMStatus},
};
use move_core_types::{
    identifier::{IdentStr, Identifier},
    move_resource::MoveStructType,
};
use move_vm_runtime::{data_cache::MoveStorage, session::Session};

use crate::logging::AdapterLogSchema;

/// This trait describes the VM adapter's interface.
pub trait VMAdapter {
    /// Creates a new Session backed by the given storage.
    /// TODO: this probably doesn't belong in this trait. We should be able to remove
    /// this after refactoring Diem transaction executor code.
    fn new_session<'r, R: MoveStorage>(&self, remote: &'r R) -> Session<'r, '_, R>;

    /// Checks the signature of the given signed transaction and returns
    /// `Ok(SignatureCheckedTransaction)` if the signature is valid.
    fn check_signature(txn: SignedTransaction) -> Result<SignatureCheckedTransaction>;

    /// Returns true if multi agent transactions are allowed.
    fn allows_multi_agent(&self) -> Result<bool, VMStatus>;

    /// Runs the prologue for the given transaction.
    fn run_prologue<S: MoveStorage>(
        &self,
        session: &mut Session<S>,
        transaction: &SignatureCheckedTransaction,
        currency_code: &Identifier,
        log_context: &AdapterLogSchema,
    ) -> Result<(), VMStatus>;
}

/// Validate a signed transaction by performing the following:
/// 1. Check the signature(s) included in the signed transaction
/// 2. Check that the transaction is allowed in the context provided by the `adapter`
/// 3. Run the prologue to perform additional on-chain checks
/// The returned `VMValidatorResult` will have status `None` and if all checks succeeded
/// and `Some(DiscardedVMStatus)` otherwise.
pub fn validate_signed_transaction<A: VMAdapter>(
    adapter: &A,
    transaction: SignedTransaction,
    state_view: &dyn StateView,
) -> VMValidatorResult {
    let _timer = TXN_VALIDATION_SECONDS.start_timer();
    let txn_sender = transaction.sender();
    let log_context = AdapterLogSchema::new(state_view.id(), 0);

    let txn = if let Ok(t) = A::check_signature(transaction) {
        t
    } else {
        return VMValidatorResult::error(StatusCode::INVALID_SIGNATURE);
    };

    let remote_cache = StateViewCache::new(state_view);
    let account_role = get_account_role(txn_sender, &remote_cache);
    let mut session = adapter.new_session(&remote_cache);

    let (status, normalized_gas_price) = match validate_signature_checked_transaction(
        adapter,
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

fn get_account_role(sender: AccountAddress, remote_cache: &StateViewCache) -> GovernanceRole {
    let role_access_path = create_access_path(sender, RoleId::struct_tag());
    if let Ok(Some(blob)) = remote_cache.get(&role_access_path) {
        return bcs::from_bytes::<account_config::RoleId>(&blob)
            .map(|role_id| GovernanceRole::from_role_id(role_id.role_id()))
            .unwrap_or(GovernanceRole::NonGovernanceRole);
    }
    GovernanceRole::NonGovernanceRole
}

pub(crate) fn validate_signature_checked_transaction<S: MoveStorage, A: VMAdapter>(
    adapter: &A,
    mut session: &mut Session<S>,
    transaction: &SignatureCheckedTransaction,
    remote_cache: &S,
    allow_too_new: bool,
    log_context: &AdapterLogSchema,
) -> Result<(u64, Identifier), VMStatus> {
    if transaction.is_multi_agent() && !adapter.allows_multi_agent()? {
        // Multi agent is not allowed
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

    // let txn_data = TransactionMetadata::new(transaction);
    let prologue_status =
        adapter.run_prologue(&mut session, transaction, &currency_code, log_context);

    if let Err(err) = prologue_status {
        // Accept "future" sequence numbers during the validation phase so that multiple
        // transactions from the same account can be in mempool together.
        if !allow_too_new || err.status_code() != StatusCode::SEQUENCE_NUMBER_TOO_NEW {
            return Err(err);
        }
    }
    Ok((normalized_gas_price, currency_code))
}

fn get_currency_info<S: MoveStorage>(
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
