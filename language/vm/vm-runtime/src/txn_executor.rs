// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Processor for a single transaction.

use crate::{
    code_cache::{
        module_adapter::ModuleFetcherImpl,
        module_cache::{BlockModuleCache, ModuleCache, VMModuleCache},
    },
    counters::*,
    data_cache::{BlockDataCache, RemoteCache},
    execution_context::TransactionExecutionContext,
    gas_meter::load_gas_schedule,
    interpreter::Interpreter,
    loaded_data::function::FunctionRef,
};
use bytecode_verifier::VerifiedModule;
use libra_state_view::StateView;
use libra_types::{
    account_address::AccountAddress,
    account_config,
    identifier::{IdentStr, Identifier},
    language_storage::ModuleId,
    transaction::{TransactionArgument, TransactionOutput, TransactionStatus},
    vm_error::{StatusCode, StatusType, VMStatus},
    write_set::WriteSet,
};
use std::marker::PhantomData;
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

    /// The ModuleId for the transaction fee distribution module
    pub static ref TRANSACTION_FEE_DISTRIBUTION_MODULE: ModuleId =
        { ModuleId::new(account_config::core_code_address(), Identifier::new("TransactionFeeDistribution").unwrap()) };
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
pub struct TransactionExecutor<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    interpreter_context: TransactionExecutionContext<'txn>,
    module_cache: P,
    txn_data: TransactionMetadata,
    gas_schedule: &'txn CostTable,
    phantom: PhantomData<&'alloc ()>,
}

impl<'alloc, 'txn, P> TransactionExecutor<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    /// Create a new `TransactionExecutor` to execute a single transaction. `module_cache` is the
    /// cache that stores the modules previously read from the blockchain. `data_cache` is the cache
    /// that holds read-only connection to the state store as well as the changes made by previous
    /// transactions within the same block.
    pub fn new(
        module_cache: P,
        gas_schedule: &'txn CostTable,
        data_cache: &'txn dyn RemoteCache,
        txn_data: TransactionMetadata,
    ) -> Self {
        let interpreter_context =
            TransactionExecutionContext::new(txn_data.max_gas_amount(), data_cache);
        TransactionExecutor {
            interpreter_context,
            module_cache,
            txn_data,
            gas_schedule,
            phantom: PhantomData,
        }
    }

    /// Returns the module cache for this executor.
    pub fn module_cache(&self) -> &P {
        &self.module_cache
    }

    /// Create an account on the blockchain by calling into `CREATE_ACCOUNT_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    pub fn create_account(&mut self, addr: AccountAddress) -> VMResult<()> {
        Interpreter::create_account_entry(
            &mut self.interpreter_context,
            &self.module_cache,
            &self.txn_data,
            &self.gas_schedule,
            addr,
        )
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    pub(crate) fn run_prologue(&mut self) -> VMResult<()> {
        record_stats! {time_hist | TXN_PROLOGUE_TIME_TAKEN | {
            Interpreter::execute_function(
                &mut self.interpreter_context,
                &self.module_cache,
                &self.txn_data,
                &CostTable::zero(),
                &ACCOUNT_MODULE,
                &PROLOGUE_NAME,
                vec![],
                )?;
            }
        };
        Ok(())
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_epilogue(&mut self) -> VMResult<()> {
        record_stats! {time_hist | TXN_EPILOGUE_TIME_TAKEN | {
            Interpreter::execute_function(
                &mut self.interpreter_context,
                &self.module_cache,
                &self.txn_data,
                &CostTable::zero(),
                &ACCOUNT_MODULE,
                &EPILOGUE_NAME,
                vec![],
                )?;
            }
        }
        Ok(())
    }

    /// Generate the TransactionOutput on failure. There can be two possibilities:
    /// 1. The transaction encounters some runtime error, such as out of gas, arithmetic overflow,
    /// etc. In this scenario, we are going to keep this transaction and charge proper gas to the
    /// sender. 2. The transaction encounters VM invariant violation error type which indicates some
    /// properties should have been guaranteed failed. Such transaction should be discarded for
    /// sanity but this implies a bug in the VM that we should take care of.
    pub(crate) fn failed_transaction_cleanup(&mut self, result: VMResult<()>) -> TransactionOutput {
        // Discard all the local writes, restart execution from a clean state.
        self.clear();
        match self.run_epilogue() {
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
        to_be_published_modules: Vec<(ModuleId, Vec<u8>)>,
    ) -> TransactionOutput {
        // First run the epilogue
        match self.run_epilogue() {
            // If epilogue runs successfully, try to emit the writeset.
            Ok(_) => match self.make_write_set(to_be_published_modules, Ok(())) {
                // This step could fail if the program has dangling global reference
                Ok(trans_out) => trans_out,
                // In case of failure, run the cleanup code.
                Err(err) => self.failed_transaction_cleanup(Err(err)),
            },
            // If the sender depleted its balance and can't pay for the gas, run the cleanup code.
            Err(err) => match err.status_type() {
                StatusType::InvariantViolation => error_output(err),
                _ => self.failed_transaction_cleanup(Err(err)),
            },
        }
    }

    /// Entrypoint into the interpreter. All external calls need to be routed through this
    /// function.
    pub(crate) fn interpeter_entrypoint(
        &mut self,
        func: FunctionRef<'txn>,
        args: Vec<TransactionArgument>,
    ) -> VMResult<()> {
        Interpreter::entrypoint(
            &mut self.interpreter_context,
            &self.module_cache,
            &self.txn_data,
            &self.gas_schedule,
            func,
            convert_txn_args(args),
        )
    }

    /// Execute a function.
    /// `module` is an identifier for the name the module is stored in. `function_name` is the name
    /// of the function. If such function is found, the VM will execute this function with arguments
    /// `args`. The return value will be placed on the top of the value stack and abort if an error
    /// occurs.
    pub fn execute_function(
        &mut self,
        module: &ModuleId,
        function_name: &IdentStr,
        args: Vec<Value>,
    ) -> VMResult<()> {
        Interpreter::execute_function(
            &mut self.interpreter_context,
            &self.module_cache,
            &self.txn_data,
            &self.gas_schedule,
            module,
            function_name,
            args,
        )
    }

    /// Execute a function with the sender set to `sender`, restoring the original sender afterward.
    /// This should only be used in the logic for generating the genesis block.
    #[allow(non_snake_case)]
    pub fn execute_function_with_sender_FOR_GENESIS_ONLY(
        &mut self,
        address: AccountAddress,
        module: &ModuleId,
        function_name: &IdentStr,
        args: Vec<Value>,
    ) -> VMResult<()> {
        let old_sender = self.txn_data.sender;
        self.txn_data.sender = address;
        let res = self.execute_function(module, function_name, args);
        self.txn_data.sender = old_sender;
        res
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

    pub fn exists_module(&self, m: &ModuleId) -> bool {
        self.interpreter_context.exists_module(m)
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
        let vm_module_cache = VMModuleCache::new(&arena);
        let code_cache =
            BlockModuleCache::new(&vm_module_cache, ModuleFetcherImpl::new(state_view));
        code_cache.cache_module(module.clone());
        let data_cache = BlockDataCache::new(state_view);
        let gas_schedule = load_gas_schedule(&code_cache, &data_cache)?;
        let mut txn_executor = TransactionExecutor::new(
            code_cache,
            &gas_schedule,
            &data_cache,
            TransactionMetadata::default(),
        );
        txn_executor.execute_function(&module_id, &entry_name, convert_txn_args(args))
    }
}
