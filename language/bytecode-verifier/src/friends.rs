// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains verification of usage of dependencies for modules
use move_binary_format::{
    access::ModuleAccess,
    errors::{Location, PartialVMError, PartialVMResult, VMResult},
    file_format::CompiledModule,
};
use move_core_types::vm_status::StatusCode;

pub fn verify_module(module: &CompiledModule) -> VMResult<()> {
    verify_module_impl(module).map_err(|e| e.finish(Location::Module(module.self_id())))
}

fn verify_module_impl(module: &CompiledModule) -> PartialVMResult<()> {
    // cannot make friends with the module itself
    let self_handle = module.self_handle();
    if module.friend_decls().contains(self_handle) {
        return Err(PartialVMError::new(
            StatusCode::INVALID_FRIEND_DECL_WITH_SELF,
        ));
    }

    // cannot make friends with modules outside of the account address
    //
    // NOTE: this constraint is a policy decision rather than a technical requirement. The VM and
    // other bytecode verifier passes do not rely on the assumption that friend modules must be
    // declared within the same account address.
    //
    // However, lacking a definite use case of friending modules across account boundaries, and also
    // to minimize the associated changes on the module publishing flow, we temporarily enforce this
    // constraint and we may consider lifting this limitation in the future.
    let self_address =
        module.address_identifier_at(module.module_handle_at(module.self_handle_idx()).address);
    let has_external_friend = module
        .friend_decls()
        .iter()
        .any(|handle| module.address_identifier_at(handle.address) != self_address);
    if has_external_friend {
        return Err(PartialVMError::new(
            StatusCode::INVALID_FRIEND_DECL_WITH_MODULES_OUTSIDE_ACCOUNT_ADDRESS,
        ));
    }

    Ok(())
}
