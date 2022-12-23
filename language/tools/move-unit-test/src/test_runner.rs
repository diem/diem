// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    format_module_id,
    test_reporter::{FailureReason, TestFailure, TestResults, TestRunInfo, TestStatistics},
};
use anyhow::Result;
use bytecode_interpreter::{
    concrete::{settings::InterpreterSettings, value::GlobalState},
    shared::bridge::{adapt_move_vm_change_set, adapt_move_vm_result},
    StacklessBytecodeInterpreter,
};
use colored::*;
use move_binary_format::{errors::VMResult, file_format::CompiledModule};
use move_bytecode_utils::Modules;
use move_core_types::{
    account_address::AccountAddress,
    effects::ChangeSet,
    gas_schedule::{CostTable, GasAlgebra, GasCost, GasUnits},
    identifier::IdentStr,
    value::serialize_values,
    vm_status::StatusCode,
};
use move_lang::{
    shared::{AddressBytes, Flags},
    unit_test::{ExpectedFailure, ModuleTestPlan, TestCase, TestPlan},
};
use move_model::{
    model::GlobalEnv, options::ModelBuilderOptions,
    run_model_builder_with_options_and_compilation_flags,
};
use move_vm_runtime::{move_vm::MoveVM, native_functions::NativeFunctionTable};
use move_vm_test_utils::InMemoryStorage;
use move_vm_types::gas_schedule::{zero_cost_schedule, GasStatus};
use rayon::prelude::*;
use resource_viewer::MoveValueAnnotator;
use std::{collections::BTreeMap, io::Write, marker::Send, sync::Mutex, time::Instant};

/// Test state common to all tests
pub struct SharedTestingConfig {
    save_storage_state_on_failure: bool,
    execution_bound: u64,
    cost_table: CostTable,
    native_function_table: NativeFunctionTable,
    starting_storage_state: InMemoryStorage,
    source_files: Vec<String>,
    named_address_values: BTreeMap<String, AddressBytes>,
    check_stackless_vm: bool,
    verbose: bool,
}

pub struct TestRunner {
    num_threads: usize,
    testing_config: SharedTestingConfig,
    tests: TestPlan,
}

/// A gas schedule where every instruction has a cost of "1". This is used to bound execution of a
/// test to a certain number of ticks.
fn unit_cost_table() -> CostTable {
    let mut cost_schedule = zero_cost_schedule();
    cost_schedule.instruction_table.iter_mut().for_each(|cost| {
        *cost = GasCost::new(1, 1);
    });
    cost_schedule.native_table.iter_mut().for_each(|cost| {
        *cost = GasCost::new(1, 1);
    });
    cost_schedule
}

/// Setup storage state with the set of modules that will be needed for all tests
fn setup_test_storage<'a>(
    modules: impl Iterator<Item = &'a CompiledModule>,
) -> Result<InMemoryStorage> {
    let mut storage = InMemoryStorage::new();
    let modules = Modules::new(modules);
    for module in modules
        .compute_dependency_graph()
        .compute_topological_order()?
    {
        let module_id = module.self_id();
        let mut module_bytes = Vec::new();
        module.serialize(&mut module_bytes)?;
        storage.publish_or_overwrite_module(module_id, module_bytes);
    }

    Ok(storage)
}

/// Print the updates to storage represented by `cs` in the context of the starting storage state
/// `storage`.
fn print_resources(cs: &ChangeSet, storage: &InMemoryStorage) -> Result<String> {
    use std::fmt::Write;
    let mut buf = String::new();
    let annotator = MoveValueAnnotator::new(storage);
    for (account_addr, account_state) in cs.accounts() {
        writeln!(&mut buf, "0x{}:", account_addr.short_str_lossless())?;

        for (tag, resource_opt) in account_state.resources() {
            if let Some(resource) = resource_opt {
                writeln!(
                    &mut buf,
                    "\t{}",
                    format!("=> {}", annotator.view_resource(tag, resource)?).replace('\n', "\n\t")
                )?;
            }
        }
    }

    Ok(buf)
}

impl TestRunner {
    pub fn new(
        execution_bound: u64,
        num_threads: usize,
        check_stackless_vm: bool,
        verbose: bool,
        save_storage_state_on_failure: bool,
        tests: TestPlan,
        native_function_table: Option<NativeFunctionTable>,
        named_address_values: BTreeMap<String, AddressBytes>,
    ) -> Result<Self> {
        let source_files = tests
            .files
            .keys()
            .map(|filepath| filepath.to_string())
            .collect();
        let modules = tests.module_info.values().map(|info| &info.0);
        let starting_storage_state = setup_test_storage(modules)?;
        let native_function_table = native_function_table.unwrap_or_else(|| {
            move_stdlib::natives::all_natives(AccountAddress::from_hex_literal("0x1").unwrap())
        });
        Ok(Self {
            testing_config: SharedTestingConfig {
                save_storage_state_on_failure,
                starting_storage_state,
                execution_bound,
                native_function_table,
                cost_table: unit_cost_table(),
                source_files,
                check_stackless_vm,
                verbose,
                named_address_values,
            },
            num_threads,
            tests,
        })
    }

    pub fn run<W: Write + Send>(self, writer: &Mutex<W>) -> Result<TestResults> {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .build()
            .unwrap()
            .install(|| {
                let final_statistics = self
                    .tests
                    .module_tests
                    .par_iter()
                    .map(|(_, test_plan)| self.testing_config.exec_module_tests(test_plan, writer))
                    .reduce(TestStatistics::new, |acc, stats| acc.combine(stats));

                Ok(TestResults::new(final_statistics, self.tests))
            })
    }

    pub fn filter(&mut self, test_name_slice: &str) {
        for (module_id, module_test) in self.tests.module_tests.iter_mut() {
            if module_id.name().as_str().contains(test_name_slice) {
                continue;
            } else {
                let tests = std::mem::take(&mut module_test.tests);
                module_test.tests = tests
                    .into_iter()
                    .filter(|(test_name, _)| test_name.as_str().contains(test_name_slice))
                    .collect();
            }
        }
    }
}

impl SharedTestingConfig {
    fn execute_via_move_vm(
        &self,
        test_plan: &ModuleTestPlan,
        function_name: &str,
        test_info: &TestCase,
    ) -> (VMResult<ChangeSet>, VMResult<Vec<Vec<u8>>>, TestRunInfo) {
        let move_vm = MoveVM::new(self.native_function_table.clone()).unwrap();
        let mut session = move_vm.new_session(&self.starting_storage_state);
        let mut gas_meter = GasStatus::new(&self.cost_table, GasUnits::new(self.execution_bound));
        // TODO: collect VM logs if the verbose flag (i.e, `self.verbose`) is set

        let now = Instant::now();
        let return_result = session.execute_function(
            &test_plan.module_id,
            IdentStr::new(function_name).unwrap(),
            vec![], // no ty args, at least for now
            serialize_values(test_info.arguments.iter()),
            &mut gas_meter,
        );
        let test_run_info = TestRunInfo::new(
            function_name.to_string(),
            now.elapsed(),
            self.execution_bound - gas_meter.remaining_gas().get(),
        );
        (
            session.finish().map(|(cs, _)| cs),
            return_result,
            test_run_info,
        )
    }

    fn execute_via_stackless_vm(
        &self,
        env: &GlobalEnv,
        test_plan: &ModuleTestPlan,
        function_name: &str,
        test_info: &TestCase,
    ) -> (
        VMResult<ChangeSet>,
        VMResult<Vec<Vec<u8>>>,
        TestRunInfo,
        Option<String>,
    ) {
        let now = Instant::now();

        let settings = if self.verbose {
            InterpreterSettings::verbose_default()
        } else {
            InterpreterSettings::default()
        };
        let interpreter = StacklessBytecodeInterpreter::new(env, None, settings);

        // NOTE: as of now, `self.starting_storage_state` contains modules only and no resources.
        // The modules are captured by `env: &GlobalEnv` and the default GlobalState captures the
        // empty-resource state.
        let global_state = GlobalState::default();
        let (return_result, change_set, _) = interpreter.interpret(
            &test_plan.module_id,
            IdentStr::new(function_name).unwrap(),
            &[], // no ty args, at least for now
            &test_info.arguments,
            &global_state,
        );
        let prop_check_result = interpreter.report_property_checking_results();

        let test_run_info = TestRunInfo::new(
            function_name.to_string(),
            now.elapsed(),
            // NOTE (mengxu) instruction counting on stackless VM might not be very useful because
            // gas is not charged against stackless VM instruction.
            0,
        );
        (
            Ok(change_set),
            return_result,
            test_run_info,
            prop_check_result,
        )
    }

    fn exec_module_tests<W: Write>(
        &self,
        test_plan: &ModuleTestPlan,
        writer: &Mutex<W>,
    ) -> TestStatistics {
        let mut stats = TestStatistics::new();
        let pass = |fn_name: &str| {
            writeln!(
                writer.lock().unwrap(),
                "[ {}    ] {}::{}",
                "PASS".bold().bright_green(),
                format_module_id(&test_plan.module_id),
                fn_name
            )
            .unwrap()
        };
        let fail = |fn_name: &str| {
            writeln!(
                writer.lock().unwrap(),
                "[ {}    ] {}::{}",
                "FAIL".bold().bright_red(),
                format_module_id(&test_plan.module_id),
                fn_name,
            )
            .unwrap()
        };
        let timeout = |fn_name: &str| {
            writeln!(
                writer.lock().unwrap(),
                "[ {} ] {}::{}",
                "TIMEOUT".bold().bright_yellow(),
                format_module_id(&test_plan.module_id),
                fn_name,
            )
            .unwrap();
        };

        let stackless_model = if self.check_stackless_vm {
            let model = run_model_builder_with_options_and_compilation_flags(
                &self.source_files,
                &[],
                ModelBuilderOptions::default(),
                Flags::testing(),
                self.named_address_values.clone(),
            )
            .unwrap_or_else(|e| panic!("Unable to build stackless bytecode: {}", e));
            Some(model)
        } else {
            None
        };

        for (function_name, test_info) in &test_plan.tests {
            let (cs_result, exec_result, test_run_info) =
                self.execute_via_move_vm(test_plan, function_name, test_info);
            if self.check_stackless_vm {
                let (stackless_vm_change_set, stackless_vm_result, _, prop_check_result) = self
                    .execute_via_stackless_vm(
                        stackless_model.as_ref().unwrap(),
                        test_plan,
                        function_name,
                        test_info,
                    );
                let move_vm_result = adapt_move_vm_result(exec_result.clone());
                let move_vm_change_set =
                    adapt_move_vm_change_set(cs_result.clone(), &self.starting_storage_state);
                if stackless_vm_result != move_vm_result
                    || stackless_vm_change_set != move_vm_change_set
                {
                    fail(function_name);
                    stats.test_failure(
                        TestFailure::new(
                            FailureReason::mismatch(
                                move_vm_result,
                                move_vm_change_set,
                                stackless_vm_result,
                                stackless_vm_change_set,
                            ),
                            test_run_info,
                            None,
                            None,
                        ),
                        test_plan,
                    );
                    continue;
                }
                if let Some(prop_failure) = prop_check_result {
                    fail(function_name);
                    stats.test_failure(
                        TestFailure::new(
                            FailureReason::property(prop_failure),
                            test_run_info,
                            None,
                            None,
                        ),
                        test_plan,
                    );
                    continue;
                }
            }

            let save_session_state = || {
                if self.save_storage_state_on_failure {
                    cs_result.ok().and_then(|changeset| {
                        print_resources(&changeset, &self.starting_storage_state).ok()
                    })
                } else {
                    None
                }
            };
            match exec_result {
                Err(err) => match (test_info.expected_failure.as_ref(), err.sub_status()) {
                    // Ran out of ticks, report a test timeout and log a test failure
                    _ if err.major_status() == StatusCode::OUT_OF_GAS => {
                        timeout(function_name);
                        stats.test_failure(
                            TestFailure::new(
                                FailureReason::timeout(),
                                test_run_info,
                                Some(err),
                                save_session_state(),
                            ),
                            test_plan,
                        )
                    }
                    // Expected the test to not abort, but it aborted with `code`
                    (None, Some(code)) => {
                        fail(function_name);
                        stats.test_failure(
                            TestFailure::new(
                                FailureReason::aborted(code),
                                test_run_info,
                                Some(err),
                                save_session_state(),
                            ),
                            test_plan,
                        )
                    }
                    // Expected the test the abort with a specific `code`, and it did abort with
                    // that abort code
                    (Some(ExpectedFailure::ExpectedWithCode(code)), Some(other_code))
                        if err.major_status() == StatusCode::ABORTED && *code == other_code =>
                    {
                        pass(function_name);
                        stats.test_success(test_run_info, test_plan);
                    }
                    // Expected the test to abort with a specific `code` but it aborted with a
                    // different `other_code`
                    (Some(ExpectedFailure::ExpectedWithCode(code)), Some(other_code)) => {
                        fail(function_name);
                        stats.test_failure(
                            TestFailure::new(
                                FailureReason::wrong_abort(*code, other_code),
                                test_run_info,
                                Some(err),
                                save_session_state(),
                            ),
                            test_plan,
                        )
                    }
                    // Expected the test to abort and it aborted, but we don't need to check the code
                    (Some(ExpectedFailure::Expected), Some(_)) => {
                        pass(function_name);
                        stats.test_success(test_run_info, test_plan);
                    }
                    // Expected the test to abort and it aborted with internal error
                    (Some(ExpectedFailure::Expected), None)
                        if err.major_status() != StatusCode::EXECUTED =>
                    {
                        pass(function_name);
                        stats.test_success(test_run_info, test_plan);
                    }
                    // Unexpected return status from the VM, signal that we hit an unknown error.
                    (_, None) => {
                        fail(function_name);
                        stats.test_failure(
                            TestFailure::new(
                                FailureReason::unknown(),
                                test_run_info,
                                Some(err),
                                save_session_state(),
                            ),
                            test_plan,
                        )
                    }
                },
                Ok(_) => {
                    // Expected the test to fail, but it executed
                    if test_info.expected_failure.is_some() {
                        fail(function_name);
                        stats.test_failure(
                            TestFailure::new(
                                FailureReason::no_abort(),
                                test_run_info,
                                None,
                                save_session_state(),
                            ),
                            test_plan,
                        )
                    } else {
                        // Expected the test to execute fully and it did
                        pass(function_name);
                        stats.test_success(test_run_info, test_plan);
                    }
                }
            }
        }

        stats
    }
}
