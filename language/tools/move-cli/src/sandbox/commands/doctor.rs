// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::sandbox::utils::on_disk_state_view::OnDiskStateView;
use move_binary_format::{access::ModuleAccess, errors::PartialVMError};
use move_bytecode_utils::Modules;
use move_core_types::vm_status::StatusCode;

use anyhow::{bail, Result};
use std::{ffi::OsStr, path::Path};

/// Run sanity checks on storage and build dirs. This is primarily intended for testing the CLI;
/// doctor should never fail unless `publish --ignore-breaking changes` is used or files under
/// `storage` or `build` are modified manually. This runs the following checks:
/// (1) all modules pass the bytecode verifier
/// (2) all modules pass the linker
/// (3) all resources can be deserialized
/// (4) all events can be deserialized
/// (5) build/mv_interfaces is consistent with the global storage (TODO?)
pub fn doctor(state: &OnDiskStateView) -> Result<()> {
    fn parent_addr(p: &Path) -> &OsStr {
        p.parent().unwrap().parent().unwrap().file_name().unwrap()
    }

    // verify and link each module
    let all_modules = state.get_all_modules()?;
    let code_cache = Modules::new(&all_modules);
    for module in &all_modules {
        if bytecode_verifier::verify_module(module).is_err() {
            bail!("Failed to verify module {:?}", module.self_id())
        }

        let imm_deps = code_cache.get_immediate_dependencies(&module.self_id())?;
        if bytecode_verifier::dependencies::verify_module(module, imm_deps).is_err() {
            bail!(
                "Failed to link module {:?} against its dependencies",
                module.self_id()
            )
        }

        let cyclic_check_result = bytecode_verifier::cyclic_dependencies::verify_module(
            module,
            |module_id| {
                code_cache
                    .get_module(module_id)
                    .map_err(|_| PartialVMError::new(StatusCode::MISSING_DEPENDENCY))
                    .map(|m| m.immediate_dependencies())
            },
            |module_id| {
                code_cache
                    .get_module(module_id)
                    .map_err(|_| PartialVMError::new(StatusCode::MISSING_DEPENDENCY))
                    .map(|m| m.immediate_friends())
            },
        );
        if let Err(cyclic_check_error) = cyclic_check_result {
            // the only possible error in the CLI's context is CYCLIC_MODULE_DEPENDENCY
            assert_eq!(
                cyclic_check_error.major_status(),
                StatusCode::CYCLIC_MODULE_DEPENDENCY
            );
            bail!(
                "Cyclic module dependencies are detected with module {} in the loop",
                module.self_id()
            )
        }
    }
    // deserialize each resource
    for resource_path in state.resource_paths() {
        let resource = state.view_resource(&resource_path);
        if resource.is_err() {
            bail!(
                "Failed to deserialize resource {:?} stored under address {:?}",
                resource_path.file_name().unwrap(),
                parent_addr(&resource_path)
            )
        }
    }
    // deserialize each event
    for event_path in state.event_paths() {
        let event = state.view_events(&event_path);
        if event.is_err() {
            bail!(
                "Failed to deserialize event {:?} stored under address {:?}",
                event_path.file_name().unwrap(),
                parent_addr(&event_path)
            )
        }
    }

    Ok(())
}
