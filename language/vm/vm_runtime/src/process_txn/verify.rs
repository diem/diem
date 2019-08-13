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
use logger::prelude::*;
use types::{
    account_address::AccountAddress,
    transaction::{Program, SignatureCheckedTransaction, TransactionArgument, TransactionPayload},
    vm_error::{VMStatus, VMVerificationError, VMVerificationStatus},
};
use vm::{
    access::ModuleAccess,
    errors::{VMStaticViolation, VerificationError, VerificationStatus},
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
                    main,
                    modules,
                })
            }
            TransactionPayload::WriteSet(_write_set) => {
                // All the checks are performed in validation, so there's no need for more checks
                // here.
                None
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
    ) -> Result<(FunctionRef<'alloc>, Vec<VerifiedModule>), VMStatus> {
        // Ensure the script can correctly be resolved into main.
        let main = match script_cache.cache_script(&program.code()) {
            Ok(Ok(main)) => main,
            Ok(Err(ref err)) => return Err(err.into()),
            Err(ref err) => return Err(err.into()),
        };

        if !verify_actuals(main.signature(), program.args()) {
            return Err(VMStatus::Verification(vec![VMVerificationStatus::Script(
                VMVerificationError::TypeMismatch("Actual Type Mismatch".to_string()),
            )]));
        }

        // Make sure all the modules trying to be published in this module are valid.
        let modules: Vec<CompiledModule> = match program
            .modules()
            .iter()
            .map(|module_blob| CompiledModule::deserialize(&module_blob))
            .collect()
        {
            Ok(modules) => modules,
            Err(ref err) => {
                warn!("[VM] module deserialization failed {:?}", err);
                return Err(err.into());
            }
        };

        // Run the modules through the bytecode verifier.
        let modules = match static_verify_modules(sender_address, modules) {
            Ok(modules) => modules,
            Err(statuses) => {
                warn!("[VM] bytecode verifier returned errors");
                return Err(statuses.iter().collect());
            }
        };

        Ok((main, modules))
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

/// State for program-based [`VerifiedTransaction`] instances.
#[allow(dead_code)]
pub(super) struct VerifiedTransactionState<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    pub(super) txn_executor:
        TransactionExecutor<'txn, 'txn, TransactionModuleCache<'alloc, 'txn, P>>,
    pub(super) main: FunctionRef<'alloc>,
    pub(super) modules: Vec<VerifiedModule>,
}

fn static_verify_modules(
    sender_address: &AccountAddress,
    modules: Vec<CompiledModule>,
) -> Result<Vec<VerifiedModule>, Vec<VerificationStatus>> {
    // It is possible to write this function without the expects, but that makes it very ugly.
    let mut statuses: Vec<Box<dyn Iterator<Item = VerificationStatus>>> = vec![];

    let modules_len = modules.len();

    let mut modules_out = vec![];
    for (module_idx, module) in modules.into_iter().enumerate() {
        // Make sure the module's self address matches the transaction sender. The self address is
        // where the module will actually be published. If we did not check this, the sender could
        // publish a module under anyone's account.
        //
        // For scripts this isn't a problem because they don't get published to accounts.
        let self_error = if module.address() != sender_address {
            Some(VerificationError {
                kind: IndexKind::AddressPool,
                idx: CompiledModule::IMPLEMENTED_MODULE_INDEX as usize,
                err: VMStaticViolation::ModuleAddressDoesNotMatchSender,
            })
        } else {
            None
        };

        let (module, mut errors) = match VerifiedModule::new(module) {
            Ok(module) => (Some(module), vec![]),
            Err((_, errors)) => (None, errors),
        };

        if let Some(error) = self_error {
            errors.push(error);
        }

        if errors.is_empty() {
            modules_out.push(module.expect("empty errors => module should verify"));
        } else {
            statuses.push(Box::new(errors.into_iter().map(move |error| {
                VerificationStatus::Module(module_idx as u16, error)
            })));
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
) -> Result<(VerifiedScript, Vec<VerifiedModule>), Vec<VerificationStatus>> {
    // It is possible to write this function without the expects, but that makes it very ugly.
    let mut statuses: Vec<VerificationStatus> = vec![];
    let script = match VerifiedScript::new(script) {
        Ok(script) => Some(script),
        Err((_, errors)) => {
            statuses.extend(errors.into_iter().map(VerificationStatus::Script));
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
