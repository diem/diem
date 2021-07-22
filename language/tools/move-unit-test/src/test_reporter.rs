// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::format_module_id;
use colored::{control, Colorize};
use move_binary_format::errors::{Location, VMError, VMResult};
use move_core_types::{effects::ChangeSet, language_storage::ModuleId};
use move_lang::{
    diagnostics::{self, Diagnostic},
    unit_test::{ModuleTestPlan, TestPlan},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Result, Write},
    sync::Mutex,
    time::Duration,
};

#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub enum FailureReason {
    // Expected to abort, but it didn't
    NoAbort(String),
    // Aborted with the wrong code
    WrongAbort(String, u64, u64),
    // Abort wasn't expected, but it did
    Aborted(String, u64),
    // Test timed out
    Timeout(String),
    // The execution results of the Move VM and stackless VM does not match
    Mismatch {
        move_vm_return_values: Box<VMResult<Vec<Vec<u8>>>>,
        move_vm_change_set: Box<VMResult<ChangeSet>>,
        stackless_vm_return_values: Box<VMResult<Vec<Vec<u8>>>>,
        stackless_vm_change_set: Box<VMResult<ChangeSet>>,
    },
    // Property checking failed
    Property(String),
    // The test failed for some unknown reason. This shouldn't be encountered
    Unknown(String),
}

#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct TestFailure {
    pub test_run_info: TestRunInfo,
    pub vm_error: Option<VMError>,
    pub failure_reason: FailureReason,
    pub storage_state: Option<String>,
}

#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct TestRunInfo {
    pub function_ident: String,
    pub elapsed_time: Duration,
    pub instructions_executed: u64,
}

#[derive(Debug, Clone)]
pub struct TestStatistics {
    passed: BTreeMap<ModuleId, BTreeSet<TestRunInfo>>,
    failed: BTreeMap<ModuleId, BTreeSet<TestFailure>>,
}

#[derive(Debug, Clone)]
pub struct TestResults {
    final_statistics: TestStatistics,
    test_plan: TestPlan,
}

impl TestRunInfo {
    pub fn new(function_ident: String, elapsed_time: Duration, instructions_executed: u64) -> Self {
        Self {
            function_ident,
            elapsed_time,
            instructions_executed,
        }
    }
}

impl FailureReason {
    pub fn no_abort() -> Self {
        FailureReason::NoAbort("Test did not abort as expected".to_string())
    }

    pub fn wrong_abort(expected: u64, received: u64) -> Self {
        FailureReason::WrongAbort(
            "Test did not abort with expected code".to_string(),
            expected,
            received,
        )
    }

    pub fn aborted(abort_code: u64) -> Self {
        FailureReason::Aborted("Test was not expected to abort".to_string(), abort_code)
    }

    pub fn timeout() -> Self {
        FailureReason::Timeout("Test timed out".to_string())
    }

    pub fn mismatch(
        move_vm_return_values: VMResult<Vec<Vec<u8>>>,
        move_vm_change_set: VMResult<ChangeSet>,
        stackless_vm_return_values: VMResult<Vec<Vec<u8>>>,
        stackless_vm_change_set: VMResult<ChangeSet>,
    ) -> Self {
        FailureReason::Mismatch {
            move_vm_return_values: Box::new(move_vm_return_values),
            move_vm_change_set: Box::new(move_vm_change_set),
            stackless_vm_return_values: Box::new(stackless_vm_return_values),
            stackless_vm_change_set: Box::new(stackless_vm_change_set),
        }
    }

    pub fn property(details: String) -> Self {
        FailureReason::Property(details)
    }

    pub fn unknown() -> Self {
        FailureReason::Unknown("ITE: An unknown error was reported.".to_string())
    }
}

impl TestFailure {
    pub fn new(
        failure_reason: FailureReason,
        test_run_info: TestRunInfo,
        vm_error: Option<VMError>,
        storage_state: Option<String>,
    ) -> Self {
        Self {
            test_run_info,
            vm_error,
            failure_reason,
            storage_state,
        }
    }

    pub fn render_error(&self, test_plan: &TestPlan) -> String {
        let error_string = match &self.failure_reason {
            FailureReason::NoAbort(message) => message.to_string(),
            FailureReason::Timeout(message) => message.to_string(),
            FailureReason::WrongAbort(message, expected_code, other_code) => {
                let base_message = format!(
                    "{}. Expected test to abort with {} but instead it aborted with {} here",
                    message, expected_code, other_code,
                );
                Self::report_error_with_location(test_plan, base_message, &self.vm_error)
            }
            FailureReason::Aborted(message, code) => {
                let base_message = format!("{} but it aborted with {} here", message, code);
                Self::report_error_with_location(test_plan, base_message, &self.vm_error)
            }
            FailureReason::Mismatch {
                move_vm_return_values,
                move_vm_change_set,
                stackless_vm_return_values,
                stackless_vm_change_set,
            } => {
                format!(
                    "Executions via Move VM [M] and stackless VM [S] yield different results.\n\
                    [M] - return values: {:?}\n\
                    [S] - return values: {:?}\n\
                    [M] - change set: {:?}\n\
                    [S] - change set: {:?}\n\
                    ",
                    move_vm_return_values,
                    stackless_vm_return_values,
                    move_vm_change_set,
                    stackless_vm_change_set
                )
            }
            FailureReason::Property(message) => message.clone(),
            FailureReason::Unknown(message) => {
                format!(
                    "{} Location: {}\nVMError (if there is one): {}",
                    message,
                    TestFailure::report_error_with_location(
                        test_plan,
                        "".to_string(),
                        &self.vm_error
                    ),
                    self.vm_error
                        .as_ref()
                        .map(|err| format!("{:#?}", err))
                        .unwrap_or_else(|| "".to_string()),
                )
            }
        };

        match &self.storage_state {
            None => error_string,
            Some(storage_state) => {
                format!(
                    "{}\n────── Storage state at point of failure ──────\n{}",
                    error_string,
                    if storage_state.is_empty() {
                        "<empty>"
                    } else {
                        storage_state
                    }
                )
            }
        }
    }

    fn report_error_with_location(
        test_plan: &TestPlan,
        base_message: String,
        vm_error: &Option<VMError>,
    ) -> String {
        let report_diagnostics = if control::SHOULD_COLORIZE.should_colorize() {
            diagnostics::report_diagnostics_to_color_buffer
        } else {
            diagnostics::report_diagnostics_to_buffer
        };

        let vm_error = match vm_error {
            None => return base_message,
            Some(vm_error) => vm_error,
        };

        match vm_error.location() {
            Location::Module(module_id) => {
                let diags = vm_error
                    .offsets()
                    .iter()
                    .filter_map(|(fdef_idx, offset)| {
                        let function_source_map = test_plan
                            .module_info
                            .get(module_id)?
                            .source_map
                            .get_function_source_map(*fdef_idx)
                            .ok()?;
                        let loc = function_source_map.get_code_location(*offset)?;
                        let msg = format!("In this function in {}", format_module_id(module_id));
                        // TODO(tzakian) maybe migrate off of move-langs diagnostics?
                        Some(Diagnostic::new(
                            diagnostics::codes::Tests::TestFailed,
                            (loc, base_message.clone()),
                            vec![(function_source_map.decl_location, msg)],
                        ))
                    })
                    .collect();

                String::from_utf8(report_diagnostics(&test_plan.files, diags)).unwrap()
            }
            _ => base_message,
        }
    }
}

impl TestStatistics {
    pub fn new() -> Self {
        Self {
            passed: BTreeMap::new(),
            failed: BTreeMap::new(),
        }
    }

    pub fn test_failure(&mut self, test_failure: TestFailure, test_plan: &ModuleTestPlan) {
        self.failed
            .entry(test_plan.module_id.clone())
            .or_insert_with(BTreeSet::new)
            .insert(test_failure);
    }

    pub fn test_success(&mut self, test_info: TestRunInfo, test_plan: &ModuleTestPlan) {
        self.passed
            .entry(test_plan.module_id.clone())
            .or_insert_with(BTreeSet::new)
            .insert(test_info);
    }

    pub fn combine(mut self, other: Self) -> Self {
        for (module_id, test_result) in other.passed {
            let entry = self.passed.entry(module_id).or_default();
            entry.extend(test_result.into_iter());
        }
        for (module_id, test_result) in other.failed {
            let entry = self.failed.entry(module_id).or_default();
            entry.extend(test_result.into_iter());
        }
        self
    }
}

impl TestResults {
    pub fn new(final_statistics: TestStatistics, test_plan: TestPlan) -> Self {
        Self {
            final_statistics,
            test_plan,
        }
    }

    pub fn report_statistics<W: Write>(&self, writer: &Mutex<W>) -> Result<()> {
        writeln!(writer.lock().unwrap(), "\nTest Statistics:\n")?;

        let mut max_function_name_size = 0;
        let mut stats = Vec::new();

        for (module_id, test_results) in self.final_statistics.passed.iter() {
            for test_result in test_results {
                let qualified_function_name = format!(
                    "{}::{}",
                    format_module_id(module_id),
                    test_result.function_ident
                );
                max_function_name_size =
                    std::cmp::max(max_function_name_size, qualified_function_name.len());
                stats.push((
                    qualified_function_name,
                    test_result.elapsed_time.as_secs_f32(),
                    test_result.instructions_executed,
                ))
            }
        }

        for (module_id, test_failures) in self.final_statistics.failed.iter() {
            for test_failure in test_failures {
                let qualified_function_name = format!(
                    "{}::{}",
                    format_module_id(module_id),
                    test_failure.test_run_info.function_ident
                );
                max_function_name_size =
                    std::cmp::max(max_function_name_size, qualified_function_name.len());
                stats.push((
                    qualified_function_name,
                    test_failure.test_run_info.elapsed_time.as_secs_f32(),
                    test_failure.test_run_info.instructions_executed,
                ));
            }
        }

        if !stats.is_empty() {
            writeln!(
                writer.lock().unwrap(),
                "┌─{:─^width$}─┬─{:─^10}─┬─{:─^25}─┐",
                "",
                "",
                "",
                width = max_function_name_size,
            )?;
            writeln!(
                writer.lock().unwrap(),
                "│ {name:^width$} │ {time:^10} │ {instructions:^25} │",
                width = max_function_name_size,
                name = "Test Name",
                time = "Time",
                instructions = "Instructions Executed"
            )?;

            for (qualified_function_name, time, instructions) in stats {
                writeln!(
                    writer.lock().unwrap(),
                    "├─{:─^width$}─┼─{:─^10}─┼─{:─^25}─┤",
                    "",
                    "",
                    "",
                    width = max_function_name_size,
                )?;
                writeln!(
                    writer.lock().unwrap(),
                    "│ {name:<width$} │ {time:^10.3} │ {instructions:^25} │",
                    name = qualified_function_name,
                    width = max_function_name_size,
                    time = time,
                    instructions = instructions,
                )?;
            }

            writeln!(
                writer.lock().unwrap(),
                "└─{:─^width$}─┴─{:─^10}─┴─{:─^25}─┘",
                "",
                "",
                "",
                width = max_function_name_size,
            )?;
        }

        writeln!(writer.lock().unwrap())
    }

    /// Returns `true` if all tests passed, `false` if there was a test failure/timeout
    pub fn summarize<W: Write>(self, writer: &Mutex<W>) -> Result<bool> {
        let num_failed_tests = self
            .final_statistics
            .failed
            .iter()
            .fold(0, |acc, (_, fns)| acc + fns.len()) as u64;
        let num_passed_tests = self
            .final_statistics
            .passed
            .iter()
            .fold(0, |acc, (_, fns)| acc + fns.len()) as u64;
        if !self.final_statistics.failed.is_empty() {
            writeln!(writer.lock().unwrap(), "\nTest failures:\n")?;
            for (module_id, test_failures) in &self.final_statistics.failed {
                writeln!(
                    writer.lock().unwrap(),
                    "Failures in {}:",
                    format_module_id(module_id)
                )?;
                for test_failure in test_failures {
                    writeln!(
                        writer.lock().unwrap(),
                        "\n┌── {} ──────",
                        test_failure.test_run_info.function_ident.bold()
                    )?;
                    writeln!(
                        writer.lock().unwrap(),
                        "│ {}",
                        test_failure
                            .render_error(&self.test_plan)
                            .replace('\n', "\n│ ")
                    )?;
                    writeln!(writer.lock().unwrap(), "└──────────────────\n")?;
                }
            }
        }

        writeln!(
            writer.lock().unwrap(),
            "Test result: {}. Total tests: {}; passed: {}; failed: {}",
            if num_failed_tests == 0 {
                "OK".bold().bright_green()
            } else {
                "FAILED".bold().bright_red()
            },
            num_passed_tests + num_failed_tests,
            num_passed_tests,
            num_failed_tests
        )?;
        Ok(num_failed_tests == 0)
    }
}
