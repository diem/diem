// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    sandbox::utils::{
        contains_module, explain_execution_effects, explain_execution_error, get_gas_status,
        is_bytecode_file, maybe_commit_effects, on_disk_state_view::OnDiskStateView,
    },
    NativeFunctionRecord,
};
use move_binary_format::file_format::{CompiledModule, CompiledScript};
use move_core_types::{
    account_address::AccountAddress,
    errmap::ErrorMapping,
    identifier::IdentStr,
    language_storage::TypeTag,
    transaction_argument::{convert_txn_args, TransactionArgument},
};
use move_lang::{
    self, compiled_unit::AnnotatedCompiledUnit, shared::AddressBytes, Compiler, Flags,
};
use move_vm_runtime::move_vm::MoveVM;

use anyhow::{anyhow, bail, Result};
use std::{collections::BTreeMap, fs, path::Path};

pub fn run(
    natives: impl IntoIterator<Item = NativeFunctionRecord>,
    error_descriptions: &ErrorMapping,
    state: &OnDiskStateView,
    script_file: &str,
    script_name_opt: &Option<String>,
    signers: &[String],
    txn_args: &[TransactionArgument],
    vm_type_args: Vec<TypeTag>,
    named_address_mapping: BTreeMap<String, AddressBytes>,
    gas_budget: Option<u64>,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    fn compile_script(
        state: &OnDiskStateView,
        script_file: &str,
        named_address_mapping: BTreeMap<String, AddressBytes>,
        verbose: bool,
    ) -> Result<Option<CompiledScript>> {
        if verbose {
            println!("Compiling transaction script...")
        }
        let (_files, compiled_units) =
            Compiler::new(&[script_file.to_string()], &[state.interface_files_dir()?])
                .set_flags(Flags::empty().set_sources_shadow_deps(false))
                .set_named_address_values(named_address_mapping)
                .build_and_report()?;

        let mut script_opt = None;
        for c in compiled_units {
            match c {
                AnnotatedCompiledUnit::Script(annot_script) => {
                    if script_opt.is_some() {
                        bail!("Error: Found more than one script")
                    }
                    script_opt = Some(annot_script.named_script.script)
                }
                AnnotatedCompiledUnit::Module(annot_module) => {
                    if verbose {
                        println!(
                            "Warning: Found module '{}' in file specified for the script. This \
                             module will not be published.",
                            annot_module.module_ident()
                        )
                    }
                }
            }
        }

        Ok(script_opt)
    }

    let path = Path::new(script_file);
    if !path.exists() {
        bail!("Script file {:?} does not exist", path)
    };
    let bytecode = if is_bytecode_file(path) {
        assert!(
            state.is_module_path(path) || !contains_module(path),
            "Attempting to run module {:?} outside of the `storage/` directory.
move run` must be applied to a module inside `storage/`",
            path
        );
        // script bytecode; read directly from file
        fs::read(path)?
    } else {
        // script source file; compile first and then extract bytecode
        let script_opt = compile_script(state, script_file, named_address_mapping, verbose)?;
        match script_opt {
            Some(script) => {
                let mut script_bytes = vec![];
                script.serialize(&mut script_bytes)?;
                script_bytes
            }
            None => bail!("Unable to find script in file {:?}", script_file),
        }
    };

    let signer_addresses = signers
        .iter()
        .map(|s| AccountAddress::from_hex_literal(s))
        .collect::<Result<Vec<AccountAddress>, _>>()?;
    // TODO: parse Value's directly instead of going through the indirection of TransactionArgument?
    let vm_args: Vec<Vec<u8>> = convert_txn_args(txn_args);

    let vm = MoveVM::new(natives).unwrap();
    let mut gas_status = get_gas_status(gas_budget)?;
    let mut session = vm.new_session(state);

    let script_type_parameters = vec![];
    let script_parameters = vec![];
    let res = match script_name_opt {
        Some(script_name) => {
            // script fun. parse module, extract script ID to pass to VM
            let module = CompiledModule::deserialize(&bytecode)
                .map_err(|e| anyhow!("Error deserializing module: {:?}", e))?;
            session
                .execute_script_function(
                    &module.self_id(),
                    IdentStr::new(script_name)?,
                    vm_type_args.clone(),
                    vm_args,
                    signer_addresses.clone(),
                    &mut gas_status,
                )
                .map(|_| ())
        }
        None => session.execute_script(
            bytecode.to_vec(),
            vm_type_args.clone(),
            vm_args,
            signer_addresses.clone(),
            &mut gas_status,
        ),
    };

    if let Err(err) = res {
        explain_execution_error(
            error_descriptions,
            err,
            state,
            &script_type_parameters,
            &script_parameters,
            &vm_type_args,
            &signer_addresses,
            txn_args,
        )
    } else {
        let (changeset, events) = session.finish().map_err(|e| e.into_vm_status())?;
        if verbose {
            explain_execution_effects(&changeset, &events, state)?
        }
        maybe_commit_effects(!dry_run, changeset, events, state)
    }
}
