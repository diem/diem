// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use crate::{
    sandbox::utils::{
        explain_publish_changeset, explain_publish_error, get_gas_status,
        on_disk_state_view::OnDiskStateView,
    },
    NativeFunctionRecord,
};
use move_lang::{self, compiled_unit::CompiledUnit, Compiler, Flags};
use move_vm_runtime::move_vm::MoveVM;

use anyhow::Result;

pub fn publish(
    natives: impl IntoIterator<Item = NativeFunctionRecord>,
    state: &OnDiskStateView,
    files: &[String],
    republish: bool,
    ignore_breaking_changes: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Compiling Move modules...")
    }

    let (_, compiled_units) = Compiler::new(files, &[state.interface_files_dir()?])
        .set_flags(Flags::empty().set_sources_shadow_deps(republish))
        .build_and_report()?;

    let num_modules = compiled_units
        .iter()
        .filter(|u| matches!(u, CompiledUnit::Module { .. }))
        .count();
    if verbose {
        println!("Found and compiled {} modules", num_modules)
    }

    let mut modules = vec![];
    for c in compiled_units {
        match c {
            CompiledUnit::Script { loc, .. } => {
                if verbose {
                    println!(
                        "Warning: Found script in specified files for publishing. But scripts \
                         cannot be published. Script found in: {}",
                        loc.file()
                    )
                }
            }
            CompiledUnit::Module { module, ident, .. } => modules.push((ident, module)),
        }
    }

    // use the the publish_module API frm the VM if we do not allow breaking changes
    if !ignore_breaking_changes {
        let vm = MoveVM::new(natives).unwrap();
        let mut gas_status = get_gas_status(None)?;
        let mut session = vm.new_session(state);

        let mut has_error = false;
        let mut id_to_ident = BTreeMap::new();
        for (ident, module) in &modules {
            let mut module_bytes = vec![];
            module.serialize(&mut module_bytes)?;

            let id = module.self_id();
            let sender = *id.address();
            id_to_ident.insert(
                id.clone(),
                ident.address_name.as_ref().map(|n| n.value.clone()),
            );

            let res = session.publish_module(module_bytes, sender, &mut gas_status);
            if let Err(err) = res {
                explain_publish_error(err, &state, module)?;
                has_error = true;
                break;
            }
        }

        if !has_error {
            let (changeset, events) = session.finish().map_err(|e| e.into_vm_status())?;
            assert!(events.is_empty());
            if verbose {
                explain_publish_changeset(&changeset, &state);
            }
            let modules: Vec<_> = changeset
                .into_modules()
                .map(|(module_id, blob_opt)| {
                    let addr_name = id_to_ident[&module_id].clone();
                    let ident = (module_id, addr_name);
                    (ident, blob_opt.expect("must be non-deletion"))
                })
                .collect();
            state.save_modules(&modules)?;
        }
    } else {
        // NOTE: the VM enforces the most strict way of module republishing and does not allow
        // backward incompatible changes, as as result, if this flag is set, we skip the VM process
        // and force the CLI to override the on-disk state directly
        let mut serialized_modules = vec![];
        for (ident, module) in modules {
            let (address_name_opt, id) = ident.into_module_id();
            let address_name_opt = address_name_opt.map(|n| n.value);
            let mut module_bytes = vec![];
            module.serialize(&mut module_bytes)?;
            serialized_modules.push(((id, address_name_opt), module_bytes));
        }
        state.save_modules(&serialized_modules)?;
    }

    Ok(())
}
