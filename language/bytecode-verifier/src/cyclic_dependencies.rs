// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains verification of usage of dependencies for modules
use diem_types::vm_status::StatusCode;
use move_core_types::language_storage::ModuleId;
use std::collections::BTreeSet;
use vm::{
    access::ModuleAccess,
    errors::{Location, PartialVMError, PartialVMResult, VMResult},
    file_format::CompiledModule,
};

pub fn verify_module<F: Fn(&ModuleId) -> PartialVMResult<Vec<ModuleId>>>(
    module: &CompiledModule,
    immediate_module_dependencies: F,
) -> VMResult<()> {
    verify_module_impl(module, immediate_module_dependencies)
        .map_err(|e| e.finish(Location::Module(module.self_id())))
}

fn verify_module_impl<F: Fn(&ModuleId) -> PartialVMResult<Vec<ModuleId>>>(
    module: &CompiledModule,
    immediate_module_dependencies: F,
) -> PartialVMResult<()> {
    fn check_existence_in_dependency_recursive<
        F: Fn(&ModuleId) -> PartialVMResult<Vec<ModuleId>>,
    >(
        target_module_id: &ModuleId,
        cursor_module_id: &ModuleId,
        immediate_module_dependencies: &F,
        visited_modules: &mut BTreeSet<ModuleId>,
    ) -> PartialVMResult<bool> {
        if cursor_module_id == target_module_id {
            return Ok(true);
        }
        if visited_modules.insert(cursor_module_id.clone()) {
            for next in immediate_module_dependencies(cursor_module_id)? {
                if check_existence_in_dependency_recursive(
                    target_module_id,
                    &next,
                    immediate_module_dependencies,
                    visited_modules,
                )? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    let self_id = module.self_id();
    let mut visited_modules = BTreeSet::new();
    for dep in module.immediate_module_dependencies() {
        if check_existence_in_dependency_recursive(
            &self_id,
            &dep,
            &immediate_module_dependencies,
            &mut visited_modules,
        )? {
            return Err(PartialVMError::new(StatusCode::CYCLIC_MODULE_DEPENDENCY));
        }
    }
    Ok(())
}
