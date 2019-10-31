use crate::{
    code_cache::{
        module_cache::{ModuleCache, TransactionModuleCache},
        script_cache::ScriptCache,
    },
    loaded_data::function::{FunctionRef, FunctionReference},
    process_txn::{execute::ExecutedTransaction, validate::ValidatedTransaction},
    txn_executor::TransactionExecutor,
};
use bytecode_verifier::{VerifiedModule, VerifiedScript};
use libra_logger::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    transaction::{
        ChannelTransactionPayloadBody, Module, Program, Script, SignatureCheckedTransaction,
        TransactionArgument, TransactionPayload,
    },
    vm_error::{StatusCode, VMStatus},
};
use vm::{
    access::ModuleAccess,
    errors::{verification_error, VMResult},
    file_format::{CompiledModule, CompiledScript, FunctionSignature, SignatureToken},
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
            TransactionPayload::Program(program) => {
                let txn_state = txn_state
                    .expect("program-based transactions should always have associated state");

                let (main, modules) = Self::verify_program(&txn.sender(), program, script_cache)?;

                Some(VerifiedTransactionState {
                    txn_executor: txn_state.txn_executor,
                    verified_txn: VerTxn::Program(VerProgram { main, modules }),
                })
            }
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
                }
            }
        };

        Ok(Self {
            txn: validated_txn.into_inner(),
            txn_state,
        })
    }

    fn verify_program(
        sender_address: &AccountAddress,
        program: &Program,
        script_cache: &'txn ScriptCache<'alloc>,
    ) -> VMResult<(FunctionRef<'alloc>, Vec<VerifiedModule>)> {
        // Ensure the script can correctly be resolved into main.
        let main = script_cache.cache_script(&program.code())?;

        if !verify_actuals(main.signature(), program.args()) {
            return Err(VMStatus::new(StatusCode::TYPE_MISMATCH)
                .with_message("Actual Type Mismatch".to_string()));
        }

        // Make sure all the modules trying to be published in this module are valid.
        let modules: Vec<CompiledModule> = match program
            .modules()
            .iter()
            .map(|module_blob| CompiledModule::deserialize(&module_blob))
            .collect()
        {
            Ok(modules) => modules,
            Err(err) => {
                warn!("[VM] module deserialization failed {:?}", err);
                return Err(err);
            }
        };

        // Run the modules through the bytecode verifier.
        let modules = match static_verify_modules(sender_address, modules) {
            Ok(modules) => modules,
            Err(mut statuses) => {
                warn!("[VM] bytecode verifier returned errors");
                let err = if statuses.is_empty() {
                    VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION)
                } else {
                    statuses.remove(0)
                };
                return Err(err);
            }
        };

        Ok((main, modules))
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
    pub(super) txn_executor:
        TransactionExecutor<'txn, 'txn, TransactionModuleCache<'alloc, 'txn, P>>,
    pub(super) verified_txn: VerTxn<'alloc>,
}

/// A verified `Program` is a main and a list of modules.
pub struct VerProgram<'alloc> {
    pub(super) main: FunctionRef<'alloc>,
    pub(super) modules: Vec<VerifiedModule>,
}

/// A verified transaction is a transaction executing code that has gone through the verifier.
///
/// It can be a program, a script or a module. A transaction script gets executed by the VM.
/// A module script publishes the module provided.
// TODO: A Script will be a FunctionRef once we remove the ability to publish in scripts.
pub enum VerTxn<'alloc> {
    Program(VerProgram<'alloc>),
    Script(FunctionRef<'alloc>),
    Module(Box<VerifiedModule>),
}

fn static_verify_modules(
    sender_address: &AccountAddress,
    modules: Vec<CompiledModule>,
) -> Result<Vec<VerifiedModule>, Vec<VMStatus>> {
    // It is possible to write this function without the expects, but that makes it very ugly.
    let mut statuses: Vec<Box<dyn Iterator<Item = VMStatus>>> = vec![];

    let modules_len = modules.len();

    let mut modules_out = vec![];
    for module in modules.into_iter() {
        // Make sure the module's self address matches the transaction sender. The self address is
        // where the module will actually be published. If we did not check this, the sender could
        // publish a module under anyone's account.
        //
        // For scripts this isn't a problem because they don't get published to accounts.
        let self_error = if module.address() != sender_address {
            Some(verification_error(
                IndexKind::AddressPool,
                CompiledModule::IMPLEMENTED_MODULE_INDEX as usize,
                StatusCode::MODULE_ADDRESS_DOES_NOT_MATCH_SENDER,
            ))
        } else {
            None
        };

        let (module, mut errors) = match VerifiedModule::new(module) {
            Ok(module) => (Some(module), vec![]),
            Err((_, errors)) => (None, errors),
        };

        if let Some(error) = self_error {
            // Verification should stop before we generate enough errors to overflow
            assume!(errors.len() < usize::max_value());
            errors.push(error);
        }

        if errors.is_empty() {
            // `modules_out` is initally empty, a single element is pushed per loop iteration and
            // the number of iterations is bound to the max size of `modules``
            assume!(modules_out.len() < usize::max_value());
            modules_out.push(module.expect("empty errors => module should verify"));
        } else {
            statuses.push(Box::new(errors.into_iter()));
        }
    }

    let statuses: Vec<_> = statuses.into_iter().flatten().collect();
    if statuses.is_empty() {
        assert_eq!(modules_out.len(), modules_len);
        Ok(modules_out)
    } else {
        Err(statuses)
    }
}

/// Run static checks on a program directly. Provided as an alternative API for tests.
pub fn static_verify_program(
    sender_address: &AccountAddress,
    script: CompiledScript,
    modules: Vec<CompiledModule>,
) -> Result<(VerifiedScript, Vec<VerifiedModule>), Vec<VMStatus>> {
    // It is possible to write this function without the expects, but that makes it very ugly.
    let mut statuses: Vec<VMStatus> = vec![];
    let script = match VerifiedScript::new(script) {
        Ok(script) => Some(script),
        Err((_, errors)) => {
            statuses.extend(errors.into_iter());
            None
        }
    };

    let modules = match static_verify_modules(sender_address, modules) {
        Ok(modules) => Some(modules),
        Err(module_statuses) => {
            statuses.extend(module_statuses);
            None
        }
    };

    if statuses.is_empty() {
        Ok((
            script.expect("Ok case => script should verify"),
            modules.expect("Ok case => modules should verify"),
        ))
    } else {
        Err(statuses)
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
