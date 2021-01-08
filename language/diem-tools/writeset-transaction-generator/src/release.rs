// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_types::{
    access_path::AccessPath,
    transaction::{ChangeSet, WriteSetPayload},
    write_set::{WriteOp, WriteSetMut},
};
use std::collections::{BTreeMap, BTreeSet};
use vm::CompiledModule;

pub fn create_release_writeset(
    remote_frameworks: Vec<CompiledModule>,
    local_frameworks: Vec<CompiledModule>,
) -> Result<WriteSetPayload> {
    let remote_framework_map = remote_frameworks
        .into_iter()
        .map(|m| (m.self_id(), m))
        .collect::<BTreeMap<_, _>>();
    let remote_ids = remote_framework_map.keys().collect::<BTreeSet<_>>();
    let local_framework_map = local_frameworks
        .into_iter()
        .map(|m| (m.self_id(), m))
        .collect::<BTreeMap<_, _>>();
    let local_ids = local_framework_map.keys().collect::<BTreeSet<_>>();

    let mut framework_changes = BTreeMap::new();

    // 1. Insert new modules to be published.
    for module_id in local_ids.difference(&remote_ids) {
        let module = local_framework_map
            .get(*module_id)
            .expect("ModuleID not found in local stdlib");
        framework_changes.insert(*module_id, Some(module));
    }

    // 2. Remove modules that are already deleted locally.
    for module_id in remote_ids.difference(&local_ids) {
        framework_changes.insert(*module_id, None);
    }

    // 3. Check the diff between on chain modules and local modules, update when local bytes is different.
    for module_id in local_ids.intersection(&remote_ids) {
        let local_module = local_framework_map
            .get(*module_id)
            .expect("ModuleID not found in local stdlib");
        let remote_module = remote_framework_map
            .get(*module_id)
            .expect("ModuleID not found in local stdlib");
        if local_module != remote_module {
            framework_changes.insert(*module_id, Some(local_module));
        }
    }

    let mut write_patch = WriteSetMut::new(vec![]);
    for (id, module) in framework_changes.into_iter() {
        let path = AccessPath::code_access_path(id.clone());
        match module {
            Some(m) => {
                let mut bytes = vec![];
                m.serialize(&mut bytes)?;
                write_patch.push((path, WriteOp::Value(bytes)));
            }
            None => write_patch.push((path, WriteOp::Deletion)),
        }
    }

    Ok(WriteSetPayload::Direct(ChangeSet::new(
        write_patch.freeze()?,
        vec![],
    )))
}
