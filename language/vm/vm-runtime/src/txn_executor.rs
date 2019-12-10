// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Processor for a single transaction.

use crate::{
    chain_state::TransactionExecutionContext,
    counters::*,
    data_cache::{BlockDataCache, RemoteCache},
    runtime::VMRuntime,
};
use bytecode_verifier::VerifiedModule;
use libra_config::config::{VMConfig, VMPublishingOption};
use libra_state_view::StateView;
use libra_types::{
    account_address::AccountAddress,
    account_config,
    byte_array::ByteArray,
    identifier::{IdentStr, Identifier},
    language_storage::ModuleId,
    transaction::{TransactionArgument, TransactionOutput, TransactionStatus},
    vm_error::{StatusCode, StatusType, VMStatus},
    write_set::WriteSet,
};
use vm::{
    access::ModuleAccess,
    errors::*,
    file_format::FunctionDefinitionIndex,
    gas_schedule::{CostTable, GasAlgebra},
    transaction_metadata::TransactionMetadata,
    vm_string::VMString,
};
use vm_cache_map::Arena;
use vm_runtime_types::value::Value;

pub use crate::gas_meter::GAS_SCHEDULE_MODULE;

// Metadata needed for resolving the account module.
lazy_static! {
    /// The ModuleId for the Account module
    pub static ref ACCOUNT_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("LibraAccount").unwrap()) };
    /// The ModuleId for the LibraCoin module
    pub static ref COIN_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("LibraCoin").unwrap()) };
    /// The ModuleId for the Event
    pub static ref EVENT_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("Event").unwrap()) };
    /// The ModuleId for the validator config
    pub static ref VALIDATOR_CONFIG_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("ValidatorConfig").unwrap()) };
    /// The ModuleId for the libra system module
    pub static ref LIBRA_SYSTEM_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("LibraSystem").unwrap()) };
}

// Names for special functions.
lazy_static! {
    static ref PROLOGUE_NAME: Identifier = Identifier::new("prologue").unwrap();
    static ref EPILOGUE_NAME: Identifier = Identifier::new("epilogue").unwrap();
    static ref CREATE_ACCOUNT_NAME: Identifier = Identifier::new("make").unwrap();
    static ref ACCOUNT_STRUCT_NAME: Identifier = Identifier::new("T").unwrap();
    static ref EMIT_EVENT_NAME: Identifier = Identifier::new("write_to_event_store").unwrap();
}

/// A struct that executes one single transaction.
/// 'alloc is the lifetime for the code cache, which is the argument type P here. Hence the P should
/// live as long as alloc.
/// 'txn is the lifetime of one single transaction.
pub struct TransactionExecutor<'txn> {
    interpreter_context: TransactionExecutionContext<'txn>,
    txn_data: TransactionMetadata,
    gas_schedule: &'txn CostTable,
}

impl<'txn> TransactionExecutor<'txn> {
    /// Create a new `TransactionExecutor` to execute a single transaction. `module_cache` is the
    /// cache that stores the modules previously read from the blockchain. `data_cache` is the cache
    /// that holds read-only connection to the state store as well as the changes made by previous
    /// transactions within the same block.
    pub fn new(
        gas_schedule: &'txn CostTable,
        data_cache: &'txn dyn RemoteCache,
        txn_data: TransactionMetadata,
    ) -> Self {
        let interpreter_context =
            TransactionExecutionContext::new(txn_data.max_gas_amount(), data_cache);
        TransactionExecutor {
            interpreter_context,
            txn_data,
            gas_schedule,
        }
    }

    /// Create an account on the blockchain by calling into `CREATE_ACCOUNT_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    pub fn create_account(
        &mut self,
        runtime: &VMRuntime,
        state_view: &dyn StateView,
        addr: AccountAddress,
    ) -> VMResult<()> {
        runtime.create_account(
            state_view,
            &mut self.interpreter_context,
            &self.txn_data,
            &self.gas_schedule,
            addr,
        )
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    pub(crate) fn run_prologue(
        &mut self,
        runtime: &VMRuntime,
        state_view: &dyn StateView,
    ) -> VMResult<()> {
        let txn_sequence_number = self.txn_data.sequence_number();
        let txn_public_key = self.txn_data.public_key().to_bytes().to_vec();
        let txn_gas_price = self.txn_data.gas_unit_price().get();
        let txn_max_gas_units = self.txn_data.max_gas_amount().get();
        record_stats! {time_hist | TXN_PROLOGUE_TIME_TAKEN | {
            runtime.execute_function(
                state_view,
                &mut self.interpreter_context,
                &self.txn_data,
                &CostTable::zero(),
                &ACCOUNT_MODULE,
                &PROLOGUE_NAME,
                vec![
                    Value::u64(txn_sequence_number),
                    Value::byte_array(ByteArray::new(txn_public_key)),
                    Value::u64(txn_gas_price),
                    Value::u64(txn_max_gas_units),
                ],
                )?;
            }
        };
        Ok(())
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_epilogue(&mut self, runtime: &VMRuntime, state_view: &dyn StateView) -> VMResult<()> {
        let txn_sequence_number = self.txn_data.sequence_number();
        let txn_gas_price = self.txn_data.gas_unit_price().get();
        let txn_max_gas_units = self.txn_data.max_gas_amount().get();
        let gas_remaining = self.interpreter_context.gas_left().get();
        record_stats! {time_hist | TXN_EPILOGUE_TIME_TAKEN | {
            runtime.execute_function(
                state_view,
                &mut self.interpreter_context,
                &self.txn_data,
                &CostTable::zero(),
                &ACCOUNT_MODULE,
                &EPILOGUE_NAME,
                vec![
                    Value::u64(txn_sequence_number),
                    Value::u64(txn_gas_price),
                    Value::u64(txn_max_gas_units),
                    Value::u64(gas_remaining)
                ],
                )?;
            }
        }
        Ok(())
    }

    /// Execute a function.
    /// `module` is an identifier for the name the module is stored in. `function_name` is the name
    /// of the function. If such function is found, the VM will execute this function with arguments
    /// `args`. The return value will be placed on the top of the value stack and abort if an error
    /// occurs.
    pub fn execute_function(
        &mut self,
        runtime: &VMRuntime,
        state_view: &dyn StateView,
        module: &ModuleId,
        function_name: &IdentStr,
        args: Vec<Value>,
    ) -> VMResult<()> {
        runtime.execute_function(
            state_view,
            &mut self.interpreter_context,
            &self.txn_data,
            &self.gas_schedule,
            module,
            function_name,
            args,
        )
    }

    /// Entrypoint into the interpreter. All external calls need to be routed through this
    /// function.
    pub(crate) fn execute_script(
        &mut self,
        runtime: &VMRuntime,
        state_view: &dyn StateView,
        script: Vec<u8>,
        args: Vec<TransactionArgument>,
    ) -> VMResult<()> {
        runtime.execute_script(
            state_view,
            &mut self.interpreter_context,
            &self.txn_data,
            &self.gas_schedule,
            script,
            convert_txn_args(args),
        )
    }

    /// Verifies that a `CompiledModule` is valid and returns the `ModuleId` if successful.
    pub(crate) fn publish_module(
        &mut self,
        module: &[u8],
        runtime: &VMRuntime,
        _state_view: &dyn StateView,
    ) -> VMResult<ModuleId> {
        runtime.publish_module(module, &mut self.interpreter_context, &self.txn_data)
    }

    /// Execute a function with the sender set to `sender`, restoring the original sender afterward.
    /// This should only be used in the logic for generating the genesis block.
    #[allow(non_snake_case)]
    pub fn execute_function_with_sender_FOR_GENESIS_ONLY(
        &mut self,
        runtime: &VMRuntime,
        state_view: &dyn StateView,
        address: AccountAddress,
        module: &ModuleId,
        function_name: &IdentStr,
        args: Vec<Value>,
    ) -> VMResult<()> {
        let old_sender = self.txn_data.sender;
        self.txn_data.sender = address;
        let res = self.execute_function(runtime, state_view, module, function_name, args);
        self.txn_data.sender = old_sender;
        res
    }

    /// Generate the TransactionOutput on failure. There can be two possibilities:
    /// 1. The transaction encounters some runtime error, such as out of gas, arithmetic overflow,
    /// etc. In this scenario, we are going to keep this transaction and charge proper gas to the
    /// sender. 2. The transaction encounters VM invariant violation error type which indicates some
    /// properties should have been guaranteed failed. Such transaction should be discarded for
    /// sanity but this implies a bug in the VM that we should take care of.
    pub(crate) fn failed_transaction_cleanup(
        &mut self,
        runtime: &VMRuntime,
        state_view: &dyn StateView,
        result: VMResult<()>,
    ) -> TransactionOutput {
        // Discard all the local writes, restart execution from a clean state.
        self.clear();
        match self.run_epilogue(runtime, state_view) {
            Ok(_) => self
                .make_write_set(vec![], result)
                .unwrap_or_else(error_output),
            // Running epilogue shouldn't fail here as we've already checked for enough balance in
            // the prologue
            Err(err) => error_output(err),
        }
    }

    /// Clear all the writes local to this transaction.
    fn clear(&mut self) {
        self.interpreter_context.clear();
    }

    /// Generate the TransactionOutput for a successful transaction
    pub(crate) fn transaction_cleanup(
        &mut self,
        runtime: &VMRuntime,
        state_view: &dyn StateView,
        to_be_published_modules: Vec<(ModuleId, Vec<u8>)>,
    ) -> TransactionOutput {
        // First run the epilogue
        match self.run_epilogue(runtime, state_view) {
            // If epilogue runs successfully, try to emit the writeset.
            Ok(_) => match self.make_write_set(to_be_published_modules, Ok(())) {
                // This step could fail if the program has dangling global reference
                Ok(trans_out) => trans_out,
                // In case of failure, run the cleanup code.
                Err(err) => self.failed_transaction_cleanup(runtime, state_view, Err(err)),
            },
            // If the sender depleted its balance and can't pay for the gas, run the cleanup code.
            Err(err) => match err.status_type() {
                StatusType::InvariantViolation => error_output(err),
                _ => self.failed_transaction_cleanup(runtime, state_view, Err(err)),
            },
        }
    }

    /// Produce a write set at the end of a transaction. This will clear all the local states in
    /// the TransactionProcessor and turn them into a writeset.
    pub fn make_write_set(
        &mut self,
        to_be_published_modules: Vec<(ModuleId, Vec<u8>)>,
        result: VMResult<()>,
    ) -> VMResult<TransactionOutput> {
        // This should only be used for bookkeeping. The gas is already deducted from the sender's
        // account in the account module's epilogue.
        let gas_used: u64 = self
            .txn_data
            .max_gas_amount()
            .sub(self.interpreter_context.gas_left())
            .mul(self.txn_data.gas_unit_price())
            .get();
        let write_set = self
            .interpreter_context
            .make_write_set(to_be_published_modules)?;

        record_stats!(observe | TXN_TOTAL_GAS_USAGE | gas_used);

        Ok(TransactionOutput::new(
            write_set,
            self.interpreter_context.events().to_vec(),
            gas_used,
            match result {
                Ok(()) => TransactionStatus::from(VMStatus::new(StatusCode::EXECUTED)),
                Err(err) => TransactionStatus::from(err),
            },
        ))
    }
}

#[inline]
fn error_output(err: VMStatus) -> TransactionOutput {
    // Since this transaction will be discarded, no writeset will be included.
    TransactionOutput::new(
        WriteSet::default(),
        vec![],
        0,
        TransactionStatus::Discard(err),
    )
}

/// Convert the transaction arguments into move values.
pub fn convert_txn_args(args: Vec<TransactionArgument>) -> Vec<Value> {
    args.into_iter()
        .map(|arg| match arg {
            TransactionArgument::U64(i) => Value::u64(i),
            TransactionArgument::Address(a) => Value::address(a),
            TransactionArgument::Bool(b) => Value::bool(b),
            TransactionArgument::ByteArray(b) => Value::byte_array(b),
            TransactionArgument::String(s) => Value::string(VMString::new(s)),
        })
        .collect()
}

/// Execute the first function in a module
pub fn execute_function_in_module(
    state_view: &dyn StateView,
    module: VerifiedModule,
    idx: FunctionDefinitionIndex,
    args: Vec<TransactionArgument>,
) -> VMResult<()> {
    let module_id = module.as_inner().self_id();
    let entry_name = {
        let entry_func_idx = module.function_def_at(idx).function;
        let entry_name_idx = module.function_handle_at(entry_func_idx).name;
        module.identifier_at(entry_name_idx)
    };
    {
        let arena = Arena::new();
        let config = VMConfig {
            publishing_options: VMPublishingOption::Open,
        };
        let mut runtime = VMRuntime::new(&arena, &config);
        runtime.cache_module(module.clone());

        let mut data_cache = BlockDataCache::new(state_view);
        let gas_schedule = runtime.load_gas_schedule(&mut data_cache, state_view)?;
        let mut txn_executor =
            TransactionExecutor::new(&gas_schedule, &data_cache, TransactionMetadata::default());
        txn_executor.execute_function(
            &runtime,
            state_view,
            &module_id,
            &entry_name,
            convert_txn_args(args),
        )
    }
}
