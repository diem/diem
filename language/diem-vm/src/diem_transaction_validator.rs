// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    adapter_common::{validate_signed_transaction, VMAdapter},
    diem_transaction_executor::DiemVM,
    logging::AdapterLogSchema,
    transaction_metadata::TransactionMetadata,
    VMValidator,
};
use anyhow::Result;
use diem_state_view::StateView;
use diem_types::{
    on_chain_config::{DIEM_VERSION_2, DIEM_VERSION_3},
    transaction::{
        SignatureCheckedTransaction, SignedTransaction, TransactionPayload, VMValidatorResult,
    },
};
use move_core_types::{
    identifier::Identifier,
    vm_status::{StatusCode, VMStatus},
};
use move_vm_runtime::{data_cache::MoveStorage, session::Session};

impl VMAdapter for DiemVM {
    fn new_session<'r, R: MoveStorage>(&self, remote: &'r R) -> Session<'r, '_, R> {
        self.0.new_session(remote)
    }

    fn check_signature(txn: SignedTransaction) -> Result<SignatureCheckedTransaction> {
        txn.check_signature()
    }

    fn allows_multi_agent(&self) -> Result<bool, VMStatus> {
        Ok(self.0.get_diem_version()? >= DIEM_VERSION_3)
    }

    fn run_prologue<S: MoveStorage>(
        &self,
        session: &mut Session<S>,
        transaction: &SignatureCheckedTransaction,
        currency_code: &Identifier,
        log_context: &AdapterLogSchema,
    ) -> Result<(), VMStatus> {
        let txn_data = TransactionMetadata::new(transaction);
        match transaction.payload() {
            TransactionPayload::Script(_) => {
                self.0.check_gas(&txn_data, log_context)?;
                self.0
                    .run_script_prologue(session, &txn_data, &currency_code, log_context)
            }
            TransactionPayload::ScriptFunction(_) => {
                // gate the behavior until the Diem version is ready
                if self.0.get_diem_version()? < DIEM_VERSION_2 {
                    return Err(VMStatus::Error(StatusCode::FEATURE_UNDER_GATING));
                }
                // NOTE: Script and ScriptFunction shares the same prologue
                self.0.check_gas(&txn_data, log_context)?;
                self.0
                    .run_script_prologue(session, &txn_data, &currency_code, log_context)
            }
            TransactionPayload::Module(_module) => {
                self.0.check_gas(&txn_data, log_context)?;
                self.0
                    .run_module_prologue(session, &txn_data, &currency_code, log_context)
            }
            TransactionPayload::WriteSet(_cs) => {
                self.0
                    .run_writeset_prologue(session, &txn_data, log_context)
            }
        }
    }
}

// VMValidator external API
impl VMValidator for DiemVM {
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
        validate_signed_transaction::<Self>(&self, transaction, state_view)
    }
}
