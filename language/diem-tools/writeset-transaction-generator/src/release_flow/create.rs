// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    release_flow::{
        get_commit_hash, hash_for_modules, load_latest_artifact, save_release_artifact,
        verify::verify_payload_change, ReleaseArtifact,
    },
    writeset_builder::build_changeset,
};
use anyhow::{bail, Result};
use diem_types::{
    access_path::AccessPath,
    chain_id::ChainId,
    transaction::{ChangeSet, Version, WriteSetPayload},
    write_set::{WriteOp, WriteSetMut},
};
use diem_validator_interface::{
    DebuggerStateView, DiemValidatorInterface, JsonRpcDebuggerInterface,
};
use move_binary_format::CompiledModule;
use std::collections::{BTreeMap, BTreeSet, HashSet};

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
    updated_diem_version: Option<u64>,
    release_name: &str,
) -> Result<WriteSetPayload> {
    let release_artifact = ReleaseArtifact {
        chain_id,
        version,
        stdlib_hash: hash_for_modules(
            release_modules
                .iter()
                .map(|(bytes, module)| (module.self_id(), bytes)),
        )?,
        commit_hash: get_commit_hash()?,
        diem_version: updated_diem_version,
        release_name: release_name.to_string(),
    };

    if !first_release {
        let artifact = load_latest_artifact(&chain_id)?;

        if artifact.chain_id != chain_id {
            bail!(
                "Artifact chain id does not match input chain id. Artifact: {:?} Input {:?}",
                artifact.chain_id,
                chain_id
            );
        }
        if artifact.version > version {
            bail!(
                "Artifact version is ahead of the argument: old: {:?}, new: {:?}",
                artifact.version,
                version
            );
        }

        if let (Some(old_version), Some(new_version)) =
            (artifact.diem_version, updated_diem_version)
        {
            if old_version >= new_version {
                bail!("DiemVersion should be strictly increasing")
            }
        }
    }

    let remote = Box::new(JsonRpcDebuggerInterface::new(url.as_str())?);
    let payload =
        create_release_from_artifact(&release_artifact, url.as_str(), release_modules, None)?;

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
    override_version: Option<Version>,
) -> Result<WriteSetPayload> {
    let remote = JsonRpcDebuggerInterface::new(remote_url)?;
    let remote_modules = remote
        .get_diem_framework_modules_by_version(override_version.unwrap_or(artifact.version))?;
    let modules_payload = create_release_writeset(&remote_modules, release_modules)?;

    Ok(if let Some(updated_diem_version) = artifact.diem_version {
        let state_view = DebuggerStateView::new(&remote, artifact.version);
        let (updated_version_writeset, events) = build_changeset(&state_view, |session| {
            session.set_diem_version(updated_diem_version);
        })
        .into_inner();

        let (modules, _) = match modules_payload {
            WriteSetPayload::Direct(cs) => cs,
            payload => bail!(
                "Unexpected payload; wanted WriteSetPayload::Direct, found {:?}",
                payload
            ),
        }
        .into_inner();

        if !updated_version_writeset
            .iter()
            .map(|(ap, _)| ap)
            .collect::<HashSet<_>>()
            .is_disjoint(&modules.iter().map(|(ap, _)| ap).collect::<HashSet<_>>())
        {
            bail!("DiemVersion WriteSet collides with module upgrade WriteSet");
        }

        let write_set = WriteSetMut::new(
            updated_version_writeset
                .iter()
                .chain(modules.iter())
                .cloned()
                .collect(),
        )
        .freeze()?;

        WriteSetPayload::Direct(ChangeSet::new(write_set, events))
    } else {
        modules_payload
    })
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
