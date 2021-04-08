// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains verification of usage of dependencies for modules
use move_binary_format::{
    access::ModuleAccess,
    errors::{Location, PartialVMError, PartialVMResult, VMResult},
    file_format::CompiledModule,
};
use move_core_types::{language_storage::ModuleId, vm_status::StatusCode};
use std::collections::BTreeSet;

// This function performs a DFS in the module graph starting from each node in `items_to_explore`
// and explores the neighbors of a node using the `immediate_nexts` closure.
//
// During the DFS,
// - 1) if the `target_module_id` is found, the exploration will be short-circuited and returns a
//   PartialVMError bearing the StatusCode specified in `error_on_cycle`.
// - 2) if the `target_module_id` is not found, the modules visited in the DFS will be returned at
//   the end of function execution.
fn collect_all_with_cycle_detection<F: Fn(&ModuleId) -> PartialVMResult<Vec<ModuleId>>>(
    target_module_id: &ModuleId,
    items_to_explore: &[ModuleId],
    immediate_nexts: &F,
    error_on_cycle: StatusCode,
) -> PartialVMResult<BTreeSet<ModuleId>> {
    fn collect_all_with_cycle_detection_recursive<
        F: Fn(&ModuleId) -> PartialVMResult<Vec<ModuleId>>,
    >(
        target_module_id: &ModuleId,
        cursor_module_id: &ModuleId,
        immediate_nexts: &F,
        visited_modules: &mut BTreeSet<ModuleId>,
    ) -> PartialVMResult<bool> {
        if cursor_module_id == target_module_id {
            return Ok(true);
        }
        if visited_modules.insert(cursor_module_id.clone()) {
            for next in immediate_nexts(cursor_module_id)? {
                if collect_all_with_cycle_detection_recursive(
                    target_module_id,
                    &next,
                    immediate_nexts,
                    visited_modules,
                )? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    let mut visited_modules = BTreeSet::new();
    for item in items_to_explore {
        if collect_all_with_cycle_detection_recursive(
            target_module_id,
            item,
            immediate_nexts,
            &mut visited_modules,
        )? {
            return Err(PartialVMError::new(error_on_cycle));
        }
    }
    Ok(visited_modules)
}

pub fn verify_module<D, F>(module: &CompiledModule, imm_deps: D, imm_friends: F) -> VMResult<()>
where
    D: Fn(&ModuleId) -> PartialVMResult<Vec<ModuleId>>,
    F: Fn(&ModuleId) -> PartialVMResult<Vec<ModuleId>>,
{
    verify_module_impl(module, imm_deps, imm_friends)
        .map_err(|e| e.finish(Location::Module(module.self_id())))
}

fn verify_module_impl<D, F>(
    module: &CompiledModule,
    imm_deps: D,
    imm_friends: F,
) -> PartialVMResult<()>
where
    D: Fn(&ModuleId) -> PartialVMResult<Vec<ModuleId>>,
    F: Fn(&ModuleId) -> PartialVMResult<Vec<ModuleId>>,
{
    let self_id = module.self_id();

    // collect and check that there is no cyclic dependency relation
    let all_deps = collect_all_with_cycle_detection(
        &self_id,
        &module.immediate_dependencies(),
        &imm_deps,
        StatusCode::CYCLIC_MODULE_DEPENDENCY,
    )?;

    // collect and check that there is no cyclic friend relation
    let all_friends = collect_all_with_cycle_detection(
        &self_id,
        &module.immediate_friends(),
        &imm_friends,
        StatusCode::CYCLIC_MODULE_FRIENDSHIP,
    )?;

    // check that any direct/transitive dependency is neither a direct nor transitive friend
    match all_deps.intersection(&all_friends).next() {
        Some(overlap) => Err(PartialVMError::new(
            StatusCode::INVALID_FRIEND_DECL_WITH_MODULES_IN_DEPENDENCIES,
        )
        .with_message(format!(
            "At least one module, {}, appears in both the dependency set and the friend set",
            overlap
        ))),
        None => Ok(()),
    }
}
