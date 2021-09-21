// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    fs::{create_dir_all, read_to_string},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::Result;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

use move_lang::{
    compiled_unit::CompiledUnit,
    diagnostics::{self, codes::Severity},
    unit_test::{plan_builder::construct_test_plan, TestPlan},
    PASS_CFGIR,
};
use move_package::{
    compilation::build_plan::BuildPlan, source_package::layout::SourcePackageLayout,
};
use move_prover::run_move_prover_with_model;
use move_unit_test::UnitTestingConfig;
use structopt::StructOpt;

const ERROR_MAP_FILENAME: &str = "error_map";

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
        /// all error reasons are "ECANNOT_PERFORM_OPERATION", "ECANNOT_ACCESS", etc.
        error_prefix: Option<String>,
    },
    /// Run the Move Prover on the package at `path`. If no path is provided defaults to current
    /// directory.
    #[structopt(name = "prove")]
    Prove {
        #[structopt(subcommand)]
        cmd: Option<ProverOptions>,
    },
    #[structopt(name = "test")]
    UnitTest {
        /// Bound the number of instructions that can be executed by any one test.
        #[structopt(
            name = "instructions",
            default_value = "5000",
            short = "i",
            long = "instructions"
        )]
        instruction_execution_bound: u64,
        /// A filter string to determine which unit tests to run
        #[structopt(name = "filter", short = "f", long = "filter")]
        filter: Option<String>,

        /// List all tests
        #[structopt(name = "list", short = "l", long = "list")]
        list: bool,

        /// Number of threads to use for running tests.
        #[structopt(
            name = "num_threads",
            default_value = "8",
            short = "t",
            long = "threads"
        )]
        num_threads: usize,
        /// Report test statistics at the end of testing
        #[structopt(name = "report_statistics", short = "s", long = "statistics")]
        report_statistics: bool,

        /// Show the storage state at the end of execution of a failing test
        #[structopt(name = "global_state_on_error", short = "g", long = "state_on_error")]
        report_storage_on_error: bool,

        /// Use the stackless bytecode interpreter to run the tests and cross check its results with
        /// the execution result from Move VM.
        #[structopt(long = "stackless")]
        check_stackless_vm: bool,

        /// Verbose mode
        #[structopt(long = "verbose")]
        verbose_mode: bool,
    },
    /// Explain abort codes raised from a package
    #[structopt(name = "explain")]
    Explain {
        /// The location (module id) returned with a `MoveAbort` error
        #[structopt(long = "location")]
        location: String,
        /// The abort code returned with a `MoveAbort` error
        #[structopt(long = "code")]
        abort_code: u64,
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
    mut config: move_package::BuildConfig,
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
            println!("Verification succeeded");
        }
        PackageCommand::ErrMapGen { error_prefix } => {
            let mut errmap_options = errmapgen::ErrmapOptions::default();
            if let Some(err_prefix) = error_prefix {
                errmap_options.error_prefix = err_prefix.to_string();
            }
            errmap_options.output_file = path
                .join(ERROR_MAP_FILENAME)
                .with_extension(move_command_line_common::files::MOVE_ERROR_DESC_EXTENSION)
                .to_string_lossy()
                .to_string();
            let model = config.move_model_for_package(&path)?;
            let mut errmap_gen = errmapgen::ErrmapGen::new(&model, &errmap_options);
            errmap_gen.gen();
            errmap_gen.save_result();
        }
        PackageCommand::Explain {
            location,
            abort_code,
        } => {
            let errmap_path = path
                .join("error_map")
                .with_extension(move_command_line_common::files::MOVE_ERROR_DESC_EXTENSION);
            if !errmap_path.is_file() {
                eprintln!(
                    "Error map for package not found at '{:?}' you may need to generate it",
                    errmap_path
                );
                std::process::exit(1);
            }
            move_explain::parse_and_explain(
                &mut std::io::stdout(),
                location,
                *abort_code,
                &errmap_path,
            )
        }
        PackageCommand::UnitTest {
            instruction_execution_bound,
            filter,
            list,
            num_threads,
            report_statistics,
            report_storage_on_error,
            check_stackless_vm,
            verbose_mode,
        } => {
            let unit_test_config = UnitTestingConfig {
                instruction_execution_bound: *instruction_execution_bound,
                filter: filter.clone(),
                list: *list,
                num_threads: *num_threads,
                report_statistics: *report_statistics,
                report_storage_on_error: *report_storage_on_error,
                check_stackless_vm: *check_stackless_vm,
                verbose: *verbose_mode,
                ..UnitTestingConfig::default_with_bound(None)
            };

            let mut test_plan = None;
            config.test_mode = true;
            config.dev_mode = true;

            let resolution_graph = config.resolution_graph_for_package(&path)?;
            let dep_file_map: HashMap<_, _> = resolution_graph
                .package_table
                .iter()
                .flat_map(|(_, rpkg)| {
                    rpkg.get_sources(&resolution_graph.build_options)
                        .unwrap()
                        .iter()
                        .map(|fname| (*fname, read_to_string(Path::new(fname.as_str())).unwrap()))
                        .collect::<HashMap<_, _>>()
                })
                .collect();
            let build_plan = BuildPlan::create(resolution_graph)?;
            let pkg =
                build_plan.compile_with_driver(&mut std::io::stdout(), |compiler, is_root| {
                    if !is_root {
                        compiler.build_and_report()
                    } else {
                        let (files, comments_and_compiler_res) =
                            compiler.run::<PASS_CFGIR>().unwrap();
                        let (_, compiler) = diagnostics::unwrap_or_report_diagnostics(
                            &files,
                            comments_and_compiler_res,
                        );
                        let (mut compiler, cfgir) = compiler.into_ast();
                        let compilation_env = compiler.compilation_env();
                        let built_test_plan = construct_test_plan(compilation_env, &cfgir);

                        if let Err(diags) =
                            compilation_env.check_diags_at_or_above_severity(Severity::Warning)
                        {
                            diagnostics::report_diagnostics(&files, diags);
                        }

                        let compilation_result = compiler.at_cfgir(cfgir).build();

                        let (units, _) =
                            diagnostics::unwrap_or_report_diagnostics(&files, compilation_result);

                        test_plan = Some((built_test_plan, files.clone(), units.clone()));
                        Ok((files, units))
                    }
                })?;

            let (test_plan, mut files, units) = test_plan.unwrap();
            files.extend(dep_file_map);
            let mut test_plan = TestPlan::new(test_plan.unwrap(), files, units);
            for pkg in pkg.transitive_dependencies() {
                for module in &pkg.compiled_units {
                    match module {
                        CompiledUnit::Script(_) => (),
                        CompiledUnit::Module(module) => {
                            test_plan
                                .module_info
                                .insert(module.module.self_id(), module.clone());
                        }
                    }
                }
            }

            // TODO: We only run with stdlib natives for now. Revisit once we have native support
            // in packages.
            if unit_test_config
                .run_and_report_unit_tests(test_plan, None, std::io::stdout())
                .unwrap()
                .1
            {
                std::process::exit(0)
            } else {
                std::process::exit(1)
            }
        }
    };
    Ok(())
}
