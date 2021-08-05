// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod cargo_runner;
pub mod test_reporter;
pub mod test_runner;
use crate::test_runner::TestRunner;
use move_core_types::language_storage::ModuleId;
use move_lang::{
    self,
    diagnostics::{self, codes::Severity},
    unit_test::{self, TestPlan},
    Compiler, Flags, PASS_CFGIR,
};
use move_vm_runtime::native_functions::NativeFunctionTable;
use std::{
    io::{Result, Write},
    marker::Send,
    sync::Mutex,
};
use structopt::*;

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "Move Unit Test", about = "Unit testing for Move code.")]
pub struct UnitTestingConfig {
    /// Bound the number of instructions that can be executed by any one test.
    #[structopt(
        name = "instructions",
        default_value = "5000",
        short = "i",
        long = "instructions"
    )]
    pub instruction_execution_bound: u64,

    /// A filter string to determine which unit tests to run
    #[structopt(name = "filter", short = "f", long = "filter")]
    pub filter: Option<String>,

    /// List all tests
    #[structopt(name = "list", short = "l", long = "list")]
    pub list: bool,

    /// Number of threads to use for running tests.
    #[structopt(
        name = "num_threads",
        default_value = "8",
        short = "t",
        long = "threads"
    )]
    pub num_threads: usize,

    /// Dependency files
    #[structopt(name = "dependencies", long = "dependencies", short = "d")]
    pub dep_files: Vec<String>,

    /// Report test statistics at the end of testing
    #[structopt(name = "report_statistics", short = "s", long = "statistics")]
    pub report_statistics: bool,

    /// Show the storage state at the end of execution of a failing test
    #[structopt(name = "global_state_on_error", short = "g", long = "state_on_error")]
    pub report_storage_on_error: bool,

    /// Source files
    #[structopt(name = "sources")]
    pub source_files: Vec<String>,

    /// Use the stackless bytecode interpreter to run the tests and cross check its results with
    /// the execution result from Move VM.
    #[structopt(long = "stackless")]
    pub check_stackless_vm: bool,

    /// Verbose mode
    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,
}

fn format_module_id(module_id: &ModuleId) -> String {
    format!(
        "0x{}::{}",
        module_id.address().short_str_lossless(),
        module_id.name()
    )
}

impl UnitTestingConfig {
    /// Create a unit testing config for use with `register_move_unit_tests`
    pub fn default_with_bound(bound: Option<u64>) -> Self {
        Self {
            instruction_execution_bound: bound.unwrap_or(5000),
            filter: None,
            num_threads: 8,
            report_statistics: false,
            report_storage_on_error: false,
            source_files: vec![],
            dep_files: vec![],
            check_stackless_vm: false,
            verbose: false,
            list: false,
        }
    }

    fn compile_to_test_plan(&self, source_files: &[String], deps: &[String]) -> Option<TestPlan> {
        let (files, comments_and_compiler_res) = Compiler::new(source_files, deps)
            .set_flags(Flags::testing())
            .run::<PASS_CFGIR>()
            .unwrap();
        let (_, compiler) =
            diagnostics::unwrap_or_report_diagnostics(&files, comments_and_compiler_res);

        let (mut compiler, cfgir) = compiler.into_ast();
        let compilation_env = compiler.compilation_env();
        let test_plan = unit_test::plan_builder::construct_test_plan(compilation_env, &cfgir);

        if let Err(diags) = compilation_env.check_diags_at_or_above_severity(Severity::Warning) {
            diagnostics::report_diagnostics(&files, diags);
        }

        let compilation_result = compiler.at_cfgir(cfgir).build();

        let (units, warnings) =
            diagnostics::unwrap_or_report_diagnostics(&files, compilation_result);
        diagnostics::report_warnings(&files, warnings);
        test_plan.map(|tests| TestPlan::new(tests, files, units))
    }

    /// Build a test plan from a unit test config
    pub fn build_test_plan(&self) -> Option<TestPlan> {
        let mut deps = self.dep_files.clone();

        deps.extend(move_stdlib::unit_testing_files());

        let TestPlan {
            files, module_info, ..
        } = self.compile_to_test_plan(&deps, &[])?;

        let mut test_plan = self.compile_to_test_plan(&self.source_files, &deps)?;
        test_plan.module_info.extend(module_info.into_iter());
        test_plan.files.extend(files.into_iter());
        Some(test_plan)
    }

    /// Public entry point to Move unit testing as a library
    /// Returns `true` if all unit tests passed. Otherwise, returns `false`.
    pub fn run_and_report_unit_tests<W: Write + Send>(
        &self,
        test_plan: TestPlan,
        native_function_table: Option<NativeFunctionTable>,
        writer: W,
    ) -> Result<(W, bool)> {
        let shared_writer = Mutex::new(writer);

        if self.list {
            for (module_id, test_plan) in &test_plan.module_tests {
                for test_name in test_plan.tests.keys() {
                    writeln!(
                        shared_writer.lock().unwrap(),
                        "{}::{}: test",
                        format_module_id(module_id),
                        test_name
                    )?;
                }
            }
            return Ok((shared_writer.into_inner().unwrap(), true));
        }

        writeln!(shared_writer.lock().unwrap(), "Running Move unit tests")?;
        let mut test_runner = TestRunner::new(
            self.instruction_execution_bound,
            self.num_threads,
            self.check_stackless_vm,
            self.verbose,
            self.report_storage_on_error,
            test_plan,
            native_function_table,
        )
        .unwrap();

        if let Some(filter_str) = &self.filter {
            test_runner.filter(filter_str)
        }

        let test_results = test_runner.run(&shared_writer).unwrap();
        if self.report_statistics {
            test_results.report_statistics(&shared_writer)?;
        }
        let all_tests_passed = test_results.summarize(&shared_writer)?;

        let writer = shared_writer.into_inner().unwrap();
        Ok((writer, all_tests_passed))
    }
}
