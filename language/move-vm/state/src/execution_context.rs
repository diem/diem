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
use move_core_types::gas_schedule::{GasAlgebra, GasCarrier, GasUnits};
use move_vm_types::{chain_state::ChainState, loaded_data::types::StructType, values::GlobalValue};
use vm::errors::VMResult;

/// An `ExecutionContext` represents mutable state that is retained in-memory between invocations of
/// the Move VM.
pub trait ExecutionContext {
    /// Returns the list of events emmitted during execution.
    fn events(&self) -> &[ContractEvent];

    /// Generates a `WriteSet` as a result of an execution.
    fn make_write_set(&mut self) -> VMResult<WriteSet>;

    /// Clears all the in-memory writes local to this execution.
    fn clear(&mut self);
}

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
}

impl<'txn> ExecutionContext for TransactionExecutionContext<'txn> {
    fn events(&self) -> &[ContractEvent] {
        &self.event_data
    }

    fn make_write_set(&mut self) -> VMResult<WriteSet> {
        self.data_view.make_write_set()
    }

    fn clear(&mut self) {
        self.data_view.clear();
        self.event_data.clear();
    }
}

impl<'txn> ChainState for TransactionExecutionContext<'txn> {
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

    fn remaining_gas(&self) -> GasUnits<GasCarrier> {
        self.gas_left
    }

    fn borrow_resource(
        &mut self,
        ap: &AccessPath,
        ty: StructType,
    ) -> VMResult<Option<&GlobalValue>> {
        let map_entry = self.data_view.load_data(ap, ty)?;
        Ok(map_entry.as_ref().map(|(_, g)| g))
    }

    fn move_resource_from(
        &mut self,
        ap: &AccessPath,
        ty: StructType,
    ) -> VMResult<Option<GlobalValue>> {
        let map_entry = self.data_view.load_data(ap, ty)?;
        // .take() means that the entry is removed from the data map -- this marks the
        // access path for deletion.
        Ok(map_entry.take().map(|(_, g)| g))
    }

    fn load_module(&self, module: &ModuleId) -> VMResult<Vec<u8>> {
        self.data_view.load_module(module)
    }

    fn publish_module(&mut self, module_id: ModuleId, module: Vec<u8>) -> VMResult<()> {
        self.data_view.publish_module(module_id, module)
    }

    fn publish_resource(&mut self, ap: &AccessPath, g: (StructType, GlobalValue)) -> VMResult<()> {
        self.data_view.publish_resource(ap, g)
    }

    fn exists_module(&self, key: &ModuleId) -> bool {
        self.data_view.exists_module(key)
    }

    fn emit_event(&mut self, event: ContractEvent) {
        self.event_data.push(event)
    }
}

pub struct SystemExecutionContext<'txn>(TransactionExecutionContext<'txn>);

impl<'txn> SystemExecutionContext<'txn> {
    pub fn new(data_cache: &'txn dyn RemoteCache, gas_left: GasUnits<GasCarrier>) -> Self {
        SystemExecutionContext(TransactionExecutionContext::new(gas_left, data_cache))
    }
}

impl<'txn> ExecutionContext for SystemExecutionContext<'txn> {
    fn events(&self) -> &[ContractEvent] {
        self.0.events()
    }

    fn make_write_set(&mut self) -> VMResult<WriteSet> {
        self.0.make_write_set()
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

impl<'txn> ChainState for SystemExecutionContext<'txn> {
    fn deduct_gas(&mut self, _amount: GasUnits<GasCarrier>) -> VMResult<()> {
        Ok(())
    }

    fn publish_resource(&mut self, ap: &AccessPath, g: (StructType, GlobalValue)) -> VMResult<()> {
        self.0.publish_resource(ap, g)
    }

    fn remaining_gas(&self) -> GasUnits<GasCarrier> {
        self.0.remaining_gas()
    }

    fn borrow_resource(
        &mut self,
        ap: &AccessPath,
        ty: StructType,
    ) -> VMResult<Option<&GlobalValue>> {
        self.0.borrow_resource(ap, ty)
    }

    fn move_resource_from(
        &mut self,
        ap: &AccessPath,
        ty: StructType,
    ) -> VMResult<Option<GlobalValue>> {
        self.0.move_resource_from(ap, ty)
    }

    fn load_module(&self, module: &ModuleId) -> VMResult<Vec<u8>> {
        self.0.load_module(module)
    }

    fn exists_module(&self, key: &ModuleId) -> bool {
        self.0.exists_module(key)
    }

    fn publish_module(&mut self, module_id: ModuleId, module: Vec<u8>) -> VMResult<()> {
        self.0.publish_module(module_id, module)
    }

    fn emit_event(&mut self, event: ContractEvent) {
        self.0.emit_event(event)
    }
}

impl<'txn> From<TransactionExecutionContext<'txn>> for SystemExecutionContext<'txn> {
    fn from(ctx: TransactionExecutionContext<'txn>) -> Self {
        Self(ctx)
    }
}
