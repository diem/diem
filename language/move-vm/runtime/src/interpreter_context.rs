// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::prelude::*;
use libra_types::{
    access_path::AccessPath,
    contract_event::ContractEvent,
    language_storage::ModuleId,
    vm_error::{sub_status, StatusCode},
};
use move_vm_types::{
    chain_state::ChainState,
    loaded_data::types::StructType,
    values::{GlobalValue, Struct, Value},
};
use vm::{
    errors::*,
    gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier, GasUnits},
};

/// The `InterpreterContext` context trait specifies the mutations that are allowed to the
/// `TransactionExecutionContext` within the interpreter.
pub trait InterpreterContext {
    fn move_resource_to(
        &mut self,
        ap: &AccessPath,
        ty: StructType,
        resource: Struct,
    ) -> VMResult<()>;

    fn move_resource_from(&mut self, ap: &AccessPath, ty: StructType) -> VMResult<Value>;

    fn resource_exists(
        &mut self,
        ap: &AccessPath,
        ty: StructType,
    ) -> VMResult<(bool, AbstractMemorySize<GasCarrier>)>;

    fn borrow_global(&mut self, ap: &AccessPath, ty: StructType) -> VMResult<&GlobalValue>;

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
        ty: StructType,
        resource: Struct,
    ) -> VMResult<()> {
        // a resource can be written to an AccessPath if the data does not exists or
        // it was deleted (MoveFrom)
        let can_write = match self.borrow_resource(ap, ty.clone()) {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(e) => match e.major_status {
                StatusCode::MISSING_DATA => true,
                _ => return Err(e),
            },
        };
        if can_write {
            let new_root = GlobalValue::new(Value::struct_(resource))?;
            new_root.mark_dirty()?;
            self.publish_resource(ap, (ty, new_root))
        } else {
            warn!("[VM] Cannot write over existing resource {}", ap);
            Err(vm_error(
                Location::new(),
                StatusCode::CANNOT_WRITE_EXISTING_RESOURCE,
            ))
        }
    }

    fn move_resource_from(&mut self, ap: &AccessPath, ty: StructType) -> VMResult<Value> {
        let root_value = match ChainState::move_resource_from(self, ap, ty) {
            Ok(g) => g,
            Err(e) => {
                warn!("[VM] (MoveFrom) Error reading data for {}: {:?}", ap, e);
                return Err(e);
            }
        };

        match root_value {
            Some(global_val) => Ok(Value::struct_(global_val.into_owned_struct()?)),
            None => Err(
                vm_error(Location::new(), StatusCode::DYNAMIC_REFERENCE_ERROR)
                    .with_sub_status(sub_status::DRE_GLOBAL_ALREADY_BORROWED),
            ),
        }
    }

    fn resource_exists(
        &mut self,
        ap: &AccessPath,
        ty: StructType,
    ) -> VMResult<(bool, AbstractMemorySize<GasCarrier>)> {
        Ok(match self.borrow_resource(ap, ty) {
            Ok(Some(gref)) => (true, gref.size()),
            Ok(None) | Err(_) => (false, AbstractMemorySize::new(0)),
        })
    }

    fn borrow_global(&mut self, ap: &AccessPath, ty: StructType) -> VMResult<&GlobalValue> {
        match self.borrow_resource(ap, ty) {
            Ok(Some(g)) => Ok(g),
            Ok(None) => Err(
                // TODO: wrong status code?
                vm_error(Location::new(), StatusCode::DYNAMIC_REFERENCE_ERROR)
                    .with_sub_status(sub_status::DRE_GLOBAL_ALREADY_BORROWED),
            ),
            Err(e) => {
                error!("[VM] (BorrowGlobal) Error reading data for {}: {:?}", ap, e);
                Err(e)
            }
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
