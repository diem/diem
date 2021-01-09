// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail, Result};
use diem_types::{
    access_path::{AccessPath, Path},
    account_config::diem_root_address,
    account_state::AccountState,
    transaction::{ChangeSet, TransactionStatus, Version, WriteSetPayload},
    write_set::{WriteOp, WriteSetMut},
};
use diem_validator_interface::{DebuggerStateView, DiemValidatorInterface};
use diem_vm::{data_cache::StateViewCache, transaction_metadata::TransactionMetadata, DiemVM};
use move_core_types::vm_status::{KeptVMStatus, VMStatus};
use move_vm_runtime::logging::NoContextLog;
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
};
use stdlib::build_stdlib;
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

/// Make sure that given a remote state, applying the `payload` will make sure the new on-chain
/// states contains the exact same Diem Framework modules as the locally compiled stdlib.
pub fn verify_payload_change<I: DiemValidatorInterface>(
    validator: &I,
    block_height_opt: Option<Version>,
    payload: &WriteSetPayload,
) -> Result<()> {
    let block_height = match block_height_opt {
        Some(h) => h,
        None => validator.get_latest_version()?,
    };
    let state_view = DebuggerStateView::new(validator, block_height);
    let mut vm = DiemVM::new(&state_view);
    let cache = StateViewCache::new(&state_view);
    let log_context = NoContextLog::new();
    let mut txn_data = TransactionMetadata::default();
    txn_data.sequence_number =
        match validator.get_account_state_by_version(diem_root_address(), block_height)? {
            Some(account) => AccountState::try_from(&account)?
                .get_account_resource()?
                .ok_or_else(|| anyhow!("Diem root account doesn't exist"))?
                .sequence_number(),
            None => bail!("Diem root account blob doesn't exist"),
        };
    txn_data.sender = diem_root_address();

    let (result, output) =
        vm.execute_writeset_transaction(&cache, payload, txn_data, &log_context)?;

    if result != VMStatus::Executed {
        bail!("Unexpected abort from running WriteSetPayload")
    }

    if output.status() != &TransactionStatus::Keep(KeptVMStatus::Executed) {
        bail!("Unexpected transaction status from running WriteSetPayload")
    }

    // Should contain a reconfiguration event
    let new_epoch_event_key = diem_types::on_chain_config::new_epoch_event_key();
    if !output
        .events()
        .iter()
        .any(|e| *e.key() == new_epoch_event_key)
    {
        bail!("Output WriteSet won't trigger a reconfiguration")
    }

    // Applying this writeset should make Diem framework equal to its on-disk status
    let mut old_modules = validator
        .get_diem_framework_modules_by_version(block_height)?
        .into_iter()
        .map(|m| (m.self_id(), m))
        .collect::<BTreeMap<_, _>>();

    for (access_path, write_op) in output.write_set() {
        let path = bcs::from_bytes::<Path>(access_path.path.as_slice())?;
        if let Path::Code(module_id) = path {
            match write_op {
                WriteOp::Deletion => {
                    println!("Deleting deprecated module: {:?}", module_id);
                    if old_modules.remove(&module_id).is_none() {
                        bail!("Removing non-existent module")
                    }
                }
                WriteOp::Value(v) => {
                    let updated_module = match CompiledModule::deserialize(v.as_slice()) {
                        Ok(m) => m,
                        Err(e) => bail!("Unexpected module deserialize error {:?}", e),
                    };

                    match old_modules.insert(module_id.clone(), updated_module.clone()) {
                        Some(_) => println!(
                            "Updating existing module: {:?} \n {:#?}",
                            module_id, updated_module
                        ),
                        None => println!(
                            "Adding new module: {:?} \n {:#?}",
                            module_id, updated_module
                        ),
                    }
                }
            }
        }
    }

    let local_modules = build_stdlib()
        .into_iter()
        .map(|(_, m)| (m.self_id(), m))
        .collect::<BTreeMap<_, _>>();
    if local_modules.len() != old_modules.len() {
        bail!(
            "Found {:?} modules locally but {:?} in remote storage",
            local_modules.len(),
            old_modules.len()
        )
    }
    for (remote, local) in old_modules.values().zip(local_modules.values()) {
        if remote != local {
            bail!("Applying writeset onto the state causes module {:?} diverge from the on disk files", local.self_id())
        }
    }
    Ok(())
}
