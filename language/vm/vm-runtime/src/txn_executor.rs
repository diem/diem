// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Processor for a single transaction.

use crate::chain_state::ChainState;
use crate::{
    chain_state::TransactionExecutionContext, counters::*, data_cache::RemoteCache,
    runtime::VMRuntime,
};
use libra_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::ModuleId,
    transaction::{TransactionArgument, TransactionOutput, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
};
use vm::{
    errors::*,
    gas_schedule::{CostTable, GasAlgebra},
    transaction_metadata::TransactionMetadata,
};
use vm_runtime_types::value::Value;

/// A struct that executes one single transaction.
pub struct TransactionExecutor<'txn> {
    interpreter_context: TransactionExecutionContext<'txn>,
    txn_data: TransactionMetadata,
    gas_schedule: &'txn CostTable,
}

impl<'txn> TransactionExecutor<'txn> {
    /// Create a new `TransactionExecutor` to execute a single transaction.
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
    pub fn create_account(&mut self, runtime: &VMRuntime, addr: AccountAddress) -> VMResult<()> {
        runtime.create_account(
            &mut self.interpreter_context,
            &self.txn_data,
            &self.gas_schedule,
            addr,
        )
    }

    /// Execute a function.
    /// `module` is an identifier for the name the module is stored in. `function_name` is the name
    /// of the function. If such function is found, the VM will execute this function with arguments
    /// `args`. The return value will be placed on the top of the value stack and abort if an error
    /// occurs.
    pub fn execute_function(
        &mut self,
        runtime: &VMRuntime,
        module: &ModuleId,
        function_name: &IdentStr,
        args: Vec<Value>,
    ) -> VMResult<()> {
        runtime.execute_function(
            &mut self.interpreter_context,
            &self.txn_data,
            &self.gas_schedule,
            module,
            function_name,
            args,
        )
    }

    #[allow(non_snake_case)]
    pub fn publish_module_FOR_GENESIS_ONLY(
        &mut self,
        module_id: ModuleId,
        module: Vec<u8>,
    ) -> VMResult<()> {
        self.interpreter_context.publish_module(module_id, module)
    }

    /// Execute a function with the sender set to `sender`, restoring the original sender afterward.
    /// This should only be used in the logic for generating the genesis block.
    #[allow(non_snake_case)]
    pub fn execute_function_with_sender_FOR_GENESIS_ONLY(
        &mut self,
        runtime: &VMRuntime,
        address: AccountAddress,
        module: &ModuleId,
        function_name: &IdentStr,
        args: Vec<Value>,
    ) -> VMResult<()> {
        let old_sender = self.txn_data.sender;
        self.txn_data.sender = address;
        let res = self.execute_function(runtime, module, function_name, args);
        self.txn_data.sender = old_sender;
        res
    }

    /// Produce a write set at the end of a transaction. This will clear all the local states in
    /// the TransactionProcessor and turn them into a writeset.
    pub fn make_write_set(&mut self, result: VMResult<()>) -> VMResult<TransactionOutput> {
        // This should only be used for bookkeeping. The gas is already deducted from the sender's
        // account in the account module's epilogue.
        let gas_used: u64 = self
            .txn_data
            .max_gas_amount()
            .sub(self.interpreter_context.gas_left())
            .mul(self.txn_data.gas_unit_price())
            .get();
        let write_set = self.interpreter_context.make_write_set()?;

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

/// Convert the transaction arguments into move values.
pub fn convert_txn_args(args: Vec<TransactionArgument>) -> Vec<Value> {
    args.into_iter()
        .map(|arg| match arg {
            TransactionArgument::U64(i) => Value::u64(i),
            TransactionArgument::Address(a) => Value::address(a),
            TransactionArgument::Bool(b) => Value::bool(b),
            TransactionArgument::ByteArray(b) => Value::byte_array(b),
        })
        .collect()
}
