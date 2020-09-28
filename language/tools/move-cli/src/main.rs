// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use errmapgen::ErrorMapping;

use move_cli::*;
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasUnits},
    language_storage::TypeTag,
    parser,
    transaction_argument::TransactionArgument,
    vm_status::{AbortLocation, StatusCode, VMStatus},
};
use move_lang::{self, compiled_unit::CompiledUnit};
use move_vm_runtime::{data_cache::TransactionEffects, move_vm::MoveVM};
use move_vm_types::{gas_schedule, values::Value};
use vm::{
    access::ScriptAccess,
    errors::VMError,
    file_format::{CompiledModule, CompiledScript, SignatureToken},
};

use anyhow::{anyhow, bail, Result};
use move_vm_types::logger::StdErrLogger;
use std::{fs, path::Path};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "Move", about = "CLI frontend for Move compiler and VM")]
struct Move {
    /// Directory storing Move resources, events, and module bytecodes produced by script execution.
    #[structopt(name = "move-data", long = "move-data", default_value = MOVE_DATA)]
    move_data: String,
    /// Directory storing Move source files that will be compiled and loaded into the VM before
    /// script execution.
    #[structopt(name = "move-src", long = "move-src", default_value = MOVE_SRC)]
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
    /// Compile/run a Move script that reads/writes resources stored on disk in `move_data`.
    /// This command compiles each each module stored in `move_src` and loads it into the VM
    /// before running the script.
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
        /// Maximum number of gas units to be consumed by execution.
        /// When the budget is exhaused, execution will abort.
        /// By default, no `gas-budget` is specified and gas metering is disabled.
        #[structopt(long = "gas-budget", short = "g")]
        gas_budget: Option<u64>,
        /// If true, commit the effects of executing `script_file` (i.e., published, updated, and
        /// deleted resources) to disk.
        #[structopt(long = "commit", short = "c")]
        commit: bool,
    },

    /// Run expected value tests using the given batch file
    #[structopt(name = "test")]
    Test {
        // TODO: generalize this to support running all the tests in a given directory
        /// File containing batch of commands to run
        #[structopt(name = "file")]
        file: String,
    },
    /// View Move resources, events files, and modules stored on disk
    #[structopt(name = "view")]
    View {
        /// Path to a resource, events file, or module stored on disk.
        #[structopt(name = "file")]
        file: String,
    },
    /// Delete all resources, events, and modules stored on disk under `move_data`.
    /// Does *not* delete anything in `move_src`.
    Clean {},
}

/// Create a directory at ./`dir_name` if one does not already exist
fn maybe_create_dir(dir_name: &str) -> Result<&Path> {
    let dir = Path::new(dir_name);
    if !dir.exists() {
        fs::create_dir(dir)?
    }
    Ok(dir)
}

/// Compile the user modules in `move_src` and the script in `script_file`
fn compile(
    args: &Move,
    script_file: &Option<String>,
) -> Result<(OnDiskStateView, Option<CompiledScript>)> {
    let move_data = maybe_create_dir(&args.move_data)?;

    let mut user_move_src_files = if Path::new(&args.move_src).exists() {
        move_lang::find_move_filenames(&[args.move_src.clone()], true)?
    } else {
        vec![]
    };
    if let Some(f) = script_file.as_ref() {
        user_move_src_files.push(f.to_string())
    }

    let has_user_modules = !user_move_src_files.is_empty();
    if has_user_modules {
        println!("Compiling {:?} source file(s)", user_move_src_files.len());
    }

    let deps = stdlib::stdlib_files();
    let code_address = Some(MOVE_SRC_ADDRESS);
    let (_, compilation_units) =
        move_lang::move_compile(&user_move_src_files, &deps, code_address, None)?;

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
            CompiledUnit::Script { script, loc, .. } => {
                match script_file {
                    Some(f) => {
                        if Path::new(f).canonicalize()? == Path::new(loc.file()).canonicalize()? {
                            script_opt = Some(script)
                        }
                    }
                    None => (),
                }
                // TODO: save script bytecodes on disk? where should we put them?
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
    gas_budget: Option<u64>,
    commit: bool,
) -> Result<()> {
    let (state, script_opt) = compile(args, &Some(script_file.to_string()))?;
    let script = match script_opt {
        Some(s) => s,
        None => bail!(
            "Script file {:?} contains module instead of script",
            script_file
        ),
    };
    let mut script_bytes = vec![];
    script.serialize(&mut script_bytes)?;

    let vm = MoveVM::new();
    let gas_schedule = &vm_genesis::genesis_gas_schedule::INITIAL_GAS_SCHEDULE;
    let mut cost_strategy = if let Some(gas_budget) = gas_budget {
        let max_gas_budget = u64::MAX / gas_schedule.gas_constants.gas_unit_scaling_factor;
        if gas_budget >= max_gas_budget {
            bail!("Gas budget set too high; maximum is {}", max_gas_budget)
        }
        gas_schedule::CostStrategy::transaction(gas_schedule, GasUnits::new(gas_budget))
    } else {
        // no budget specified. use CostStrategy::system, which disables gas metering
        gas_schedule::CostStrategy::system(gas_schedule, GasUnits::new(0))
    };

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

    let logger = StdErrLogger;
    let mut session = vm.new_session(&state);

    let res = session.execute_script(
        script_bytes,
        vm_type_args.clone(),
        vm_args,
        signer_addresses.clone(),
        &mut cost_strategy,
        &logger,
    );

    if let Err(err) = res {
        explain_error(
            err,
            &state,
            &script,
            &vm_type_args,
            &signer_addresses,
            txn_args,
        )
    } else {
        let effects = session.finish().map_err(|e| e.into_vm_status())?;
        explain_effects(&effects, &state)?;
        maybe_commit_effects(commit, Some(effects), &state)
    }
}

fn explain_effects(effects: &TransactionEffects, state: &OnDiskStateView) -> Result<()> {
    // all module publishing happens via save_modules(), so effects shouldn't contain modules
    assert!(effects.modules.is_empty());
    if !effects.events.is_empty() {
        println!("Emitted {:?} events:", effects.events.len());
        // TODO: better event printing
        for (event_key, event_sequence_number, _event_type, _event_layout, event_data) in
            &effects.events
        {
            println!(
                "Emitted {:?} as the {}th event to stream {:?}",
                event_data, event_sequence_number, event_key
            )
        }
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

            for (event_key, event_sequence_number, event_type, event_layout, event_data) in
                effects.events
            {
                state.save_event(
                    &event_key,
                    event_sequence_number,
                    event_type,
                    &event_layout,
                    event_data,
                )?
            }
        }

        // TODO: print modules to be saved?
        state.save_modules()?;
        println!("Committed changes.")
    } else if !effects_opt.map_or(true, |effects| effects.resources.is_empty()) {
        println!("Discarding changes; re-run with --commit if you would like to keep them.")
    }

    Ok(())
}

fn explain_type_error(
    script: &CompiledScript,
    signers: &[AccountAddress],
    txn_args: &[TransactionArgument],
) {
    use SignatureToken::*;
    let script_params = script.signature_at(script.as_inner().parameters);
    let expected_num_signers = script_params
        .0
        .iter()
        .filter(|t| match t {
            Reference(r) => r.is_signer(),
            _ => false,
        })
        .count();
    if expected_num_signers != signers.len() {
        println!(
            "Execution failed with incorrect number of signers: script expected {:?}, but found {:?}",
            expected_num_signers,
            signers.len()
        );
        return;
    }

    // TODO: printing type(s) of missing arguments could be useful
    let expected_num_args = script_params.len() - signers.len();
    if expected_num_args != txn_args.len() {
        println!(
            "Execution failed with incorrect number of arguments: script expected {:?}, but found {:?}",
	    expected_num_args,
            txn_args.len()
        );
        return;
    }

    // TODO: print more helpful error message pinpointing the (argument, type)
    // pair that didn't match
    println!("Execution failed with type error when binding type arguments to type parameters")
}

/// Explain an execution error
fn explain_error(
    error: VMError,
    state: &OnDiskStateView,
    script: &CompiledScript,
    vm_type_args: &[TypeTag],
    signers: &[AccountAddress],
    txn_args: &[TransactionArgument],
) -> Result<()> {
    use StatusCode::*;
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
                    RESOURCE_ALREADY_EXISTS => "a RESOURCE_ALREADY_EXISTS error (i.e., `move_to<T>(account)` when there is already a resource of type `T` under `account`)".to_string(),
                    MISSING_DATA => "a RESOURCE_DOES_NOT_EXIST error (i.e., `move_from<T>(a)`, `borrow_global<T>(a)`, or `borrow_global_mut<T>(a)` when there is no resource of type `T` at address `a`)".to_string(),
                    ARITHMETIC_ERROR => "an arithmetic error (i.e., integer overflow, underflow, or divide-by-zero)".to_string(),
                    EXECUTION_STACK_OVERFLOW => "an execution stack overflow".to_string(),
                    CALL_STACK_OVERFLOW => "a call stack overflow".to_string(),
                    OUT_OF_GAS => "an out of gas error".to_string(),
                    _ => format!("a {} error", status_code.status_type()),
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
                "Execution failed because of {} in {} at code offset {}",
                status_explanation, location_explanation, code_offset
            )
        }
        VMStatus::Error(NUMBER_OF_TYPE_ARGUMENTS_MISMATCH) => {
	    println!("Execution failed with incorrect number of type arguments: script expected {:?}, but found {:?}", &script.as_inner().type_parameters.len(), vm_type_args.len())
	}
        VMStatus::Error(TYPE_MISMATCH) => {
	    explain_type_error(script, signers, txn_args)
        }
	VMStatus::Error(LINKER_ERROR) => {
	    // TODO: is this the only reason we can see LINKER_ERROR? Can we also see it if someone manually deletes modules in move_data?
	    println!("Execution failed due to unresolved type argument(s) (i.e., `--type-args 0x1::M:T` when there is no module named M at 0x1 or no type named T in module 0x1::M)");
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
    let move_data = maybe_create_dir(&args.move_data)?.canonicalize()?;
    let stdlib_modules = vec![]; // ok to use empty dir here since we're not compiling
    let state = OnDiskStateView::create(move_data, &stdlib_modules)?;

    let path = Path::new(&file);
    if state.is_resource_path(path) {
        match state.view_resource(path)? {
            Some(resource) => println!("{}", resource),
            None => println!("Resource not found."),
        }
    } else if state.is_event_path(path) {
        let events = state.view_events(path)?;
        if events.is_empty() {
            println!("Events not found.")
        } else {
            for event in events {
                println!("{}", event)
            }
        }
    } else if state.is_module_path(path) {
        match state.view_module(path)? {
            Some(module) => println!("{}", module),
            None => println!("Module not found."),
        }
    } else {
        bail!("`move view <file>` must point to a valid file under move_data")
    }
    Ok(())
}

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
            gas_budget,
            commit,
        } => run(
            &move_args,
            script_file,
            signers,
            args,
            type_args.to_vec(),
            *gas_budget,
            *commit,
        ),
        Command::Test { file } => test::run_one(
            &Path::new(file),
            &std::env::current_exe()?.to_string_lossy(),
        ),
        Command::View { file } => view(&move_args, file),
        Command::Clean {} => {
            // delete move_data
            let move_data = Path::new(&move_args.move_data);
            if move_data.exists() {
                fs::remove_dir_all(&move_data)?;
            }
            Ok(())
        }
    }
}
