// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use errmapgen::ErrorMapping;

use move_cli::{
    package::{parse_mode_from_string, Mode},
    *,
};
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{GasAlgebra, GasUnits},
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    parser,
    transaction_argument::TransactionArgument,
    vm_status::{AbortLocation, StatusCode, VMStatus},
};
use move_lang::{
    self, compiled_unit::CompiledUnit, Pass as MovePass, PassResult as MovePassResult,
    MOVE_COMPILED_INTERFACES_DIR,
};
use move_vm_runtime::{data_cache::TransactionEffects, logging::NoContextLog, move_vm::MoveVM};
use move_vm_types::{gas_schedule, values::Value};
use vm::{
    access::ScriptAccess,
    compatibility::Compatibility,
    errors::VMError,
    file_format::{CompiledModule, CompiledScript, SignatureToken},
    normalized::Module,
};

use anyhow::{bail, Result};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "move",
    about = "CLI frontend for Move compiler and VM",
    rename_all = "kebab-case"
)]
struct Move {
    /// Directory storing Move resources, events, and module bytecodes produced by script execution.
    #[structopt(long, default_value = DEFAULT_STORAGE_DIR, global = true)]
    storage_dir: String,
    /// Directory storing build artifacts produced by compilation
    #[structopt(long, short = "d", default_value = DEFAULT_BUILD_DIR, global = true)]
    build_dir: String,
    /// Dependency inclusion mode
    #[structopt(
        long,
        default_value = DEFAULT_DEP_MODE,
        global = true,
        parse(try_from_str = parse_mode_from_string),
    )]
    mode: Mode,
    /// Print additional diagnostics
    #[structopt(short = "v", global = true)]
    verbose: bool,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Type check and verify the specified script and modules against the modules in `storage`
    #[structopt(name = "check")]
    Check {
        /// The source files to check
        #[structopt(
            name = "PATH_TO_SOURCE_FILE",
            default_value = DEFAULT_SOURCE_DIR,
        )]
        source_files: Vec<String>,
        /// If set, fail when attempting to typecheck a module that already exists in global storage
        #[structopt(long = "no-republish")]
        no_republish: bool,
    },
    /// Compile the specified modules and publish the resulting bytecodes in global storage
    #[structopt(name = "publish")]
    Publish {
        /// The source files containing modules to publish
        #[structopt(
            name = "PATH_TO_SOURCE_FILE",
            default_value = DEFAULT_SOURCE_DIR,
        )]
        source_files: Vec<String>,
        /// If set, fail during compilation when attempting to publish a module that already
        /// exists in global storage
        #[structopt(long = "no-republish")]
        no_republish: bool,
        /// By default, code that might cause breaking changes for bytecode
        /// linking or data layout compatibility checks will not be published.
        /// Set this flag to ignore breaking changes checks and publish anyway
        #[structopt(long = "ignore-breaking-changes")]
        ignore_breaking_changes: bool,
    },
    /// Compile/run a Move script that reads/writes resources stored on disk in `storage`.
    /// This command compiles each each module stored in `src` and loads it into the VM
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
        /// If set, the effects of executing `script_file` (i.e., published, updated, and
        /// deleted resources) will NOT be committed to disk.
        #[structopt(long = "dry-run", short = "n")]
        dry_run: bool,
    },

    /// Run expected value tests using the given batch file
    #[structopt(name = "test")]
    Test {
        /// a directory path in which all the tests will be executed
        #[structopt(name = "path")]
        path: String,
        /// Show coverage information after tests are done.
        /// By default, coverage will not be tracked nor shown.
        #[structopt(long = "track-cov")]
        track_cov: bool,
    },
    /// View Move resources, events files, and modules stored on disk
    #[structopt(name = "view")]
    View {
        /// Path to a resource, events file, or module stored on disk.
        #[structopt(name = "file")]
        file: String,
    },
    /// Delete all resources, events, and modules stored on disk under `storage`.
    /// Does *not* delete anything in `src`.
    Clean {},
}

impl Move {
    fn get_package_dir(&self) -> PathBuf {
        Path::new(&self.build_dir).join(DEFAULT_PACKAGE_DIR)
    }

    /// Prepare the library dependencies, need to run it before every related command,
    /// i.e., check, publish, and run.
    ///
    /// If `source_only` is true, only the source files will be populated. The modules will
    /// not be compiled nor loaded.
    ///
    /// Currently, `source_only` is set to true for "check" and "publish" and false for "run"
    fn prepare_mode(&self, source_only: bool) -> Result<()> {
        self.mode.prepare(&self.get_package_dir(), source_only)
    }

    /// This collect the dependencies for compiling a script or module. The dependencies
    /// include not only the loaded libraries, but also the interface files generated from
    /// prior "publish" commands.
    fn get_compilation_deps(&self) -> Result<Vec<String>> {
        let mut src_dirs = self.mode.source_files(&self.get_package_dir())?;
        src_dirs.push(interface_files_dir(&self.build_dir)?);
        Ok(src_dirs)
    }

    /// This collects only the compiled modules from dependent libraries. The modules
    /// created via the "publish" command should already sit in the storage based on
    /// current implementation.
    fn get_library_modules(&self) -> Result<Vec<CompiledModule>> {
        self.mode.compiled_modules(&self.get_package_dir())
    }
}

/// Create a directory at ./`dir_name` if one does not already exist
fn maybe_create_dir(dir_name: &str) -> Result<&Path> {
    let dir = Path::new(dir_name);
    if !dir.exists() {
        fs::create_dir_all(dir)?
    }
    Ok(dir)
}

/// Generate interface files for published files
fn generate_interface_files(args: &Move) -> Result<()> {
    move_lang::generate_interface_files(
        &[args.storage_dir.clone()],
        Some(args.build_dir.clone()),
        false,
    )?;
    Ok(())
}

fn interface_files_dir(build_dir: &str) -> Result<String> {
    let mut path = PathBuf::from(build_dir);
    path.push(MOVE_COMPILED_INTERFACES_DIR);
    let dir = path.into_os_string().into_string().unwrap();
    maybe_create_dir(&dir)?;
    Ok(dir)
}

fn shadow_storage(
    args: &Move,
    pprog: move_lang::parser::ast::Program,
) -> Result<move_lang::parser::ast::Program> {
    fn convert_module_id(
        address: &move_lang::shared::Address,
        name: &move_lang::parser::ast::ModuleName,
    ) -> ModuleId {
        use move_lang::shared::Identifier as MoveIdentifier;
        ModuleId::new(
            AccountAddress::new(address.to_u8()),
            Identifier::new(name.value().to_string()).unwrap(),
        )
    }
    use move_lang::parser::ast::{Definition, Program};
    let Program {
        source_definitions,
        lib_definitions,
    } = pprog;
    let mut interface_modules_to_remove = BTreeSet::new();
    for def in &source_definitions {
        match def {
            Definition::Address(_, addr, modules) => {
                for module in modules {
                    interface_modules_to_remove.insert(convert_module_id(addr, &module.name));
                }
            }
            Definition::Module(_) | Definition::Script(_) => (),
        }
    }

    let interface_dir = interface_files_dir(&args.build_dir)?;
    let interface_files_to_ignore = move_lang::find_filenames(&[interface_dir], |path| {
        let module_str = path.file_stem().unwrap().to_str().unwrap();
        let module = Identifier::new(module_str.to_string()).unwrap();

        let mut comps = path.components().rev();
        let _file = comps.next().unwrap();
        let addr_str = comps.next().unwrap().as_os_str().to_str().unwrap();
        let addr = AccountAddress::from_str(addr_str).unwrap();

        let id = ModuleId::new(addr, module);
        interface_modules_to_remove.contains(&id)
    })?
    .into_iter()
    .collect::<BTreeSet<_>>();
    let lib_definitions = lib_definitions
        .into_iter()
        .filter(|def| !interface_files_to_ignore.contains(def.file()))
        .collect();
    Ok(Program {
        source_definitions,
        lib_definitions,
    })
}

fn move_compile_to_and_shadow(
    args: &Move,
    files: &[String],
    republish: bool,
    until: MovePass,
) -> Result<(
    move_lang::errors::FilesSourceText,
    std::result::Result<MovePassResult, move_lang::errors::Errors>,
)> {
    let (files, pprog_and_comments_res) =
        move_lang::move_parse(files, &args.get_compilation_deps()?, None, None)?;
    let (_comments, sender_opt, mut pprog) = match pprog_and_comments_res {
        Err(errors) => return Ok((files, Err(errors))),
        Ok(res) => res,
    };
    assert!(sender_opt.is_none());
    if republish {
        pprog = shadow_storage(args, pprog)?;
    }
    Ok((
        files,
        move_lang::move_continue_up_to(MovePassResult::Parser(None, pprog), until),
    ))
}

/// Compile the user modules in `src` and the script in `script_file`
fn check(args: &Move, republish: bool, files: &[String]) -> Result<()> {
    if args.verbose {
        println!("Checking Move files...");
    }
    let (files, result) = move_compile_to_and_shadow(args, files, republish, MovePass::CFGIR)?;
    move_lang::unwrap_or_report_errors!(files, result);
    Ok(())
}

fn publish(
    args: &Move,
    files: &[String],
    republish: bool,
    ignore_breaking_changes: bool,
) -> Result<OnDiskStateView> {
    let storage_dir = maybe_create_dir(&args.storage_dir)?;

    if args.verbose {
        println!("Compiling Move modules...")
    }

    let (files, result) =
        move_compile_to_and_shadow(args, files, republish, MovePass::Compilation)?;
    let compiled_units = match move_lang::unwrap_or_report_errors!(files, result) {
        MovePassResult::Compilation(units) => units,
        _ => unreachable!(),
    };

    let num_modules = compiled_units
        .iter()
        .filter(|u| matches!(u,  CompiledUnit::Module {..}))
        .count();
    if args.verbose {
        println!("Found and compiled {} modules", num_modules)
    }

    let mut modules = vec![];
    for c in compiled_units {
        match c {
            CompiledUnit::Script { loc, .. } => {
                if args.verbose {
                    println!(
                        "Warning: Found script in specified files for publishing. But scripts \
                         cannot be published. Script found in: {}",
                        loc.file()
                    )
                }
            }
            CompiledUnit::Module { module, .. } => modules.push(module),
        }
    }

    if !ignore_breaking_changes {
        let view = OnDiskStateView::create(storage_dir.to_path_buf(), &[])?;
        for m in &modules {
            let id = m.self_id();
            if let Ok(old_m) = view.get_compiled_module(&m.self_id()) {
                let old_api = Module::new(&old_m);
                let new_api = Module::new(m);
                let compat = Compatibility::check(&old_api, &new_api);
                if !compat.is_fully_compatible() {
                    eprintln!("Breaking change detected--publishing aborted. Re-run with --ignore-breaking-changes to publish anyway.")
                }
                if !compat.struct_layout {
                    // TODO: we could choose to make this more precise by walking the global state and looking for published
                    // structs of this type. but probably a bad idea
                    bail!("Layout API for structs of module {} has changed. Need to do a data migration of published structs", id)
                }
                if !compat.struct_and_function_linking {
                    // TODO: this will report false positives if we *are* simultaneously redeploying all dependent modules.
                    // but this is not easy to check without walking the global state and looking for everything
                    bail!("Linking API for structs/functions of module {} has changed. Need to redeploy all dependent modules.", id)
                }
            }
        }
    }

    OnDiskStateView::create(storage_dir.to_path_buf(), &modules)
}

fn run(
    args: &Move,
    script_file: &str,
    signers: &[String],
    txn_args: &[TransactionArgument],
    vm_type_args: Vec<TypeTag>,
    gas_budget: Option<u64>,
    dry_run: bool,
) -> Result<()> {
    fn compile_script(
        args: &Move,
        script_file: &str,
    ) -> Result<(OnDiskStateView, Option<CompiledScript>)> {
        let storage_dir = maybe_create_dir(&args.storage_dir)?;

        if args.verbose {
            println!("Compiling transaction script...")
        }
        let (_files, compiled_units) = move_lang::move_compile_and_report(
            &[script_file.to_string()],
            &args.get_compilation_deps()?,
            None,
            None,
        )?;

        let mut script_opt = None;
        for c in compiled_units {
            match c {
                CompiledUnit::Script { script, .. } => {
                    if script_opt.is_some() {
                        bail!("Error: Found more than one script")
                    }
                    script_opt = Some(script)
                }
                CompiledUnit::Module { ident, .. } => {
                    if args.verbose {
                        println!(
                            "Warning: Found module '{}' in file specified for the script. This \
                             module will not be published.",
                            ident
                        )
                    }
                }
            }
        }

        // TODO: run `move publish` here instead?
        // preload the modules to the storage
        let state =
            OnDiskStateView::create(storage_dir.to_path_buf(), &args.get_library_modules()?)?;
        state.save_modules()?;
        Ok((
            OnDiskStateView::create(storage_dir.to_path_buf(), &[])?,
            script_opt,
        ))
    }

    let (state, script_opt) = compile_script(args, script_file)?;
    let script = match script_opt {
        Some(s) => s,
        None => bail!("Unable to find script in file {:?}", script_file),
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

    let log_context = NoContextLog::new();
    let mut session = vm.new_session(&state);

    let res = session.execute_script(
        script_bytes,
        vm_type_args.clone(),
        vm_args,
        signer_addresses.clone(),
        &mut cost_strategy,
        &log_context,
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
        if args.verbose {
            explain_effects(&effects, &state)?
        }
        maybe_commit_effects(&args, !dry_run, Some(effects), &state)
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
    args: &Move,
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

        let modules_saved = state.save_modules()?;
        if modules_saved {
            generate_interface_files(args)?;
        }
        state.save_modules()?;
    } else if !effects_opt.map_or(true, |effects| effects.resources.is_empty()) {
        println!("Discarding changes; re-run without --dry-run if you would like to keep them.")
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
            "Execution failed with incorrect number of signers: script expected {:?}, but found \
             {:?}",
            expected_num_signers,
            signers.len()
        );
        return;
    }

    // TODO: printing type(s) of missing arguments could be useful
    let expected_num_args = script_params.len() - signers.len();
    if expected_num_args != txn_args.len() {
        println!(
            "Execution failed with incorrect number of arguments: script expected {:?}, but found \
             {:?}",
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
            // TODO: this will only work for errors in the stdlib or Diem Framework. We should
            // add code to build an ErrorMapping for modules in move_lib as well
            let error_descriptions: ErrorMapping =
                bcs::from_bytes(compiled_stdlib::ERROR_DESCRIPTIONS)?;
            print!(
                "Execution aborted with code {} in module {}.",
                abort_code, id
            );

            if let Some(error_desc) = error_descriptions.get_explanation(&id, abort_code) {
                println!(
                    " Abort code details:\nReason:\n  Name: {}\n  Description:{}\nCategory:\n  \
                     Name: {}\n  Description:{}",
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
                RESOURCE_ALREADY_EXISTS => "a RESOURCE_ALREADY_EXISTS error (i.e., \
                                            `move_to<T>(account)` when there is already a \
                                            resource of type `T` under `account`)"
                    .to_string(),
                MISSING_DATA => "a RESOURCE_DOES_NOT_EXIST error (i.e., `move_from<T>(a)`, \
                                 `borrow_global<T>(a)`, or `borrow_global_mut<T>(a)` when there \
                                 is no resource of type `T` at address `a`)"
                    .to_string(),
                ARITHMETIC_ERROR => "an arithmetic error (i.e., integer overflow/underflow, \
                                     div/mod by zero, or invalid shift)"
                    .to_string(),
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
            println!(
                "Execution failed because of {} in {} at code offset {}",
                status_explanation, location_explanation, code_offset
            )
        }
        VMStatus::Error(NUMBER_OF_TYPE_ARGUMENTS_MISMATCH) => println!(
            "Execution failed with incorrect number of type arguments: script expected {:?}, but \
             found {:?}",
            &script.as_inner().type_parameters.len(),
            vm_type_args.len()
        ),
        VMStatus::Error(TYPE_MISMATCH) => explain_type_error(script, signers, txn_args),
        VMStatus::Error(LINKER_ERROR) => {
            // TODO: is this the only reason we can see LINKER_ERROR?
            // Can we also see it if someone manually deletes modules in storage?
            println!(
                "Execution failed due to unresolved type argument(s) (i.e., `--type-args \
                 0x1::M:T` when there is no module named M at 0x1 or no type named T in module \
                 0x1::M)"
            );
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
    let storage_dir = maybe_create_dir(&args.storage_dir)?.canonicalize()?;
    let stdlib_modules = vec![]; // ok to use empty dir here since we're not compiling
    let state = OnDiskStateView::create(storage_dir, &stdlib_modules)?;

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
        bail!("`move view <file>` must point to a valid file under storage")
    }
    Ok(())
}

fn main() -> Result<()> {
    let move_args = Move::from_args();

    match &move_args.cmd {
        Command::Check {
            source_files,
            no_republish,
        } => {
            move_args.prepare_mode(true)?;
            check(&move_args, !*no_republish, &source_files)
        }
        Command::Publish {
            source_files,
            no_republish,
            ignore_breaking_changes,
        } => {
            move_args.prepare_mode(true)?;
            let state = publish(
                &move_args,
                source_files,
                !*no_republish,
                *ignore_breaking_changes,
            )?;
            maybe_commit_effects(&move_args, /* commit = */ true, None, &state)
        }
        Command::Run {
            script_file,
            signers,
            args,
            type_args,
            gas_budget,
            dry_run,
        } => {
            move_args.prepare_mode(false)?;
            run(
                &move_args,
                script_file,
                signers,
                args,
                type_args.to_vec(),
                *gas_budget,
                *dry_run,
            )
        }
        Command::Test { path, track_cov } => test::run_all(
            path,
            &std::env::current_exe()?.to_string_lossy(),
            *track_cov,
        ),
        Command::View { file } => view(&move_args, file),
        Command::Clean {} => {
            // delete storage
            let storage_dir = Path::new(&move_args.storage_dir);
            if storage_dir.exists() {
                fs::remove_dir_all(&storage_dir)?;
            }

            // delete build
            let build_dir = Path::new(&move_args.build_dir);
            if build_dir.exists() {
                fs::remove_dir_all(&build_dir)?;
            }
            Ok(())
        }
    }
}
