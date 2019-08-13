use crate::{
    code_cache::module_cache::ModuleCache,
    process_txn::verify::{VerifiedTransaction, VerifiedTransactionState},
};
use logger::prelude::*;
use types::{
    transaction::{TransactionOutput, TransactionPayload, TransactionStatus},
    vm_error::{ExecutionStatus, VMStatus},
    write_set::WriteSet,
};
use vm::{
    access::ModuleAccess,
    errors::{Location, VMErrorKind, VMRuntimeError},
};

/// Represents a transaction that has been executed.
pub struct ExecutedTransaction {
    output: TransactionOutput,
}

impl ExecutedTransaction {
    /// Creates a new instance by executing this transaction.
    pub fn new<'alloc, 'txn, P>(verified_txn: VerifiedTransaction<'alloc, 'txn, P>) -> Self
    where
        'alloc: 'txn,
        P: ModuleCache<'alloc>,
    {
        let output = execute(verified_txn);
        Self { output }
    }

    /// Returns the `TransactionOutput` for this transaction.
    pub fn into_output(self) -> TransactionOutput {
        self.output
    }
}

fn execute<'alloc, 'txn, P>(
    mut verified_txn: VerifiedTransaction<'alloc, 'txn, P>,
) -> TransactionOutput
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    let txn_state = verified_txn.take_state();

    match verified_txn
        .into_inner()
        .into_raw_transaction()
        .into_payload()
    {
        TransactionPayload::Program(program) => {
            let VerifiedTransactionState {
                mut txn_executor,
                main,
                modules,
            } = txn_state.expect("program-based transactions should always have associated state");

            let (_, args, module_bytes) = program.into_inner();

            // Add modules to the cache and prepare for publishing.
            let mut publish_modules = vec![];
            for (module, raw_bytes) in modules.into_iter().zip(module_bytes) {
                let module_id = module.self_id();

                // Make sure that there is not already a module with this name published
                // under the transaction sender's account.
                // Note: although this reads from the "module cache", `get_loaded_module`
                // will read through the cache to fetch the module from the global storage
                // if it is not already cached.
                match txn_executor.module_cache().get_loaded_module(&module_id) {
                    Ok(Ok(None)) => (), // No module with this name exists. safe to publish one
                    Ok(Ok(Some(_))) | Ok(Err(_)) => {
                        // A module with this name already exists (the error case is when the module
                        // couldn't be verified, but it still exists so we should fail similarly).
                        // It is not safe to publish another one; it would clobber the old module.
                        // This would break code that links against the module and make published
                        // resources from the old module inaccessible (or worse, accessible and not
                        // typesafe).
                        //
                        // We are currently developing a versioning scheme for safe updates of
                        // modules and resources.
                        warn!("[VM] VM error duplicate module {:?}", module_id);
                        return txn_executor.failed_transaction_cleanup(Ok(Err(VMRuntimeError {
                            loc: Location::default(),
                            err: VMErrorKind::DuplicateModuleName,
                        })));
                    }
                    Err(err) => {
                        error!(
                            "[VM] VM internal error while checking for duplicate module {:?}: {:?}",
                            module_id, err
                        );
                        return ExecutedTransaction::discard_error_output(&err);
                    }
                }

                txn_executor.module_cache().cache_module(module);
                publish_modules.push((module_id, raw_bytes));
            }

            // Set up main.
            txn_executor.setup_main_args(args);

            // Run main.
            match txn_executor.execute_function_impl(main) {
                Ok(Ok(_)) => txn_executor.transaction_cleanup(publish_modules),
                Ok(Err(err)) => {
                    warn!("[VM] User error running script: {:?}", err);
                    txn_executor.failed_transaction_cleanup(Ok(Err(err)))
                }
                Err(err) => {
                    error!("[VM] VM error running script: {:?}", err);
                    ExecutedTransaction::discard_error_output(&err)
                }
            }
        }
        // WriteSet transaction. Just proceed and use the writeset as output.
        TransactionPayload::WriteSet(write_set) => TransactionOutput::new(
            write_set,
            vec![],
            0,
            VMStatus::Execution(ExecutionStatus::Executed).into(),
        ),
    }
}

impl ExecutedTransaction {
    #[inline]
    pub(crate) fn discard_error_output(err: impl Into<VMStatus>) -> TransactionOutput {
        // Since this transaction will be discarded, no writeset will be included.
        TransactionOutput::new(
            WriteSet::default(),
            vec![],
            0,
            TransactionStatus::Discard(err.into()),
        )
    }
}
