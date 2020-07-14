// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    loaded_data::runtime_types::Type,
    values::{GlobalValue, Value},
};
use move_core_types::{account_address::AccountAddress, language_storage::ModuleId};
use vm::errors::{PartialVMResult, VMResult};

/// Provide an implementation for bytecodes related to data with a given data store.
///
/// The `DataStore` is a generic concept that includes both data and events.
/// A default implementation of the `DataStore` is `TransactionDataCache` which provides
/// an in memory cache for a given transaction and the atomic transactional changes
/// proper of a script execution (transaction).
pub trait DataStore {
    // ---
    // StateStore operations
    // ---

    /// Publish a resource.
    fn publish_resource(
        &mut self,
        addr: AccountAddress,
        ty: Type,
        gv: GlobalValue,
    ) -> PartialVMResult<()>;

    /// Get a reference to a resource.
    fn borrow_resource(
        &mut self,
        addr: AccountAddress,
        ty: &Type,
    ) -> PartialVMResult<Option<&GlobalValue>>;

    /// Transfer ownership of a resource to the VM.
    fn move_resource_from(
        &mut self,
        addr: AccountAddress,
        ty: &Type,
    ) -> PartialVMResult<Option<GlobalValue>>;

    /// Get the serialized format of a `CompiledModule` given a `ModuleId`.
    fn load_module(&self, module_id: &ModuleId) -> VMResult<Vec<u8>>;

    /// Publish a module.
    fn publish_module(&mut self, module_id: &ModuleId, blob: Vec<u8>) -> VMResult<()>;

    /// Check if this module exists.
    fn exists_module(&self, module_id: &ModuleId) -> VMResult<bool>;

    // ---
    // EventStore operations
    // ---

    /// Emit an event to the EventStore
    fn emit_event(&mut self, guid: Vec<u8>, seq_num: u64, ty: Type, val: Value);
}
