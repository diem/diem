// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::data_cache::{RemoteCache, TransactionDataCache};
use libra_types::{
    access_path::AccessPath,
    contract_event::ContractEvent,
    language_storage::ModuleId,
    vm_error::{StatusCode, VMStatus},
    write_set::WriteSet,
};
use vm::{
    errors::*,
    gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier, GasUnits},
};
use vm_runtime_types::{
    loaded_data::struct_def::StructDef,
    value::{GlobalRef, Struct, Value},
};

/// A TransactionExecutionContext holds the mutable data that needs to be persisted from one
/// section of the transaction flow to another. Because of this, this is the _only_ data that can
/// both be mutated, and persist between interpretation instances.
pub struct TransactionExecutionContext<'txn> {
    /// Gas metering to track cost of execution.
    gas_left: GasUnits<GasCarrier>,
    /// List of events "fired" during the course of an execution.
    event_data: Vec<ContractEvent>,
    /// Data store
    data_view: TransactionDataCache<'txn>,
}

/// The transaction
impl<'txn> TransactionExecutionContext<'txn> {
    pub fn new(gas_left: GasUnits<GasCarrier>, data_cache: &'txn dyn RemoteCache) -> Self {
        Self {
            gas_left,
            event_data: Vec::new(),
            data_view: TransactionDataCache::new(data_cache),
        }
    }

    pub fn exists_module(&self, m: &ModuleId) -> bool {
        self.data_view.exists_module(m)
    }

    /// Clear all the writes local to this execution.
    pub fn clear(&mut self) {
        self.data_view.clear();
        self.event_data.clear();
    }

    /// Return the list of events emitted during execution.
    pub fn events(&self) -> &[ContractEvent] {
        &self.event_data
    }

    /// Return the gas remaining
    pub fn gas_left(&self) -> GasUnits<GasCarrier> {
        self.gas_left
    }

    /// Generate a `WriteSet` as a result of an execution.
    pub fn make_write_set(
        &mut self,
        to_be_published_modules: Vec<(ModuleId, Vec<u8>)>,
    ) -> VMResult<WriteSet> {
        self.data_view.make_write_set(to_be_published_modules)
    }
}

/// The InterpreterContext context trait speficies the mutations that are allowed to the
/// TransactionExecutionContext within the interpreter.
pub trait InterpreterContext {
    fn move_resource_to(
        &mut self,
        ap: &AccessPath,
        def: StructDef,
        resource: Struct,
    ) -> VMResult<()>;

    fn move_resource_from(&mut self, ap: &AccessPath, def: StructDef) -> VMResult<Value>;

    fn resource_exists(
        &mut self,
        ap: &AccessPath,
        def: StructDef,
    ) -> VMResult<(bool, AbstractMemorySize<GasCarrier>)>;

    fn borrow_global(&mut self, ap: &AccessPath, def: StructDef) -> VMResult<GlobalRef>;

    fn push_event(&mut self, event: ContractEvent);

    fn deduct_gas(&mut self, amount: GasUnits<GasCarrier>) -> VMResult<()>;

    fn remaining_gas(&self) -> GasUnits<GasCarrier>;
}

impl<'txn> InterpreterContext for TransactionExecutionContext<'txn> {
    fn move_resource_to(
        &mut self,
        ap: &AccessPath,
        def: StructDef,
        resource: Struct,
    ) -> VMResult<()> {
        self.data_view.move_resource_to(&ap, def, resource)
    }

    fn move_resource_from(&mut self, ap: &AccessPath, def: StructDef) -> VMResult<Value> {
        self.data_view.move_resource_from(&ap, def)
    }

    fn resource_exists(
        &mut self,
        ap: &AccessPath,
        def: StructDef,
    ) -> VMResult<(bool, AbstractMemorySize<GasCarrier>)> {
        self.data_view.resource_exists(&ap, def)
    }

    fn borrow_global(&mut self, ap: &AccessPath, def: StructDef) -> VMResult<GlobalRef> {
        self.data_view.borrow_global(&ap, def)
    }

    fn push_event(&mut self, event: ContractEvent) {
        self.event_data.push(event)
    }

    fn remaining_gas(&self) -> GasUnits<GasCarrier> {
        self.gas_left()
    }

    fn deduct_gas(&mut self, amount: GasUnits<GasCarrier>) -> VMResult<()> {
        if self
            .gas_left
            .app(&amount, |curr_gas, gas_amt| curr_gas >= gas_amt)
        {
            self.gas_left = self.gas_left.sub(amount);
            Ok(())
        } else {
            // Zero out the internal gas state
            self.gas_left = GasUnits::new(0);
            Err(VMStatus::new(StatusCode::OUT_OF_GAS))
        }
    }
}
