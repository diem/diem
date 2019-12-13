// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::process_txn::{execute::ExecutedTransaction, validate::ValidatedTransaction};
use crate::runtime::VMRuntime;
use crate::txn_executor::TransactionExecutor;
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::transaction::TransactionArgument;
use libra_types::vm_error::VMStatus;
use libra_types::{
    transaction::{SignatureCheckedTransaction, TransactionPayload},
    vm_error::StatusCode,
};
use vm::{
    access::ModuleAccess,
    access::ScriptAccess,
    errors::{verification_error, vm_error, Location},
    file_format::{CompiledModule, CompiledScript, FunctionSignature, SignatureToken},
    transaction_metadata::TransactionMetadata,
    IndexKind,
};

/// Represents a transaction which has been verified and has a valid binary format.
/// A valid binary format has not been verified by the bytecode verifier.
/// The transaction consistency is verified here.
pub struct VerifiedTransaction {
    txn: SignatureCheckedTransaction,
}

impl VerifiedTransaction {
    /// Creates a new instance by validating the bytecode in this validated transaction.
    pub(super) fn new(
        validated_txn: ValidatedTransaction,
        txn_data: &TransactionMetadata,
    ) -> Result<Self, VMStatus> {
        let txn = validated_txn.into_inner();

        match txn.payload() {
            TransactionPayload::Program => return Err(VMStatus::new(StatusCode::MALFORMED)),
            TransactionPayload::Module(module) => {
                let compiled_module = match CompiledModule::deserialize(module.code()) {
                    Ok(module) => module,
                    Err(err) => {
                        warn!("[VM] module deserialization failed {:?}", err);
                        return Err(err);
                    }
                };

                // Make sure the module's self address matches the transaction sender. The self address is
                // where the module will actually be published. If we did not check this, the sender could
                // publish a module under anyone's account.
                if compiled_module.address() != &txn_data.sender {
                    return Err(verification_error(
                        IndexKind::AddressPool,
                        CompiledModule::IMPLEMENTED_MODULE_INDEX as usize,
                        StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER,
                    ));
                }
            }
            TransactionPayload::Script(script) => {
                let compiled_script = match CompiledScript::deserialize(script.code()) {
                    Ok(script) => script,
                    Err(err) => {
                        warn!("[VM] deserializer returned error for script: {:?}", err);
                        let error =
                            vm_error(Location::default(), StatusCode::CODE_DESERIALIZATION_ERROR)
                                .append(err);
                        return Err(error);
                    }
                };

                let fh = compiled_script.function_handle_at(compiled_script.main().function);
                let sig = compiled_script.function_signature_at(fh.signature);
                if !verify_actuals(sig, script.args()) {
                    return Err(VMStatus::new(StatusCode::TYPE_MISMATCH)
                        .with_message("Actual Type Mismatch".to_string()));
                }
            }
            TransactionPayload::WriteSet(_) => {
                // All the checks are performed in validation, so there's no need for more checks
                // here.
                // Bytecode verification is performed during execution
            }
        }

        Ok(Self { txn })
    }

    /// Executes this transaction.
    pub fn execute(
        self,
        runtime: &VMRuntime,
        state_view: &dyn StateView,
        executor: &mut TransactionExecutor,
    ) -> ExecutedTransaction {
        ExecutedTransaction::new(self, runtime, state_view, executor)
    }

    /// Consumes `self` and returns the `SignatureCheckedTransaction` within.
    pub fn into_inner(self) -> SignatureCheckedTransaction {
        self.txn
    }
}

/// Verify if the transaction arguments match the type signature of the main function.
fn verify_actuals(signature: &FunctionSignature, args: &[TransactionArgument]) -> bool {
    if signature.arg_types.len() != args.len() {
        warn!(
            "[VM] different argument length: actuals {}, formals {}",
            args.len(),
            signature.arg_types.len()
        );
        return false;
    }
    for (ty, arg) in signature.arg_types.iter().zip(args.iter()) {
        match (ty, arg) {
            (SignatureToken::U64, TransactionArgument::U64(_)) => (),
            (SignatureToken::Address, TransactionArgument::Address(_)) => (),
            (SignatureToken::ByteArray, TransactionArgument::ByteArray(_)) => (),
            (SignatureToken::String, TransactionArgument::String(_)) => (),
            _ => {
                warn!(
                    "[VM] different argument type: formal {:?}, actual {:?}",
                    ty, arg
                );
                return false;
            }
        }
    }
    true
}
