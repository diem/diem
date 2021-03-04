// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::release_flow::{
    hash_for_modules, load_artifact, save_release_artifact, verify::verify_payload_change,
    ReleaseArtifact,
};
use anyhow::{bail, Result};
use diem_types::{
    access_path::AccessPath,
    chain_id::ChainId,
    transaction::{ChangeSet, WriteSetPayload},
    write_set::{WriteOp, WriteSetMut},
};
use diem_validator_interface::{DiemValidatorInterface, JsonRpcDebuggerInterface};
use std::collections::{BTreeMap, BTreeSet};
use vm::CompiledModule;

pub fn create_release(
    // ChainID to distinguish the diem network. e.g: PREMAINNET
    chain_id: ChainId,
    // Public JSON-rpc endpoint URL.
    // TODO: Get rid of this URL argument once we have a stable mapping from ChainId to its url.
    url: String,
    // Blockchain height
    version: u64,
    // Set the flag to true in the first release. This will manually create the first release artifact on disk.
    first_release: bool,
    release_modules: &[(Vec<u8>, CompiledModule)],
) -> Result<WriteSetPayload> {
    let release_artifact = ReleaseArtifact {
        chain_id,
        version,
        stdlib_hash: hash_for_modules(
            release_modules
                .iter()
                .map(|(bytes, module)| (module.self_id(), bytes)),
        )?,
    };

    if first_release {
        if load_artifact(&chain_id).is_ok() {
            bail!("Previous release existed");
        }
        save_release_artifact(release_artifact.clone())?;
    }
    let artifact = load_artifact(&chain_id)?;

    if artifact.chain_id != chain_id {
        bail!("Artifact mismatch with on disk file");
    }
    if artifact.version > version {
        bail!(
            "Artifact version is ahead of the argument: old: {:?}, new: {:?}",
            artifact.version,
            version
        );
    }

    let remote = Box::new(JsonRpcDebuggerInterface::new(url.as_str())?);
    let payload = create_release_from_artifact(&release_artifact, url.as_str(), release_modules)?;
    verify_payload_change(
        remote,
        Some(version),
        &payload,
        release_modules.iter().map(|(_bytes, m)| m),
    )?;
    save_release_artifact(release_artifact)?;
    Ok(payload)
}

pub(crate) fn create_release_from_artifact(
    artifact: &ReleaseArtifact,
    remote_url: &str,
    release_modules: &[(Vec<u8>, CompiledModule)],
) -> Result<WriteSetPayload> {
    let remote = JsonRpcDebuggerInterface::new(remote_url)?;
    let remote_modules = remote.get_diem_framework_modules_by_version(artifact.version)?;
    create_release_writeset(&remote_modules, release_modules)
}

pub(crate) fn create_release_writeset(
    remote_frameworks: &[CompiledModule],
    local_frameworks: &[(Vec<u8>, CompiledModule)],
) -> Result<WriteSetPayload> {
    let remote_framework_map = remote_frameworks
        .iter()
        .map(|m| (m.self_id(), m))
        .collect::<BTreeMap<_, _>>();
    let remote_ids = remote_framework_map.keys().collect::<BTreeSet<_>>();
    let local_framework_map = local_frameworks
        .iter()
        .map(|(bytes, module)| (module.self_id(), (bytes, module)))
        .collect::<BTreeMap<_, _>>();
    let local_ids = local_framework_map.keys().collect::<BTreeSet<_>>();

    let mut framework_changes = BTreeMap::new();

    // 1. Insert new modules to be published.
    for module_id in local_ids.difference(&remote_ids) {
        let module = *local_framework_map
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
        let (local_bytes, local_module) = *local_framework_map
            .get(*module_id)
            .expect("ModuleID not found in local stdlib");
        let remote_module = remote_framework_map
            .get(*module_id)
            .expect("ModuleID not found in local stdlib");
        if &local_module != remote_module {
            framework_changes.insert(*module_id, Some((local_bytes, local_module)));
        }
    }

    let mut write_patch = WriteSetMut::new(vec![]);
    for (id, module_opt) in framework_changes.into_iter() {
        let path = AccessPath::code_access_path(id.clone());
        match module_opt {
            Some((bytes, _)) => {
                write_patch.push((path, WriteOp::Value((*bytes).clone())));
            }
            None => write_patch.push((path, WriteOp::Deletion)),
        }
    }

    Ok(WriteSetPayload::Direct(ChangeSet::new(
        write_patch.freeze()?,
        vec![],
    )))
}
