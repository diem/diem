// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use errmapgen::ErrorMapping;
use move_cli::OnDiskStateView;
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasUnits},
    language_storage::TypeTag,
    parser,
    transaction_argument::TransactionArgument,
    vm_status::{AbortLocation, StatusCode, VMStatus},
};
use move_lang::{self, compiled_unit::CompiledUnit, shared::Address};
use move_vm_runtime::{data_cache::TransactionEffects, move_vm::MoveVM};
use move_vm_types::{gas_schedule, values::Value};
use vm::{
    errors::VMError,
    file_format::{CompiledModule, CompiledScript},
};

use anyhow::{anyhow, bail, Result};
use std::{fs, path::Path};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "Move", about = "CLI frontend for Move compiler and VM")]
struct Move {
    /// Directory storing Move resources and module bytecodes produced by script execution.
    #[structopt(name = "move-data", long = "move-data", default_value = "move_data")]
    move_data: String,
    /// Directory storing Move source files that will be compiled and loaded into the VM before
    /// script execution.
    #[structopt(name = "move-src", long = "move-src", default_value = "move_src")]
    move_src: String,
    // Command to be run.
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Compile the modules in `move_data` and the given script (if any).
    #[structopt(name = "compile")]
    Compile {
        /// Path to script to compile.
        #[structopt(name = "script")]
        script_file: Option<String>,
        /// If true, commit the bytecodes for the compiled modules to disk.
        #[structopt(long = "commit", short = "c")]
        commit: bool,
    },
    /// Compile/run a Move script that reads/writes resources stored on disk in `move_data`. This
    /// command compiles each each module stored in `move_src` and loads it into the VM before
    /// running the script.
    #[structopt(name = "run")]
    Run {
        /// Path to script to compile and run.
        #[structopt(name = "script")]
        script_file: String,
        /// Possibly-empty list of signers for the current transaction (e.g., `account` in
        /// `main(&account: signer)`). Must match the number of signers expected by `script_file`.
        #[structopt(long = "signers")]
        signers: Vec<String>,
        /// Possibly-empty list of arguments passed to the transaction (e.g., `i` in
        /// `main(i: u64)`). Must match the arguments types expected by `script_file`.
        #[structopt(long = "args", parse(try_from_str = parser::parse_transaction_argument))]
        args: Vec<TransactionArgument>,
        /// Possibly-empty list of type arguments passed to the transaction (e.g., `T` in
        /// `main<T>()`). Must match the type arguments kinds expected by `script_file`.
        #[structopt(long = "type-args", parse(try_from_str = parser::parse_type_tag))]
        type_args: Vec<TypeTag>,
        /// If true, commit the effects of executing `script_file` (i.e., published, updated, and
        /// deleted resources) to disk.
        #[structopt(long = "commit", short = "c")]
        commit: bool,
    },
    /// View Move resources and modules stored on disk
    #[structopt(name = "view")]
    View {
        /// Path to a resource or module stored on disk.
        #[structopt(name = "file")]
        file: String,
    },
    /// Delete all modules and resources stored on disk under `move_data`. Does *not* delete
    /// anything in `move_src`.
    Clean {},
}

/// Store modules under move_src without an explicit `address {}` block under 0x2.
const MOVE_SRC: Address = Address::new([
    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8,
]);

/// Create a directory at ./`dir_name` if one does not already exist
fn maybe_create_dir(dir_name: &str) -> Result<&Path> {
    let dir = Path::new(dir_name);
    if !dir.exists() {
        fs::create_dir(dir)?
    }
    Ok(dir)
}

/// Compile the user modules in `move_src_directory` and the script in `script_file`
fn compile(
    args: &Move,
    script_file: &Option<String>,
) -> Result<(OnDiskStateView, Option<CompiledScript>)> {
    let move_src = maybe_create_dir(&args.move_src)?;
    let move_data = maybe_create_dir(&args.move_data)?;

    // allow user modules in move_src directory to compile against stdlib and move_data modules, but
    // not Libra framework modules
    // TODO: prevent cyclic dep?
    let mut user_lib_files: Vec<String> = fs::read_dir(move_src)?
        .map(|f| f.map_or("".to_string(), |f| f.path().to_string_lossy().into_owned()))
        .filter(|p| p.ends_with(".move"))
        .collect();

    if !user_lib_files.is_empty() {
        println!("Compiling {:?} user module(s)", user_lib_files.len());
    }
    if let Some(f) = script_file.as_ref() {
        user_lib_files.push(f.to_string())
    }

    // allow script to compile against both modules in `move_src` and Libra framework modules
    // modules
    let deps = stdlib::stdlib_files();
    let code_address = Some(MOVE_SRC);
    let (_, compilation_units) = move_lang::move_compile(&user_lib_files, &deps, code_address)?;

    let mut modules: Vec<CompiledModule> = stdlib::stdlib_bytecode_files()
        .iter()
        .map(|f| {
            let bytecode_bytes = fs::read(&f)?;
            CompiledModule::deserialize(&bytecode_bytes)
                .map_err(|e| anyhow!("Error deserializing module: {:?}", e))
        })
        .collect::<Result<Vec<CompiledModule>>>()?;
    let mut script_opt = None;
    for c in compilation_units {
        match c {
            CompiledUnit::Script { script, .. } => {
                if script_opt.is_some() {
                    bail!("Error: found script in move_lib")
                }
                script_opt = Some(script)
            }
            CompiledUnit::Module { module, .. } => modules.push(module),
        }
    }
    Ok((
        OnDiskStateView::create(move_data.to_path_buf(), &modules)?,
        script_opt,
    ))
}

fn run(
    args: &Move,
    script_file: &str,
    signers: &[String],
    txn_args: &[TransactionArgument],
    vm_type_args: Vec<TypeTag>,
    commit: bool,
) -> Result<()> {
    let (state, script_opt) = compile(args, &Some(script_file.to_string()))?;
    let mut script_bytes = vec![];
    if let Some(script) = script_opt {
        script.serialize(&mut script_bytes)?
    } else {
        bail!(
            "Script file {:?} contains module instead of script",
            script_file
        )
    }

    let vm = MoveVM::new();
    // TODO: use nonzero schedule and pick reasonable max default gas price
    let cost_schedule = gas_schedule::zero_cost_schedule();
    let mut cost_strategy = gas_schedule::CostStrategy::system(&cost_schedule, GasUnits::new(0));
    let signer_addresses = signers
        .iter()
        .map(|s| AccountAddress::from_hex_literal(&s))
        .collect::<Result<Vec<AccountAddress>>>()?;
    // TODO: parse Value's directly instead of going through the indirection of TransactionArgument?
    let vm_args: Vec<Value> = txn_args
        .iter()
        .map(|arg| match arg {
            TransactionArgument::U8(i) => Value::u8(*i),
            TransactionArgument::U64(i) => Value::u64(*i),
            TransactionArgument::U128(i) => Value::u128(*i),
            TransactionArgument::Address(a) => Value::address(*a),
            TransactionArgument::Bool(b) => Value::bool(*b),
            TransactionArgument::U8Vector(v) => Value::vector_u8(v.clone()),
        })
        .collect();

    let mut session = vm.new_session(&state);

    let res = session.execute_script(
        script_bytes,
        vm_type_args,
        vm_args,
        signer_addresses,
        &mut cost_strategy,
    );

    if let Err(err) = res {
        explain_error(err, &state)?
    } else {
        let effects = session.finish().map_err(|e| e.into_vm_status())?;
        explain_effects(&effects, &state)?;
        maybe_commit_effects(commit, Some(effects), &state)?;
    }

    Ok(())
}

fn explain_effects(effects: &TransactionEffects, state: &OnDiskStateView) -> Result<()> {
    // all module publishing happens via save_modules(), so effects shouldn't contain modules
    assert!(effects.modules.is_empty());
    if !effects.events.is_empty() {
        println!("Emitted {:?} events:", effects.events.len());
        // TODO: better event printing
        for (event_handle, event_sequence_number, _event_type, _event_layout, event_data) in
            &effects.events
        {
            println!(
                "Emitted {:?} as the {}th event to stream {:?}",
                event_data, event_sequence_number, event_handle
            )
        }
        // TODO: support saving events to disk
        println!("Warning: saving events to disk is currently not supported. Discarding events.");
    }
    if !effects.resources.is_empty() {
        println!(
            "Changed resource(s) under {:?} address(es):",
            effects.resources.len()
        );
    }
    for (addr, writes) in &effects.resources {
        print!("  ");
        println!(
            "Changed {:?} resource(s) under address {:?}:",
            writes.len(),
            addr
        );
        for (struct_tag, write_opt) in writes {
            print!("    ");
            match write_opt {
                Some((_layout, value)) => {
                    if state
                        .get_resource_bytes(*addr, struct_tag.clone())?
                        .is_some()
                    {
                        // TODO: print resource diff
                        println!("Changed type {}: {}", struct_tag, value)
                    } else {
                        // TODO: nicer printing
                        println!("Added type {}: {}", struct_tag, value)
                    }
                }
                None => println!("Deleted type {}", struct_tag),
            }
        }
    }
    Ok(())
}

/// Commit the resources and modules modified by a transaction to disk
fn maybe_commit_effects(
    commit: bool,
    effects_opt: Option<TransactionEffects>,
    state: &OnDiskStateView,
) -> Result<()> {
    if commit {
        if let Some(effects) = effects_opt {
            for (addr, writes) in effects.resources {
                for (struct_tag, write_opt) in writes {
                    match write_opt {
                        Some((layout, value)) => {
                            state.save_resource(addr, struct_tag, layout, value)?
                        }
                        None => state.delete_resource(addr, struct_tag)?,
                    }
                }
            }
        }
        // TODO: print modules to be saved?
        state.save_modules()?;
        println!("Committed changes.")
    } else if !effects_opt.map_or(false, |effects| effects.resources.is_empty()) {
        println!("Discarding changes; re-run with --commit if you would like to keep them.")
    }

    Ok(())
}

/// Explain an execution error
fn explain_error(error: VMError, state: &OnDiskStateView) -> Result<()> {
    match error.into_vm_status() {
        VMStatus::MoveAbort(AbortLocation::Module(id), abort_code) => {
            // try to use move-explain to explain the abort
            // TODO: this will only work for errors in the stdlib or Libra Framework. We should
            // add code to build an ErrorMapping for modules in move_lib as well
            let error_descriptions: ErrorMapping =
                lcs::from_bytes(compiled_stdlib::ERROR_DESCRIPTIONS)?;
            print!(
                "Execution aborted with code {} in module {}.",
                abort_code, id
            );

            if let Some(error_desc) = error_descriptions.get_explanation(&id, abort_code) {
                println!(
                        " Abort code details:\nReason:\n  Name: {}\n  Description:{}\nCategory:\n  Name: {}\n  Description:{}",
                        error_desc.reason.code_name,
                        error_desc.reason.code_description,
                        error_desc.category.code_name,
                        error_desc.category.code_description,
                    )
            } else {
                println!()
            }
        }
        VMStatus::MoveAbort(AbortLocation::Script, abort_code) => {
            // TODO: map to source code location
            println!(
                "Execution aborted with code {} in transaction script",
                abort_code
            )
        }
        VMStatus::ExecutionFailure {
            status_code,
            location,
            function,
            code_offset,
        } => {
            let status_explanation = match status_code {
                    StatusCode::RESOURCE_ALREADY_EXISTS => "resource already exists (i.e., move_to<T>(account) when there is already a value of type T under account)".to_string(),
                    StatusCode::MISSING_DATA => "resource does not exist (i.e., move_from<T>(a), borrow_global<T>(a), or borrow_global_mut<T>(a) when there is no value of type T at address a)".to_string(),
                    StatusCode::ARITHMETIC_ERROR => "arithmetic error (i.e., integer overflow, underflow, or divide-by-zero)".to_string(),
                    StatusCode::EXECUTION_STACK_OVERFLOW => "execution stack overflow".to_string(),
                    StatusCode::CALL_STACK_OVERFLOW => "call stack overflow".to_string(),
                    StatusCode::OUT_OF_GAS => "out of gas".to_string(),
                    _ => format!("{} error", status_code.status_type()),
                };
            // TODO: map to source code location
            let location_explanation = match location {
                AbortLocation::Module(id) => {
                    format!("{}::{}", id, state.resolve_function(&id, function)?)
                }
                AbortLocation::Script => "script".to_string(),
            };
            // TODO: code offset is 1-indexed, but disassembler instruction numbering starts at zero
            // This is potentially confusing to someone trying to understnd where something failed
            // by looking at a code offset + disassembled bytecode; we should fix it
            println!(
                "Execution failed with {} in {} at code offset {}",
                status_explanation, location_explanation, code_offset
            )
        }
        VMStatus::Error(status_code) => {
            println!("Execution failed with unexpected error {:?}", status_code)
        }
        VMStatus::Executed => unreachable!(),
    }
    Ok(())
}

/// Print a module or resource stored in `file`
fn view(args: &Move, file: &str) -> Result<()> {
    let move_data = maybe_create_dir(&args.move_data)?;
    let stdlib_modules = vec![]; // ok to use empty dir here since we're not compiling
    let state = OnDiskStateView::create(move_data.to_path_buf(), &stdlib_modules)?;
    if file.contains("::") {
        // TODO: less hacky way to detect this?
        // viewing resource
        match state.view_resource(Path::new(&file))? {
            Some(resource) => println!("{}", resource),
            None => println!("Resource not found."),
        }
    } else {
        // viewing module
        match state.view_module(Path::new(&file))? {
            Some(module) => println!("{}", module),
            None => println!("Module not found."),
        }
    }
    Ok(())
}

// TODO: add expected output tests
fn main() -> Result<()> {
    let move_args = Move::from_args();

    match &move_args.cmd {
        Command::Compile {
            script_file,
            commit,
        } => {
            let (state, _) = compile(&move_args, &script_file)?;
            maybe_commit_effects(*commit, None, &state)?;
            Ok(())
        }
        Command::Run {
            script_file,
            signers,
            args,
            type_args,
            commit,
        } => run(
            &move_args,
            script_file,
            signers,
            args,
            type_args.to_vec(),
            *commit,
        ),
        Command::View { file } => view(&move_args, file),
        Command::Clean {} => {
            let move_data = Path::new(&move_args.move_data);
            if move_data.exists() {
                fs::remove_dir_all(&move_data)?;
                fs::create_dir(&move_data)?;
            }
            Ok(())
        }
    }
}
