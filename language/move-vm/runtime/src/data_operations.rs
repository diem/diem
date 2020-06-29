// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::prelude::*;
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier},
    vm_status::{sub_status, StatusCode},
};
use move_vm_types::{
    data_store::DataStore,
    loaded_data::runtime_types::Type,
    values::{GlobalValue, Struct, Value},
};
use vm::errors::{PartialVMError, PartialVMResult};

//
// Provides an implementation for data store bytecodes and guarantees proper invariants
// for each operation. It uses the `DataStore` for the data operation but it makes sure
// all operations are consistent to Move semantic.
//

// Publish a resource if the resource does not exists at that location.
// Return an error otherwise.
pub(crate) fn move_resource_to(
    data_store: &mut dyn DataStore,
    addr: AccountAddress,
    ty: Type,
    resource: Struct,
) -> PartialVMResult<()> {
    // a resource can be written to an AccessPath if the data does not exists or
    // it was deleted (MoveFrom)
    let can_write = match data_store.borrow_resource(addr, &ty) {
        Ok(None) => true,
        Ok(Some(_)) => false,
        Err(e) => match e.major_status() {
            StatusCode::MISSING_DATA => true,
            _ => return Err(e),
        },
    };
    if can_write {
        let new_root = GlobalValue::new(Value::struct_(resource))?;
        new_root.mark_dirty()?;
        data_store.publish_resource(addr, ty, new_root)
    } else {
        warn!(
            "[VM] Cannot write over existing resource type {:?} under address {}",
            ty, addr,
        );
        Err(PartialVMError::new(
            StatusCode::CANNOT_WRITE_EXISTING_RESOURCE,
        ))
    }
}

// Unpublish a resource if there are no live references to it.
// Return an error otherwise.
pub(crate) fn move_resource_from(
    data_store: &mut dyn DataStore,
    addr: AccountAddress,
    ty: &Type,
) -> PartialVMResult<Value> {
    let root_value = match data_store.move_resource_from(addr, ty) {
        Ok(g) => g,
        Err(e) => {
            warn!(
                "[VM] (MoveFrom) Error reading data for ({}, {:?}): {:?}",
                addr, ty, e
            );
            return Err(e);
        }
    };

    match root_value {
        Some(global_val) => Ok(Value::struct_(global_val.into_owned_struct()?)),
        None => Err(PartialVMError::new(StatusCode::DYNAMIC_REFERENCE_ERROR)
            .with_sub_status(sub_status::DRE_GLOBAL_ALREADY_BORROWED)),
    }
}

// Return true if the resource exits already at the given location, false otherwise.
pub(crate) fn resource_exists(
    data_store: &mut dyn DataStore,
    addr: AccountAddress,
    ty: &Type,
) -> PartialVMResult<(bool, AbstractMemorySize<GasCarrier>)> {
    Ok(match data_store.borrow_resource(addr, ty) {
        Ok(Some(gref)) => (true, gref.size()),
        Ok(None) | Err(_) => (false, AbstractMemorySize::new(0)),
    })
}

// Borrow a resource at a give location if the resource was not borrowed already.
// Return an error otherwise.
pub(crate) fn borrow_global<'a>(
    data_store: &'a mut dyn DataStore,
    addr: AccountAddress,
    ty: &Type,
) -> PartialVMResult<&'a GlobalValue> {
    match data_store.borrow_resource(addr, ty) {
        Ok(Some(g)) => Ok(g),
        Ok(None) => Err(
            // TODO: wrong status code?
            PartialVMError::new(StatusCode::DYNAMIC_REFERENCE_ERROR)
                .with_sub_status(sub_status::DRE_GLOBAL_ALREADY_BORROWED),
        ),
        Err(e) => {
            error!(
                "[VM] (BorrowGlobal) Error reading data for ({}, {:?}): {:?}",
                addr, ty, e
            );
            Err(e)
        }
    }
}
