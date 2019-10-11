use crate::{
    code_cache::module_cache::ModuleCache,
    process_txn::verify::{VerTxn, VerifiedTransaction, VerifiedTransactionState},
};
use libra_types::{
    transaction::{TransactionOutput, TransactionPayload, TransactionStatus},
    vm_error::{StatusCode, StatusType, VMStatus},
    write_set::WriteSet,
};
use logger::prelude::*;
use vm::{
    access::ModuleAccess,
    errors::{vm_error, Location},
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
                verified_txn,
            } = txn_state.expect("program-based transactions should always have associated state");
            let (main, modules) = match verified_txn {
                VerTxn::Program(ver_program) => (ver_program.main, ver_program.modules),
                _ => unreachable!("TransactionPayload::Program expects VerTxn::Program"),
            };

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
                    Ok(None) => (), // No module with this name exists. safe to publish one
                    Err(ref err) if err.is(StatusType::InvariantViolation) => {
                        error!(
                            "[VM] VM internal error while checking for duplicate module {:?}: {:?}",
                            module_id, err
                        );
                        return ExecutedTransaction::discard_error_output(err.clone());
                    }
                    Ok(Some(_)) | Err(_) => {
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
                        return txn_executor.failed_transaction_cleanup(Err(vm_error(
                            Location::default(),
                            StatusCode::DUPLICATE_MODULE_NAME,
                        )));
                    }
                }

                txn_executor.module_cache().cache_module(module);

                // `publish_modules` is initally empty, a single element is pushed per loop
                // iteration and the number of iterations is bound to the max size
                // of `modules`
                assume!(publish_modules.len() < usize::max_value());
                publish_modules.push((module_id, raw_bytes));
            }

            // Set up main.
            txn_executor.setup_main_args(args);

            // Run main.
            match txn_executor.interpeter_entrypoint(main) {
                Ok(_) => txn_executor.transaction_cleanup(publish_modules),
                Err(err) => match err.status_type() {
                    StatusType::InvariantViolation => {
                        error!("[VM] VM error running script: {:?}", err);
                        ExecutedTransaction::discard_error_output(err)
                    }
                    _ => {
                        warn!("[VM] User error running script: {:?}", err);
                        txn_executor.failed_transaction_cleanup(Err(err))
                    }
                },
            }
        }
        // WriteSet transaction. Just proceed and use the writeset as output.
        TransactionPayload::WriteSet(write_set) => TransactionOutput::new(
            write_set,
            vec![],
            0,
            VMStatus::new(StatusCode::EXECUTED).into(),
        ),
        TransactionPayload::Module(module) => {
            let VerifiedTransactionState {
                mut txn_executor,
                verified_txn,
            } = txn_state.expect("module transactions should always have associated state");
            let ver_module = match verified_txn {
                VerTxn::Module(ver_module) => ver_module,
                _ => unreachable!("TransactionPayload::Module expects VerTxn::Module"),
            };
            let module_id = ver_module.self_id();
            // Make sure that there is not already a module with this name published
            // under the transaction sender's account.
            // Note: although this reads from the "module cache", `get_loaded_module`
            // will read through the cache to fetch the module from the global storage
            // if it is not already cached.
            match txn_executor.module_cache().get_loaded_module(&module_id) {
                Ok(None) => (), // No module with this name exists. safe to publish one
                Err(ref err) if err.is(StatusType::InvariantViolation) => {
                    error!(
                        "[VM] VM internal error while checking for duplicate module {:?}: {:?}",
                        module_id, err
                    );
                    return ExecutedTransaction::discard_error_output(err.clone());
                }
                Ok(Some(_)) | Err(_) => {
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
                    return txn_executor.failed_transaction_cleanup(Err(vm_error(
                        Location::default(),
                        StatusCode::DUPLICATE_MODULE_NAME,
                    )));
                }
            }
            let module_bytes = module.into_inner();
            txn_executor.transaction_cleanup(vec![(module_id, module_bytes)])
        }
        TransactionPayload::Script(script) => {
            let VerifiedTransactionState {
                mut txn_executor,
                verified_txn,
            } = txn_state.expect("script-based transactions should always have associated state");
            let main = match verified_txn {
                VerTxn::Script(func) => func,
                _ => unreachable!("TransactionPayload::Script expects VerTxn::Program"),
            };

            let (_, args) = script.into_inner();
            txn_executor.setup_main_args(args);
            match txn_executor.interpeter_entrypoint(main) {
                Ok(_) => txn_executor.transaction_cleanup(vec![]),
                Err(err) => match err.status_type() {
                    StatusType::InvariantViolation => {
                        error!("[VM] VM error running script: {:?}", err);
                        ExecutedTransaction::discard_error_output(err)
                    }
                    _ => {
                        warn!("[VM] User error running script: {:?}", err);
                        txn_executor.failed_transaction_cleanup(Err(err))
                    }
                },
            }
        }
        TransactionPayload::ChannelWriteSet(channel_payload) => TransactionOutput::new(
            channel_payload.write_set,
            vec![],
            0,
            VMStatus::new(StatusCode::EXECUTED).into(),
        ),
        TransactionPayload::ChannelScript(channel_payload) => {
            let VerifiedTransactionState {
                mut txn_executor,
                verified_txn,
            } = txn_state.expect("script-based transactions should always have associated state");
            let main = match verified_txn {
                VerTxn::Script(func) => func,
                _ => unreachable!("TransactionPayload::Script expects VerTxn::Program"),
            };

            let (_, args) = channel_payload.script.into_inner();
            txn_executor.setup_main_args(args);
            let script_output = match txn_executor.interpeter_entrypoint(main) {
                Ok(_) => txn_executor.transaction_cleanup(vec![]),
                Err(err) => match err.status_type() {
                    StatusType::InvariantViolation => {
                        error!("[VM] VM error running script: {:?}", err);
                        ExecutedTransaction::discard_error_output(err)
                    }
                    _ => {
                        warn!("[VM] User error running script: {:?}", err);
                        txn_executor.failed_transaction_cleanup(Err(err))
                    }
                },
            };
            let merged_write_set =
                WriteSet::merge(&channel_payload.write_set, script_output.write_set());
            //TODO(jole) eliminate clone
            TransactionOutput::new(
                merged_write_set,
                script_output.events().to_vec(),
                script_output.gas_used(),
                script_output.status().clone(),
            )
        }
    }
}

impl ExecutedTransaction {
    #[inline]
    pub(crate) fn discard_error_output(err: VMStatus) -> TransactionOutput {
        // Since this transaction will be discarded, no writeset will be included.
        TransactionOutput::new(
            WriteSet::default(),
            vec![],
            0,
            TransactionStatus::Discard(err),
        )
    }
}
