// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chain_state::ChainState;
use libra_logger::prelude::*;
use libra_types::{
    access_path::AccessPath,
    contract_event::ContractEvent,
    language_storage::ModuleId,
    vm_error::{sub_status, StatusCode},
};
use vm::{
    errors::*,
    gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier, GasUnits},
};
use vm_runtime_types::{
    loaded_data::struct_def::StructDef,
    value::{GlobalRef, Struct, Value},
};

/// The `InterpreterContext` context trait specifies the mutations that are allowed to the
/// `TransactionExecutionContext` within the interpreter.
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

    fn exists_module(&self, m: &ModuleId) -> bool;

    fn load_module(&self, module: &ModuleId) -> VMResult<Vec<u8>>;

    fn publish_module(&mut self, module_id: ModuleId, module: Vec<u8>) -> VMResult<()>;
}

impl<T: ChainState> InterpreterContext for T {
    fn move_resource_to(
        &mut self,
        ap: &AccessPath,
        def: StructDef,
        resource: Struct,
    ) -> VMResult<()> {
        // a resource can be written to an AccessPath if the data does not exists or
        // it was deleted (MoveFrom)
        let can_write = match self.load_data(ap, def) {
            Ok(data) => data.is_deleted(),
            Err(e) => match e.major_status {
                StatusCode::MISSING_DATA => true,
                _ => return Err(e),
            },
        };
        if can_write {
            let new_root = GlobalRef::move_to(ap.clone(), resource);
            self.publish_resource(ap, new_root)
        } else {
            warn!("[VM] Cannot write over existing resource {}", ap);
            Err(vm_error(
                Location::new(),
                StatusCode::CANNOT_WRITE_EXISTING_RESOURCE,
            ))
        }
    }

    fn move_resource_from(&mut self, ap: &AccessPath, def: StructDef) -> VMResult<Value> {
        let root_ref = match self.load_data(ap, def) {
            Ok(gref) => gref,
            Err(e) => {
                warn!("[VM] (MoveFrom) Error reading data for {}: {:?}", ap, e);
                return Err(e);
            }
        };
        // is_loadable() checks ref count and whether the data was deleted
        if root_ref.is_loadable() {
            Ok(root_ref.move_from()?)
        } else {
            Err(
                vm_error(Location::new(), StatusCode::DYNAMIC_REFERENCE_ERROR)
                    .with_sub_status(sub_status::DRE_GLOBAL_ALREADY_BORROWED),
            )
        }
    }

    fn resource_exists(
        &mut self,
        ap: &AccessPath,
        def: StructDef,
    ) -> VMResult<(bool, AbstractMemorySize<GasCarrier>)> {
        Ok(match self.load_data(ap, def) {
            Ok(gref) => {
                if gref.is_deleted() {
                    (false, AbstractMemorySize::new(0))
                } else {
                    (true, gref.size())
                }
            }
            Err(_) => (false, AbstractMemorySize::new(0)),
        })
    }

    fn borrow_global(&mut self, ap: &AccessPath, def: StructDef) -> VMResult<GlobalRef> {
        let root_ref = match self.load_data(ap, def) {
            Ok(gref) => gref,
            Err(e) => {
                error!("[VM] (BorrowGlobal) Error reading data for {}: {:?}", ap, e);
                return Err(e);
            }
        };
        // is_loadable() checks ref count and whether the data was deleted
        if root_ref.is_loadable() {
            // shallow_ref increment ref count
            Ok(root_ref.clone())
        } else {
            Err(
                vm_error(Location::new(), StatusCode::DYNAMIC_REFERENCE_ERROR)
                    .with_sub_status(sub_status::DRE_GLOBAL_ALREADY_BORROWED),
            )
        }
    }

    fn push_event(&mut self, event: ContractEvent) {
        self.emit_event(event)
    }

    fn remaining_gas(&self) -> GasUnits<GasCarrier> {
        self.remaining_gas()
    }

    fn deduct_gas(&mut self, amount: GasUnits<GasCarrier>) -> VMResult<()> {
        self.deduct_gas(amount)
    }

    fn exists_module(&self, m: &ModuleId) -> bool {
        self.exists_module(m)
    }

    fn load_module(&self, module: &ModuleId) -> VMResult<Vec<u8>> {
        self.load_module(module)
    }

    fn publish_module(&mut self, module_id: ModuleId, module: Vec<u8>) -> VMResult<()> {
        self.publish_module(module_id, module)
    }
}
