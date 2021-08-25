// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs::create_dir_all,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::Result;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

use move_package::source_package::layout::SourcePackageLayout;
use move_prover::run_move_prover_with_model;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum PackageCommand {
    /// Create a new Move package with name `name` at `path`. If `path` is not provided the package
    /// will be created in the directory `name`.
    #[structopt(name = "new")]
    New {
        /// The name of the package to be created.
        name: String,
    },
    /// Build the package at `path`. If no path is provided defaults to current directory.
    #[structopt(name = "build")]
    Build,
    /// Generate error map for the package and its dependencies at `path` for use by the Move
    /// explanation tool.
    #[structopt(name = "errmap")]
    ErrMapGen {
        /// The prefix that all error reasons within modules will be prefixed with, e.g., "E" if
        /// all error reasons are "E_CANNOT_PERFORM_OPERATION", "E_CANNOT_ACCESS", etc.
        error_prefix: Option<String>,
        /// The file to serialize the generated error map to.
        #[structopt(default_value = "error_map", parse(from_os_str))]
        output_file: PathBuf,
    },
    /// Run the Move Prover on the package at `path`. If no path is provided defaults to current
    /// directory.
    #[structopt(name = "prove")]
    Prove {
        #[structopt(subcommand)]
        cmd: Option<ProverOptions>,
    },
}

#[derive(StructOpt)]
pub enum ProverOptions {
    // Pass through unknown commands to the prover Clap parser
    #[structopt(external_subcommand)]
    Options(Vec<String>),
}

pub fn handle_package_commands(
    path: &Option<PathBuf>,
    config: move_package::BuildConfig,
    cmd: &PackageCommand,
) -> Result<()> {
    let path = path
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    match cmd {
        PackageCommand::Build => {
            config.compile_package(&path, &mut std::io::stdout())?;
        }
        PackageCommand::New { name } => {
            let creation_path = Path::new(&path).join(name);
            create_dir_all(&creation_path)?;
            create_dir_all(creation_path.join(SourcePackageLayout::Sources.path()))?;
            let mut w =
                std::fs::File::create(creation_path.join(SourcePackageLayout::Manifest.path()))?;
            writeln!(
                &mut w,
                "[package]\nname = \"{}\"\nversion = \"0.0.0\"",
                name
            )?;
        }
        PackageCommand::Prove { cmd } => {
            let options = match cmd {
                None => move_prover::cli::Options::default(),
                Some(ProverOptions::Options(options)) => {
                    move_prover::cli::Options::create_from_args(options)?
                }
            };
            let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
            let now = Instant::now();
            let model = config.move_model_for_package(&path)?;
            run_move_prover_with_model(&model, &mut error_writer, options, Some(now))?;
        }
        PackageCommand::ErrMapGen {
            error_prefix,
            output_file,
        } => {
            let mut errmap_options = errmapgen::ErrmapOptions::default();
            if let Some(err_prefix) = error_prefix {
                errmap_options.error_prefix = err_prefix.to_string();
            }
            errmap_options.output_file = format!(
                "{:?}.{}",
                output_file,
                move_command_line_common::files::MOVE_ERROR_DESC_EXTENSION
            );
            let model = config.move_model_for_package(&path)?;
            let mut errmap_gen = errmapgen::ErrmapGen::new(&model, &errmap_options);
            errmap_gen.gen();
            errmap_gen.save_result();
        }
    };
    Ok(())
}
