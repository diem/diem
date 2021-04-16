// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_cli::{
    commands,
    mode::{Mode, ModeType},
    test, DEFAULT_BUILD_DIR, DEFAULT_DEP_MODE, DEFAULT_SOURCE_DIR, DEFAULT_STORAGE_DIR,
};
use move_core_types::{
    language_storage::TypeTag, parser, transaction_argument::TransactionArgument,
};
use std::{fs, path::Path};
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "move",
    about = "CLI frontend for Move compiler and VM",
    rename_all = "kebab-case"
)]
pub struct Move {
    /// Directory storing Move resources, events, and module bytecodes produced by module publishing
    /// and script execution.
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
    )]
    mode: ModeType,
    /// Print additional diagnostics
    #[structopt(short = "v", global = true)]
    verbose: bool,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
pub enum Command {
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
    /// This command compiles the script first before running it.
    #[structopt(name = "run")]
    Run {
        /// Path to .mv file containing either script or module bytecodes. If the file is a module, the
        /// `script_name` parameter must be set.
        #[structopt(name = "script")]
        script_file: String,
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
        /// ASCII strings (e.g., 'b"hi" will parse as the vector<u8> value [68, 69])
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
        /// Create a new test directory scaffold with the specified <path>
        #[structopt(long = "create")]
        create: bool,
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
    /// Run well-formedness checks on the `storage` and `build` directories.
    #[structopt(name = "doctor")]
    Doctor {},
}

fn main() -> Result<()> {
    let move_args = Move::from_args();
    let mode = Mode::new(move_args.mode);

    match &move_args.cmd {
        Command::Check {
            source_files,
            no_republish,
        } => {
            let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
            commands::check(&state, !*no_republish, &source_files, move_args.verbose)
        }
        Command::Publish {
            source_files,
            no_republish,
            ignore_breaking_changes,
        } => {
            let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
            commands::publish(
                &state,
                source_files,
                !*no_republish,
                *ignore_breaking_changes,
                move_args.verbose,
            )
        }
        Command::Run {
            script_file,
            script_name,
            signers,
            args,
            type_args,
            gas_budget,
            dry_run,
        } => {
            let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
            commands::run(
                &state,
                script_file,
                script_name,
                signers,
                args,
                type_args.to_vec(),
                *gas_budget,
                *dry_run,
                move_args.verbose,
            )
        }
        Command::Test {
            path,
            track_cov: _,
            create: true,
        } => test::create_test_scaffold(path),
        Command::Test {
            path,
            track_cov,
            create: false,
        } => test::run_all(
            path,
            &std::env::current_exe()?.to_string_lossy(),
            *track_cov,
        ),
        Command::View { file } => {
            let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
            commands::view(&state, file)
        }
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
        Command::Doctor {} => {
            let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
            commands::doctor(&state)
        }
    }
}
