// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{loaded_data::struct_def::StructDef, values::GlobalValue};
use libra_types::{
    access_path::AccessPath, contract_event::ContractEvent, language_storage::ModuleId,
};
use vm::{
    errors::VMResult,
    gas_schedule::{GasCarrier, GasUnits},
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
