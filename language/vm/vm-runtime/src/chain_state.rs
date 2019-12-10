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
    errors::VMResult,
    gas_schedule::{GasAlgebra, GasCarrier, GasUnits},
};
use vm_runtime_types::{loaded_data::struct_def::StructDef, value::GlobalRef};

/// Trait that describes what Move bytecode runtime expects from the Libra blockchain.
pub trait ChainState {
    // Gas operations
    fn deduct_gas(&mut self, amount: GasUnits<GasCarrier>) -> VMResult<()>;
    fn remaining_gas(&self) -> GasUnits<GasCarrier>;

    // StateStore operations. Ideally the api should look like:
    // fn read_data(&self, ap: &AccessPath) -> VMResult<Vec<u8>>;
    // fn write_data(&mut self, ap: &AccessPath, data: Vec<u8>) -> VMResult<()>;
    // However this is not implementable due to the current implementation of MoveVM: data are
    // organized as a tree of GlobalRefs.
    /// Get a mutable reference to a resource stored on chain.
    fn load_data(&mut self, ap: &AccessPath, def: StructDef) -> VMResult<&mut GlobalRef>;
    /// Publish a resource to be stored on chain.
    fn publish_resource(&mut self, ap: &AccessPath, root: GlobalRef) -> VMResult<()>;

    /// Check if this module exists on chain.
    // TODO: Can we get rid of this api with the loader refactor?
    fn exists_module(&self, key: &ModuleId) -> bool;

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
    pub fn make_write_set(
        &mut self,
        to_be_published_modules: Vec<(ModuleId, Vec<u8>)>,
    ) -> VMResult<WriteSet> {
        self.data_view.make_write_set(to_be_published_modules)
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

    fn publish_resource(&mut self, ap: &AccessPath, root: GlobalRef) -> VMResult<()> {
        self.data_view.publish_resource(ap, root)
    }

    fn remaining_gas(&self) -> GasUnits<GasCarrier> {
        self.gas_left
    }

    fn load_data(&mut self, ap: &AccessPath, def: StructDef) -> VMResult<&mut GlobalRef> {
        self.data_view.load_data(ap, def)
    }
    fn exists_module(&self, key: &ModuleId) -> bool {
        self.data_view.exists_module(key)
    }

    fn emit_event(&mut self, event: ContractEvent) {
        self.event_data.push(event)
    }
}
