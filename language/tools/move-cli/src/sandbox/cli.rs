// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    base, sandbox, sandbox::utils::Mode, shared, Move, NativeFunctionRecord, DEFAULT_SOURCE_DIR,
};
use anyhow::Result;
use move_core_types::{
    errmap::ErrorMapping, language_storage::TypeTag, parser,
    transaction_argument::TransactionArgument,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

use super::utils::on_disk_state_view::OnDiskStateView;

#[derive(StructOpt)]
pub enum SandboxCommand {
    /// Compile the specified modules and publish the resulting bytecodes in global storage.
    #[structopt(name = "publish")]
    Publish {
        /// The source files containing modules to publish.
        #[structopt(
            name = "PATH_TO_SOURCE_FILE",
            default_value = DEFAULT_SOURCE_DIR,
        )]
        source_files: Vec<String>,
        /// If set, fail during compilation when attempting to publish a module that already
        /// exists in global storage.
        #[structopt(long = "no-republish")]
        no_republish: bool,
        /// By default, code that might cause breaking changes for bytecode
        /// linking or data layout compatibility checks will not be published.
        /// Set this flag to ignore breaking changes checks and publish anyway.
        #[structopt(long = "ignore-breaking-changes")]
        ignore_breaking_changes: bool,
        /// Manually specify the publishing order of modules.
        #[structopt(long = "override-ordering")]
        override_ordering: Option<Vec<String>>,
    },
    /// Compile/run a Move script that reads/writes resources stored on disk in `storage`.
    /// This command compiles the script first before running it.
    #[structopt(name = "run")]
    Run {
        /// Path to .mv file containing either script or module bytecodes. If the file is a module, the
        /// `script_name` parameter must be set.
        #[structopt(name = "script", parse(from_os_str))]
        script_file: PathBuf,
        /// Name of the script function inside `script_file` to call. Should only be set if `script_file`
        /// points to a module.
        #[structopt(name = "name")]
        script_name: Option<String>,
        /// Possibly-empty list of signers for the current transaction (e.g., `account` in
        /// `main(&account: signer)`). Must match the number of signers expected by `script_file`.
        #[structopt(long = "signers")]
        signers: Vec<String>,
        /// Possibly-empty list of arguments passed to the transaction (e.g., `i` in
        /// `main(i: u64)`). Must match the arguments types expected by `script_file`.
        /// Supported argument types are
        /// bool literals (true, false),
        /// u64 literals (e.g., 10, 58),
        /// address literals (e.g., 0x12, 0x0000000000000000000000000000000f),
        /// hexadecimal strings (e.g., x"0012" will parse as the vector<u8> value [00, 12]), and
        /// ASCII strings (e.g., 'b"hi" will parse as the vector<u8> value [68, 69]).
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
    /// Run expected value tests using the given batch file.
    #[structopt(name = "test")]
    Test {
        /// a directory path in which all the tests will be executed.
        #[structopt(name = "path", parse(from_os_str))]
        path: PathBuf,
        /// Use an ephemeral directory to serve as the testing workspace.
        /// By default, the directory containing the `args.txt` will be the workspace.
        #[structopt(long = "use-temp-dir")]
        use_temp_dir: bool,
        /// Show coverage information after tests are done.
        /// By default, coverage will not be tracked nor shown.
        #[structopt(long = "track-cov")]
        track_cov: bool,
        /// Create a new test directory scaffold with the specified <path>.
        #[structopt(long = "create")]
        create: bool,
    },
    /// View Move resources, events files, and modules stored on disk.
    #[structopt(name = "view")]
    View {
        /// Path to a resource, events file, or module stored on disk.
        #[structopt(name = "file", parse(from_os_str))]
        file: PathBuf,
    },
    /// Delete all resources, events, and modules stored on disk under `storage`.
    /// Does *not* delete anything in `src`.
    Clean {},
    /// Run well-formedness checks on the `storage` and `build` directories.
    #[structopt(name = "doctor")]
    Doctor {},
    /// Typecheck and verify the scripts and/or modules under `src`.
    #[structopt(name = "link")]
    Link {
        /// The source files containing modules to publish.
        #[structopt(
            name = "PATH_TO_SOURCE_FILE",
            default_value = DEFAULT_SOURCE_DIR,
        )]
        source_files: Vec<String>,

        /// If set, fail when attempting to typecheck a module that already exists in global storage.
        #[structopt(long = "no-republish")]
        no_republish: bool,
    },
    /// Generate struct layout bindings for the modules stored on disk under `storage`
    // TODO: expand this to generate script bindings, docs, errmaps, etc.?.
    #[structopt(name = "generate")]
    Generate {
        #[structopt(subcommand)]
        cmd: GenerateCommand,
    },
}

#[derive(StructOpt)]
pub enum GenerateCommand {
    /// Generate struct layout bindings for the modules stored on disk under `storage`.
    #[structopt(name = "struct-layouts")]
    StructLayouts {
        /// Path to a module stored on disk.
        #[structopt(long, parse(from_os_str))]
        module: PathBuf,
        /// If set, generate bindings for the specified struct and type arguments. If unset,
        /// generate bindings for all closed struct definitions.
        #[structopt(flatten)]
        options: StructLayoutOptions,
    },
}
#[derive(StructOpt)]
pub struct StructLayoutOptions {
    /// Generate layout bindings for this struct.
    #[structopt(long = "struct")]
    struct_: Option<String>,
    /// Generate layout bindings for `struct` bound to these type arguments.
    #[structopt(long = "type-args", parse(try_from_str = parser::parse_type_tag), requires="struct")]
    type_args: Option<Vec<TypeTag>>,
}

impl SandboxCommand {
    pub fn handle_command(
        &self,
        natives: Vec<NativeFunctionRecord>,
        error_descriptions: &ErrorMapping,
        move_args: &Move,
        mode: &Mode,
    ) -> Result<()> {
        let additional_named_addresses =
            shared::verify_and_create_named_address_mapping(move_args.named_addresses.clone())?;
        match self {
            SandboxCommand::Link {
                source_files,
                no_republish,
            } => {
                let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
                base::commands::check(
                    &[state.interface_files_dir()?],
                    !*no_republish,
                    source_files,
                    state.get_named_addresses(additional_named_addresses)?,
                    move_args.verbose,
                )
            }
            SandboxCommand::Publish {
                source_files,
                no_republish,
                ignore_breaking_changes,
                override_ordering,
            } => {
                let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
                sandbox::commands::publish(
                    natives,
                    &state,
                    source_files,
                    !*no_republish,
                    *ignore_breaking_changes,
                    override_ordering.as_ref().map(|o| o.as_slice()),
                    state.get_named_addresses(additional_named_addresses)?,
                    move_args.verbose,
                )
            }
            SandboxCommand::Run {
                script_file,
                script_name,
                signers,
                args,
                type_args,
                gas_budget,
                dry_run,
            } => {
                let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
                sandbox::commands::run(
                    natives,
                    error_descriptions,
                    &state,
                    script_file,
                    script_name,
                    signers,
                    args,
                    type_args.to_vec(),
                    state.get_named_addresses(additional_named_addresses)?,
                    *gas_budget,
                    *dry_run,
                    move_args.verbose,
                )
            }
            SandboxCommand::Test {
                path,
                use_temp_dir: _,
                track_cov: _,
                create: true,
            } => sandbox::commands::create_test_scaffold(path),
            SandboxCommand::Test {
                path,
                use_temp_dir,
                track_cov,
                create: false,
            } => sandbox::commands::run_all(
                path,
                &std::env::current_exe()?,
                *use_temp_dir,
                *track_cov,
            ),
            SandboxCommand::View { file } => {
                let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
                sandbox::commands::view(&state, file)
            }
            SandboxCommand::Clean {} => {
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
            SandboxCommand::Doctor {} => {
                let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
                sandbox::commands::doctor(&state)
            }
            SandboxCommand::Generate { cmd } => {
                let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
                handle_generate_commands(cmd, &state)
            }
        }
    }
}

fn handle_generate_commands(cmd: &GenerateCommand, state: &OnDiskStateView) -> Result<()> {
    match cmd {
        GenerateCommand::StructLayouts { module, options } => {
            sandbox::commands::generate::generate_struct_layouts(
                module,
                &options.struct_,
                &options.type_args,
                state,
            )
        }
    }
}
