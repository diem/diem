// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::*,
    data_cache::{RemoteCache, TransactionDataCache},
};
use libra_types::{
    access_path::AccessPath,
    contract_event::ContractEvent,
    language_storage::ModuleId,
    transaction::{TransactionOutput, TransactionStatus},
    vm_error::{StatusCode, VMStatus},
    write_set::WriteSet,
};
use move_vm_types::{loaded_data::struct_def::StructDef, values::GlobalValue};
use vm::{
    errors::VMResult,
    gas_schedule::{GasAlgebra, GasCarrier, GasUnits},
    transaction_metadata::TransactionMetadata,
};

/// Trait that describes what Move bytecode runtime expects from the Libra blockchain.
pub trait ChainState {
    // ---
    // Gas operations
    // ---

    fn deduct_gas(&mut self, amount: GasUnits<GasCarrier>) -> VMResult<()>;
    fn remaining_gas(&self) -> GasUnits<GasCarrier>;

    // ---
    // StateStore operations
    // ---

    // An alternative for these APIs might look like:
    //
    //   fn read_data(&self, ap: &AccessPath) -> VMResult<Vec<u8>>;
    //   fn write_data(&mut self, ap: &AccessPath, data: Vec<u8>) -> VMResult<()>;
    //
    // However, this would make the Move VM responsible for deserialization -- in particular,
    // caching deserialized results leads to a big performance improvement. But this directly
    // conflicts with the goal of the Move VM to be as stateless as possible. Hence the burden of
    // deserialization (and caching) is placed on the implementer of this trait.

    /// Get the serialized format of a `CompiledModule` from chain given a `ModuleId`.
    fn load_module(&self, module: &ModuleId) -> VMResult<Vec<u8>>;

    /// Get a reference to a resource stored on chain.
    fn borrow_resource(
        &mut self,
        ap: &AccessPath,
        def: StructDef,
    ) -> VMResult<Option<&GlobalValue>>;

    /// Transfer ownership of a resource stored on chain to the VM.
    fn move_resource_from(
        &mut self,
        ap: &AccessPath,
        def: StructDef,
    ) -> VMResult<Option<GlobalValue>>;

    /// Publish a module to be stored on chain.
    fn publish_module(&mut self, module_id: ModuleId, module: Vec<u8>) -> VMResult<()>;

    /// Publish a resource to be stored on chain.
    fn publish_resource(&mut self, ap: &AccessPath, g: (StructDef, GlobalValue)) -> VMResult<()>;

    /// Check if this module exists on chain.
    // TODO: Can we get rid of this api with the loader refactor?
    fn exists_module(&self, key: &ModuleId) -> bool;

    // ---
    // EventStore operations
    // ---

    /// Emit an event to the EventStore
    fn emit_event(&mut self, event: ContractEvent);
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
    pub fn make_write_set(&mut self) -> VMResult<WriteSet> {
        self.data_view.make_write_set()
    }

    pub fn get_transaction_output(
        &mut self,
        txn_data: &TransactionMetadata,
        status: VMStatus,
    ) -> VMResult<TransactionOutput> {
        let gas_used: u64 = txn_data
            .max_gas_amount()
            .sub(self.gas_left())
            .mul(txn_data.gas_unit_price())
            .get();
        let write_set = self.make_write_set()?;
        record_stats!(observe | TXN_TOTAL_GAS_USAGE | gas_used);
        Ok(TransactionOutput::new(
            write_set,
            self.events().to_vec(),
            gas_used,
            TransactionStatus::Keep(status),
        ))
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
        def: StructDef,
    ) -> VMResult<Option<&GlobalValue>> {
        let map_entry = self.data_view.load_data(ap, def)?;
        Ok(map_entry.as_ref().map(|(_, g)| g))
    }

    fn move_resource_from(
        &mut self,
        ap: &AccessPath,
        def: StructDef,
    ) -> VMResult<Option<GlobalValue>> {
        let map_entry = self.data_view.load_data(ap, def)?;
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

    fn publish_resource(&mut self, ap: &AccessPath, g: (StructDef, GlobalValue)) -> VMResult<()> {
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

    /// Clear all the writes local to this execution.
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Return the list of events emitted during execution.
    pub fn events(&self) -> &[ContractEvent] {
        self.0.events()
    }

    /// Generate a `WriteSet` as a result of an execution.
    pub fn make_write_set(&mut self) -> VMResult<WriteSet> {
        self.0.make_write_set()
    }

    pub fn get_transaction_output(
        &mut self,
        txn_data: &TransactionMetadata,
        status: VMStatus,
    ) -> VMResult<TransactionOutput> {
        self.0.get_transaction_output(txn_data, status)
    }
}

impl<'txn> ChainState for SystemExecutionContext<'txn> {
    fn deduct_gas(&mut self, _amount: GasUnits<GasCarrier>) -> VMResult<()> {
        Ok(())
    }

    fn publish_resource(&mut self, ap: &AccessPath, g: (StructDef, GlobalValue)) -> VMResult<()> {
        self.0.publish_resource(ap, g)
    }

    fn remaining_gas(&self) -> GasUnits<GasCarrier> {
        self.0.gas_left()
    }

    fn borrow_resource(
        &mut self,
        ap: &AccessPath,
        def: StructDef,
    ) -> VMResult<Option<&GlobalValue>> {
        self.0.borrow_resource(ap, def)
    }

    fn move_resource_from(
        &mut self,
        ap: &AccessPath,
        def: StructDef,
    ) -> VMResult<Option<GlobalValue>> {
        self.0.move_resource_from(ap, def)
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
