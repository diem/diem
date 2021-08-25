// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod base;
pub mod experimental;
pub mod package;
pub mod sandbox;

/// Default directory where saved Move resources live
pub const DEFAULT_STORAGE_DIR: &str = "storage";

/// Default directory where Move modules live
pub const DEFAULT_SOURCE_DIR: &str = "src";

/// Default directory where Move packages live under build_dir
pub const DEFAULT_PACKAGE_DIR: &str = "package";

/// Default dependency inclusion mode
pub const DEFAULT_DEP_MODE: &str = "stdlib";

/// Default directory for build output
pub use move_lang::command_line::DEFAULT_OUTPUT_DIR as DEFAULT_BUILD_DIR;

/// Extension for resource and event files, which are in BCS format
const BCS_EXTENSION: &str = "bcs";

use anyhow::Result;
use move_core_types::{
    account_address::AccountAddress, errmap::ErrorMapping, identifier::Identifier,
};
use move_lang::shared::{self, AddressBytes};
use move_vm_runtime::native_functions::NativeFunction;
use sandbox::utils::mode::{Mode, ModeType};
use std::path::PathBuf;
use structopt::StructOpt;

type NativeFunctionRecord = (AccountAddress, Identifier, Identifier, NativeFunction);

#[derive(StructOpt)]
#[structopt(
    name = "move",
    about = "CLI frontend for Move compiler and VM",
    rename_all = "kebab-case"
)]
pub struct Move {
    /// Named address mapping.
    #[structopt(
        name = "NAMED_ADDRESSES",
        short = "a",
        long = "addresses",
        global = true,
        parse(try_from_str = shared::parse_named_address)
    )]
    named_addresses: Vec<(String, AddressBytes)>,

    /// Directory storing Move resources, events, and module bytecodes produced by module publishing
    /// and script execution.
    #[structopt(long, default_value = DEFAULT_STORAGE_DIR, parse(from_os_str), global = true)]
    storage_dir: PathBuf,
    /// Directory storing build artifacts produced by compilation.
    #[structopt(long, default_value = DEFAULT_BUILD_DIR, parse(from_os_str), global = true)]
    build_dir: PathBuf,

    /// Dependency inclusion mode.
    #[structopt(
        long,
        default_value = DEFAULT_DEP_MODE,
        global = true,
    )]
    mode: ModeType,

    /// Print additional diagnostics.
    #[structopt(short = "v", global = true)]
    verbose: bool,
}

/// MoveCLI is the CLI that will be executed by the `move-cli` command
/// The `cmd` argument is added here rather than in `Move` to make it
/// easier for other crates to extend `move-cli`
#[derive(StructOpt)]
pub struct MoveCLI {
    #[structopt(flatten)]
    move_args: Move,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
pub enum Command {
    #[structopt(name = "package")]
    Package {
        /// Path to package. If none is supplied the current directory will be used.
        #[structopt(long = "path", short = "p", global = true, parse(from_os_str))]
        path: Option<PathBuf>,

        #[structopt(flatten)]
        config: move_package::BuildConfig,

        #[structopt(subcommand)]
        cmd: package::cli::PackageCommand,
    },
    /// Compile and emit Move bytecode for the specified scripts and/or modules.
    #[structopt(name = "compile")]
    Compile {
        /// The source files to check.
        #[structopt(
            name = "PATH_TO_SOURCE_FILE",
            default_value = DEFAULT_SOURCE_DIR,
        )]
        source_files: Vec<String>,
        /// Do not emit source map information along with the compiled bytecode.
        #[structopt(long = "no-source-maps")]
        no_source_maps: bool,
        /// Type check and verify the specified scripts and/or modules. Does not emit bytecode.
        #[structopt(long = "check")]
        check: bool,
    },
    /// Execute a sandbox command.
    #[structopt(name = "sandbox")]
    Sandbox {
        #[structopt(subcommand)]
        cmd: sandbox::cli::SandboxCommand,
    },
    /// (Experimental) Run static analyses on Move source or bytecode.
    #[structopt(name = "experimental")]
    Experimental {
        #[structopt(subcommand)]
        cmd: experimental::cli::ExperimentalCommand,
    },
}

pub fn run_cli(
    natives: Vec<NativeFunctionRecord>,
    error_descriptions: &ErrorMapping,
    move_args: &Move,
    cmd: &Command,
) -> Result<()> {
    let mode = Mode::new(move_args.mode);
    let additional_named_addresses =
        shared::verify_and_create_named_address_mapping(move_args.named_addresses.clone())?;

    match cmd {
        Command::Compile {
            source_files,
            no_source_maps,
            check,
        } => {
            let state = mode.prepare_state(&move_args.build_dir, &move_args.storage_dir)?;
            if *check {
                base::commands::check(
                    &[state.interface_files_dir()?],
                    false,
                    source_files,
                    state.get_named_addresses(additional_named_addresses)?,
                    move_args.verbose,
                )
            } else {
                base::commands::compile(
                    &[state.interface_files_dir()?],
                    state.build_dir().to_str().unwrap(),
                    false,
                    source_files,
                    state.get_named_addresses(additional_named_addresses)?,
                    !*no_source_maps,
                    move_args.verbose,
                )
            }
        }
        Command::Sandbox { cmd } => {
            cmd.handle_command(natives, error_descriptions, move_args, &mode)
        }
        Command::Experimental { cmd } => cmd.handle_command(move_args, &mode),
        Command::Package { path, config, cmd } => {
            package::cli::handle_package_commands(path, config.clone(), cmd)
        }
    }
}

pub fn move_cli(
    natives: Vec<NativeFunctionRecord>,
    error_descriptions: &ErrorMapping,
) -> Result<()> {
    let args = MoveCLI::from_args();
    run_cli(natives, error_descriptions, &args.move_args, &args.cmd)
}
