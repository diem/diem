use crate::{
    code_cache::{
        module_cache::{ModuleCache, TransactionModuleCache},
        script_cache::ScriptCache,
    },
    process_txn::{execute::ExecutedTransaction, validate::ValidatedTransaction},
    txn_executor::TransactionExecutor,
};
use bytecode_verifier::{verify_module, verify_script};
use logger::prelude::*;
use types::{
    account_address::AccountAddress,
    transaction::{Program, SignedTransaction, TransactionArgument, TransactionPayload},
    vm_error::{VMStatus, VMVerificationError, VMVerificationStatus},
};
use vm::{
    access::BaseAccess,
    errors::{VMStaticViolation, VerificationError, VerificationStatus},
    file_format::{CompiledModule, CompiledScript, SignatureToken},
    IndexKind,
};

/// Represents a transaction which has been validated and for which the program has been run
/// through the bytecode verifier.
pub struct VerifiedTransaction<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    txn: SignedTransaction,
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
    ) -> Result<Self, VMStatus> {
        let txn_state = validated_txn.take_state();
        let txn = validated_txn.as_inner();
        let txn_state = match txn.payload() {
            TransactionPayload::Program(program) => {
                let txn_state = txn_state
                    .expect("program-based transactions should always have associated state");

                let (script, modules) = Self::verify_program(&txn.sender(), program)?;

                Some(VerifiedTransactionState {
                    txn_executor: txn_state.txn_executor,
                    script,
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
    ) -> Result<(CompiledScript, Vec<CompiledModule>), VMStatus> {
        // Ensure modules and scripts deserialize correctly.
        let script = match CompiledScript::deserialize(&program.code()) {
            Ok(script) => script,
            Err(ref err) => {
                error!("[VM] script deserialization failed");
                return Err(err.into());
            }
        };
        if !verify_actuals(&script, program.args()) {
            error!("[VM] actual type mismatch");
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
                error!("[VM] module deserialization failed");
                return Err(err.into());
            }
        };

        // Run the script and module through the bytecode verifier.
        let (script, modules, statuses) = static_verify_program(sender_address, script, modules);
        if !statuses.is_empty() {
            error!("[VM] bytecode verifier returned errors");
            return Err(statuses.iter().collect());
        }

        Ok((script, modules))
    }

    /// Executes this transaction.
    pub fn execute(self, script_cache: &'txn ScriptCache<'alloc>) -> ExecutedTransaction {
        ExecutedTransaction::new(self, script_cache)
    }

    /// Returns the state stored in the transaction, if any.
    pub(super) fn take_state(&mut self) -> Option<VerifiedTransactionState<'alloc, 'txn, P>> {
        self.txn_state.take()
    }

    /// Returns a reference to the `SignedTransaction` within.
    #[allow(dead_code)]
    pub fn as_inner(&self) -> &SignedTransaction {
        &self.txn
    }

    /// Consumes `self` and returns the `SignedTransaction` within.
    pub fn into_inner(self) -> SignedTransaction {
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
    pub(super) script: CompiledScript,
    pub(super) modules: Vec<CompiledModule>,
}

/// Run static checks on a program directly. Provided as an alternative API for tests.
pub fn static_verify_program(
    sender_address: &AccountAddress,
    script: CompiledScript,
    modules: Vec<CompiledModule>,
) -> (CompiledScript, Vec<CompiledModule>, Vec<VerificationStatus>) {
    let mut statuses: Vec<Box<dyn Iterator<Item = VerificationStatus>>> = vec![];
    let (script, errors) = verify_script(script);
    statuses.push(Box::new(errors.into_iter().map(VerificationStatus::Script)));

    let mut modules_out = vec![];
    for (module_idx, module) in modules.into_iter().enumerate() {
        let (module, errors) = verify_module(module);

        // Make sure the module's self address matches the transaction sender. The self address is
        // where the module will actually be published. If we did not check this, the sender could
        // publish a module under anyone's account.
        //
        // For scripts this isn't a problem because they don't get published to accounts.
        let address_mismatch = if module.address() != sender_address {
            Some(VerificationError {
                kind: IndexKind::AddressPool,
                idx: CompiledModule::IMPLEMENTED_MODULE_INDEX as usize,
                err: VMStaticViolation::ModuleAddressDoesNotMatchSender,
            })
        } else {
            None
        };

        statuses.push(Box::new(
            errors
                .into_iter()
                .chain(address_mismatch)
                .map(move |err| VerificationStatus::Module(module_idx as u16, err)),
        ));

        modules_out.push(module);
    }

    // TODO: Cross-module verification. This will need some way of exposing module
    // dependencies to the bytecode verifier.

    let statuses = statuses.into_iter().flatten().collect();
    (script, modules_out, statuses)
}

/// Verify if the transaction arguments match the type signature of the main function.
fn verify_actuals(script: &CompiledScript, args: &[TransactionArgument]) -> bool {
    let fh = script.function_handle_at(script.main.function);
    let sig = script.function_signature_at(fh.signature);
    if sig.arg_types.len() != args.len() {
        return false;
    }
    for (ty, arg) in sig.arg_types.iter().zip(args.iter()) {
        match (ty, arg) {
            (SignatureToken::U64, TransactionArgument::U64(_)) => (),
            (SignatureToken::Address, TransactionArgument::Address(_)) => (),
            (SignatureToken::ByteArray, TransactionArgument::ByteArray(_)) => (),
            (SignatureToken::String, TransactionArgument::String(_)) => (),
            _ => return false,
        }
    }
    true
}
