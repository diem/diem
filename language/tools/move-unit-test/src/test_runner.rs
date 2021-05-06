// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    format_module_id,
    test_reporter::{FailureReason, TestFailure, TestResults, TestStatistics},
};
use anyhow::Result;
use colored::*;
use move_binary_format::{
    errors::{PartialVMError, VMResult},
    file_format::CompiledModule,
};
use move_core_types::{
    gas_schedule::{CostTable, GasAlgebra, GasCost, GasUnits},
    identifier::IdentStr,
    value::serialize_values,
    vm_status::StatusCode,
};
use move_lang::{
    shared::Flags,
    unit_test::{ExpectedFailure, ModuleTestPlan, TestCase, TestPlan},
};
use move_model::{model::GlobalEnv, run_model_builder_with_compilation_flags};
use move_vm_runtime::{logging::NoContextLog, move_vm::MoveVM};
use move_vm_test_utils::InMemoryStorage;
use move_vm_types::gas_schedule::{zero_cost_schedule, GasStatus};
use rayon::prelude::*;
use std::{io::Write, marker::Send, sync::Mutex};

/// Test state common to all tests
#[derive(Debug)]
pub struct SharedTestingConfig {
    execution_bound: u64,
    cost_table: CostTable,
    starting_storage_state: InMemoryStorage,
    source_files: Vec<String>,
    check_stackless_vm: bool,
    verbose: bool,
}

#[derive(Debug)]
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
    for module in modules {
        let module_id = module.self_id();
        let mut module_bytes = Vec::new();
        module.serialize(&mut module_bytes)?;
        storage.publish_or_overwrite_module(module_id, module_bytes);
    }

    Ok(storage)
}

impl TestRunner {
    pub fn new(
        execution_bound: u64,
        num_threads: usize,
        check_stackless_vm: bool,
        verbose: bool,
        tests: TestPlan,
    ) -> Result<Self> {
        let source_files = tests
            .files
            .keys()
            .map(|filepath| filepath.to_string())
            .collect();
        let modules = tests.module_info.values().map(|info| &info.0);
        let starting_storage_state = setup_test_storage(modules)?;
        Ok(Self {
            testing_config: SharedTestingConfig {
                starting_storage_state,
                execution_bound,
                cost_table: unit_cost_table(),
                source_files,
                check_stackless_vm,
                verbose,
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
    ) -> VMResult<Vec<Vec<u8>>> {
        let move_vm = MoveVM::new();
        let mut session = move_vm.new_session(&self.starting_storage_state);
        // TODO: collect VM logs if the verbose flag (i.e, `self.verbose`) is set
        let log_context = NoContextLog::new();

        session.execute_function(
            &test_plan.module_id,
            &IdentStr::new(function_name).unwrap(),
            vec![], // no ty args, at least for now
            serialize_values(test_info.arguments.iter()),
            &mut GasStatus::new(&self.cost_table, GasUnits::new(self.execution_bound)),
            &log_context,
        )
    }

    fn execute_via_stackless_vm(
        &self,
        env: &GlobalEnv,
        test_plan: &ModuleTestPlan,
        function_name: &str,
        test_info: &TestCase,
    ) -> VMResult<Vec<Vec<u8>>> {
        bytecode_interpreter::interpret_with_default_pipeline(
            env,
            &test_plan.module_id,
            &IdentStr::new(function_name).unwrap(),
            &[], // no ty args, at least for now
            &test_info.arguments,
            self.verbose,
        )
    }

    // The result returned by the stackless VM does not contain code offsets and indices. In order to
    // do cross-vm comparison, we need to adapt the Move VM result by removing these fields.
    fn adapt_move_vm_result(result: VMResult<Vec<Vec<u8>>>) -> VMResult<Vec<Vec<u8>>> {
        result.map_err(|err| {
            let (status_code, sub_status, message, location, _, _) = err.all_data();
            let adapted = PartialVMError::new(status_code);
            let adapted = match sub_status {
                None => adapted,
                Some(status_code) => adapted.with_sub_status(status_code),
            };
            let adapted = match message {
                None => adapted,
                Some(message) => adapted.with_message(message),
            };
            adapted.finish(location)
        })
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
            let model =
                run_model_builder_with_compilation_flags(&self.source_files, &[], Flags::testing())
                    .unwrap_or_else(|e| panic!("Unable to build stackless bytecode: {}", e));
            Some(model)
        } else {
            None
        };

        for (function_name, test_info) in &test_plan.tests {
            let exec_result = self.execute_via_move_vm(test_plan, function_name, test_info);
            if self.check_stackless_vm {
                let stackless_vm_result = self.execute_via_stackless_vm(
                    stackless_model.as_ref().unwrap(),
                    test_plan,
                    function_name,
                    test_info,
                );
                let move_vm_result = Self::adapt_move_vm_result(exec_result.clone());
                if stackless_vm_result != move_vm_result {
                    fail(function_name);
                    stats.test_failure(
                        TestFailure::new(
                            FailureReason::mismatch(move_vm_result, stackless_vm_result),
                            function_name,
                            None,
                        ),
                        &test_plan,
                    );
                    continue;
                }
            };
            match exec_result {
                Err(err) => match (test_info.expected_failure.as_ref(), err.sub_status()) {
                    // Ran out of ticks, report a test timeout and log a test failure
                    _ if err.major_status() == StatusCode::OUT_OF_GAS => {
                        timeout(function_name);
                        stats.test_failure(
                            TestFailure::new(FailureReason::timeout(), function_name, Some(err)),
                            &test_plan,
                        )
                    }
                    // Expected the test to not abort, but it aborted with `code`
                    (None, Some(code)) => {
                        fail(function_name);
                        stats.test_failure(
                            TestFailure::new(
                                FailureReason::aborted(code),
                                function_name,
                                Some(err),
                            ),
                            &test_plan,
                        )
                    }
                    // Expected the test the abort with a specific `code`, and it did abort with
                    // that abort code
                    (Some(ExpectedFailure::ExpectedWithCode(code)), Some(other_code))
                        if err.major_status() == StatusCode::ABORTED && *code == other_code =>
                    {
                        pass(function_name);
                        stats.test_success();
                    }
                    // Expected the test to abort with a specific `code` but it aborted with a
                    // different `other_code`
                    (Some(ExpectedFailure::ExpectedWithCode(code)), Some(other_code)) => {
                        fail(function_name);
                        stats.test_failure(
                            TestFailure::new(
                                FailureReason::wrong_abort(*code, other_code),
                                function_name,
                                Some(err),
                            ),
                            &test_plan,
                        )
                    }
                    // Expected the test to abort and it aborted, but we don't need to check the code
                    (Some(ExpectedFailure::Expected), Some(_)) => {
                        pass(function_name);
                        stats.test_success();
                    }
                    // Expected the test to abort and it aborted with internal error
                    (Some(ExpectedFailure::Expected), None)
                        if err.major_status() != StatusCode::EXECUTED =>
                    {
                        pass(function_name);
                        stats.test_success();
                    }
                    // Unexpected return status from the VM, signal that we hit an unknown error.
                    (_, None) => {
                        fail(function_name);
                        stats.test_failure(
                            TestFailure::new(FailureReason::unknown(), function_name, Some(err)),
                            &test_plan,
                        )
                    }
                },
                Ok(_) => {
                    // Expected the test to fail, but it executed
                    if test_info.expected_failure.is_some() {
                        fail(function_name);
                        stats.test_failure(
                            TestFailure::new(FailureReason::no_abort(), function_name, None),
                            &test_plan,
                        )
                    } else {
                        // Expected the test to execute fully and it did
                        pass(function_name);
                        stats.test_success();
                    }
                }
            }
        }

        stats
    }
}
