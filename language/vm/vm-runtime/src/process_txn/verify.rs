// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    code_cache::{module_cache::ModuleCache, script_cache::ScriptCache},
    loaded_data::function::{FunctionRef, FunctionReference},
    process_txn::{execute::ExecutedTransaction, validate::ValidatedTransaction},
    txn_executor::TransactionExecutor,
};
use bytecode_verifier::VerifiedModule;
use libra_logger::prelude::*;
use libra_types::transaction::ScriptAction;
use libra_types::{
    account_address::AccountAddress,
    transaction::{
        ChannelTransactionPayloadBody, Module, Script, SignatureCheckedTransaction, TransactionArgument, TransactionPayload,
    },
    vm_error::{StatusCode, VMStatus},
};
use vm::{
    access::ModuleAccess,
    errors::verification_error,
    file_format::{CompiledModule, FunctionSignature, SignatureToken},
    IndexKind,
};

/// Represents a transaction which has been validated and for which the program has been run
/// through the bytecode verifier.
pub struct VerifiedTransaction<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    txn: SignatureCheckedTransaction,
    #[allow(dead_code)]
    txn_state: Option<VerifiedTransactionState<'alloc, 'txn, P>>,
}

impl<'alloc, 'txn, P> VerifiedTransaction<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    /// Creates a new instance by verifying the bytecode in this validated transaction.
    pub(super) fn new(
        mut validated_txn: ValidatedTransaction<'alloc, 'txn, P>,
        script_cache: &'txn ScriptCache<'alloc>,
    ) -> Result<Self, VMStatus> {
        let txn_state = validated_txn.take_state();
        let txn = validated_txn.as_inner();
        let txn_state = match txn.payload() {
            TransactionPayload::Program => return Err(VMStatus::new(StatusCode::MALFORMED)),
            TransactionPayload::WriteSet(_write_set) => {
                // All the checks are performed in validation, so there's no need for more checks
                // here.
                None
            }
            TransactionPayload::Module(module) => {
                let txn_state = txn_state
                    .expect("module-based transactions should always have associated state");

                let verified_module = Self::verify_module(&txn.sender(), module)?;

                Some(VerifiedTransactionState {
                    txn_executor: txn_state.txn_executor,
                    verified_txn: VerTxn::Module(Box::new(verified_module)),
                })
            }
            TransactionPayload::Script(script) => {
                let txn_state = txn_state
                    .expect("script-based transactions should always have associated state");

                let main = Self::verify_script(script, script_cache)?;

                Some(VerifiedTransactionState {
                    txn_executor: txn_state.txn_executor,
                    verified_txn: VerTxn::Script(main),
                })
            }
            TransactionPayload::Channel(channel_payload) => {
                match &channel_payload.body {
                    ChannelTransactionPayloadBody::WriteSet(_) => {
                        //TODO(jole) do more verify.
                        None
                    }
                    ChannelTransactionPayloadBody::Script(script_body) => {
                        //TODO(jole) do more verify.
                        let txn_state = txn_state.expect(
                            "script-based transactions should always have associated state",
                        );

                        let main = Self::verify_script(&script_body.script, script_cache)?;

                        Some(VerifiedTransactionState {
                            txn_executor: txn_state.txn_executor,
                            verified_txn: VerTxn::Script(main),
                        })
                    }
                    ChannelTransactionPayloadBody::Action(action_body) => {
                        //TODO(jole) do more verify.
                        let txn_state = txn_state.expect(
                            "script-based transactions should always have associated state",
                        );

                        Self::verify_script_action(
                            action_body.action(),
                            &txn_state.txn_executor.module_cache(),
                        )?;

                        Some(VerifiedTransactionState {
                            txn_executor: txn_state.txn_executor,
                            verified_txn: VerTxn::ScriptAction(),
                        })
                    }
                }
            }
        };

        Ok(Self {
            txn: validated_txn.into_inner(),
            txn_state,
        })
    }

    fn verify_module(
        sender_address: &AccountAddress,
        module: &Module,
    ) -> Result<VerifiedModule, VMStatus> {
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
        if compiled_module.address() != sender_address {
            return Err(verification_error(
                IndexKind::AddressPool,
                CompiledModule::IMPLEMENTED_MODULE_INDEX as usize,
                StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER,
            ));
        }

        match VerifiedModule::new(compiled_module) {
            Ok(ver_module) => Ok(ver_module),
            Err((_, mut errors)) => {
                let err = if errors.is_empty() {
                    VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION)
                } else {
                    errors.remove(0)
                };

                Err(err)
            }
        }
    }

    fn verify_script(
        script: &Script,
        script_cache: &'txn ScriptCache<'alloc>,
    ) -> Result<FunctionRef<'alloc>, VMStatus> {
        // Ensure the script can correctly be resolved into main.
        let main = script_cache.cache_script(&script.code())?;

        if !verify_actuals(main.signature(), script.args()) {
            return Err(VMStatus::new(StatusCode::TYPE_MISMATCH)
                .with_message("Actual Type Mismatch".to_string()));
        }

        Ok(main)
    }

    fn verify_script_action(
        action: &ScriptAction,
        module_cache: &TransactionModuleCache<'alloc, 'txn, P>,
    ) -> Result<(), VMStatus> {
        let loaded_module = module_cache
            .get_loaded_module(action.module())?
            .ok_or_else(|| {
                VMStatus::new(StatusCode::LINKER_ERROR).with_message(format!(
                    "Can not find module when verify script action: {:?}",
                    action
                ))
            })?;
        let func_idx = loaded_module
            .function_defs_table
            .get(action.function())
            .ok_or_else(|| {
                VMStatus::new(StatusCode::LINKER_ERROR).with_message(format!(
                    "Can not find function when verify script action: {:?}",
                    action
                ))
            })?;
        let func = FunctionRef::new(loaded_module, *func_idx);

        if !verify_actuals(func.signature(), action.args()) {
            return Err(
                VMStatus::new(StatusCode::TYPE_MISMATCH).with_message(format!(
                    "Actual Type Mismatch. function signature: {:?}",
                    func.signature()
                )),
            );
        }
        //TODO return function ref and keep in State.
        Ok(())
    }

    /// Executes this transaction.
    pub fn execute(self) -> ExecutedTransaction {
        ExecutedTransaction::new(self)
    }

    /// Returns the state stored in the transaction, if any.
    pub(super) fn take_state(&mut self) -> Option<VerifiedTransactionState<'alloc, 'txn, P>> {
        self.txn_state.take()
    }

    /// Returns a reference to the `SignatureCheckedTransaction` within.
    #[allow(dead_code)]
    pub fn as_inner(&self) -> &SignatureCheckedTransaction {
        &self.txn
    }

    /// Consumes `self` and returns the `SignatureCheckedTransaction` within.
    pub fn into_inner(self) -> SignatureCheckedTransaction {
        self.txn
    }
}

/// State for [`VerifiedTransaction`] instances.
#[allow(dead_code)]
pub(super) struct VerifiedTransactionState<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    pub(super) txn_executor: TransactionExecutor<'alloc, 'txn, P>,
    pub(super) verified_txn: VerTxn<'alloc>,
}

/// A verified transaction is a transaction executing code that has gone through the verifier.
///
/// It can be a program, a script or a module. A transaction script gets executed by the VM.
/// A module script publishes the module provided.
pub enum VerTxn<'alloc> {
    Script(FunctionRef<'alloc>),
    Module(Box<VerifiedModule>),
    //TODO ScriptAction will be a FunctionRef with lifetime 'txn
    ScriptAction(),
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
            (SignatureToken::Bool, TransactionArgument::Bool(_)) => (),
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
