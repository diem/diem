// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::errors::{PartialVMError, VMResult};
use move_core_types::{effects::ChangeSet, language_storage::ModuleId, resolver::MoveResolver};

/// The result returned by the stackless VM does not contain code offsets and indices. In order to
/// do cross-vm comparison, we need to adapt the Move VM result by removing these fields.
pub fn adapt_move_vm_result<T>(result: VMResult<T>) -> VMResult<T> {
    result.map_err(|err| {
        let (status_code, sub_status, _, location, _, _) = err.all_data();
        let adapted = PartialVMError::new(status_code);
        let adapted = match sub_status {
            None => adapted,
            Some(status_code) => adapted.with_sub_status(status_code),
        };
        adapted.finish(location)
    })
}

/// The change-set produced by the stackless VM guarantees that for a global resource, if the
/// underlying value is not changed in the execution, there will not be an entry in the change set.
/// The same guarantee is not provided by the Move VM. In Move VM, we could borrow_global_mut but
/// write the same value back instead of an updated value. In this case, the Move VM produces an
/// entry in the change_set.
pub fn adapt_move_vm_change_set<S: MoveResolver>(
    change_set_result: VMResult<ChangeSet>,
    old_storage: &S,
) -> VMResult<ChangeSet> {
    change_set_result.map(|change_set| adapt_move_vm_change_set_internal(change_set, old_storage))
}

fn adapt_move_vm_change_set_internal<S: MoveResolver>(
    change_set: ChangeSet,
    old_storage: &S,
) -> ChangeSet {
    let mut adapted = ChangeSet::new();
    for (addr, state) in change_set.into_inner() {
        let (modules, resources) = state.into_inner();
        for (tag, val) in resources {
            match val {
                // deletion
                None => adapted.unpublish_resource(addr, tag).unwrap(),
                // addition / modification
                Some(new_val) => match old_storage.get_resource(&addr, &tag).unwrap() {
                    // addition
                    None => adapted.publish_resource(addr, tag, new_val).unwrap(),
                    // modification is only added to change_set if the values actually change
                    Some(old_val) => {
                        if new_val != old_val {
                            adapted.publish_resource(addr, tag, new_val).unwrap();
                        }
                    }
                },
            }
        }
        for (module_name, blob_opt) in modules {
            let module_id = ModuleId::new(addr, module_name);
            match blob_opt {
                // deletion
                None => adapted.unpublish_module(module_id).unwrap(),
                // addition
                Some(blob) => adapted.publish_module(module_id, blob).unwrap(),
            }
        }
    }
    adapted
}
