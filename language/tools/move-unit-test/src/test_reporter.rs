// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::format_module_id;
use colored::{control, Colorize};
use move_binary_format::errors::{Location, VMError};
use move_core_types::language_storage::ModuleId;
use move_lang::{
    errors,
    unit_test::{ModuleTestPlan, TestPlan},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Result, Write},
    sync::Mutex,
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
    // The test failed for some unknown reason. This shouldn't be encountered
    Unknown(String),
}

#[derive(Debug, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct TestFailure {
    pub function_ident: String,
    pub vm_error: Option<VMError>,
    pub failure_reason: FailureReason,
    pub storage_state: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TestStatistics {
    passed_tests: u64,
    failed_tests: BTreeMap<ModuleId, BTreeSet<TestFailure>>,
}

#[derive(Debug, Clone)]
pub struct TestResults {
    final_statistics: TestStatistics,
    test_plan: TestPlan,
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

    pub fn unknown() -> Self {
        FailureReason::Unknown("ITE: An unknown error was reported.".to_string())
    }
}

impl TestFailure {
    pub fn new(failure_reason: FailureReason, fname: &str, vm_error: Option<VMError>) -> Self {
        Self {
            failure_reason,
            function_ident: fname.to_owned(),
            vm_error,
            storage_state: None,
        }
    }

    pub fn render_error(&self, test_plan: &TestPlan) -> String {
        match &self.failure_reason {
            FailureReason::NoAbort(message) => message.to_string(),
            FailureReason::Timeout(message) => message.to_string(),
            FailureReason::WrongAbort(message, expected_code, other_code) => {
                let base_message = format!(
                    "{}. Expected test to abort with {} but instead it aborted with {} here",
                    message, expected_code, other_code,
                );
                Self::report_abort(test_plan, base_message, &self.vm_error)
            }
            FailureReason::Aborted(message, code) => {
                let base_message = format!("{} but it aborted with {} here", message, code);
                Self::report_abort(test_plan, base_message, &self.vm_error)
            }
            FailureReason::Unknown(message) => {
                format!(
                    "{}. VMError (if there is one) is: {}",
                    message,
                    self.vm_error
                        .as_ref()
                        .map(|err| format!("{:#?}", err))
                        .unwrap_or_else(|| "".to_string())
                )
            }
        }
    }

    fn report_abort(
        test_plan: &TestPlan,
        base_message: String,
        vm_error: &Option<VMError>,
    ) -> String {
        let report_error = if control::SHOULD_COLORIZE.should_colorize() {
            errors::report_errors_to_color_buffer
        } else {
            errors::report_errors_to_buffer
        };

        let vm_error = match vm_error {
            None => return base_message,
            Some(vm_error) => vm_error,
        };

        match vm_error.location() {
            Location::Module(module_id) => {
                let errors = vm_error
                    .offsets()
                    .iter()
                    .filter_map(|(fdef_idx, offset)| {
                        let function_source_map = test_plan
                            .module_info
                            .get(&module_id)?
                            .1
                            .get_function_source_map(*fdef_idx)
                            .ok()?;
                        let loc = function_source_map.get_code_location(*offset)?;
                        let msg = format!("In this function in {}", format_module_id(module_id));
                        Some(vec![
                            (loc, base_message.clone()),
                            (function_source_map.decl_location, msg),
                        ])
                    })
                    .collect::<Vec<_>>();

                String::from_utf8(report_error(test_plan.files.clone(), errors)).unwrap()
            }
            _ => base_message,
        }
    }
}

impl TestStatistics {
    pub fn new() -> Self {
        Self {
            passed_tests: 0,
            failed_tests: BTreeMap::new(),
        }
    }

    pub fn test_failure(&mut self, test_failure: TestFailure, test_plan: &ModuleTestPlan) {
        self.failed_tests
            .entry(test_plan.module_id.clone())
            .or_insert_with(BTreeSet::new)
            .insert(test_failure);
    }

    pub fn test_success(&mut self) {
        self.passed_tests += 1;
    }

    pub fn combine(mut self, other: Self) -> Self {
        self.passed_tests += other.passed_tests;
        for (module_id, failed) in other.failed_tests {
            let entry = self.failed_tests.entry(module_id).or_default();
            entry.extend(failed.into_iter());
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

    pub fn summarize<W: Write>(self, writer: &Mutex<W>) -> Result<()> {
        let num_failed_tests = self
            .final_statistics
            .failed_tests
            .iter()
            .fold(0, |acc, (_, fns)| acc + fns.len()) as u64;
        if !self.final_statistics.failed_tests.is_empty() {
            writeln!(writer.lock().unwrap(), "\nTest failures:\n")?;
            for (module_id, test_failures) in &self.final_statistics.failed_tests {
                writeln!(
                    writer.lock().unwrap(),
                    "Failures in {}:",
                    format_module_id(&module_id)
                )?;
                for test_failure in test_failures {
                    writeln!(
                        writer.lock().unwrap(),
                        "\n┌── {} ──────",
                        test_failure.function_ident.bold()
                    )?;
                    writeln!(
                        writer.lock().unwrap(),
                        "│ {}",
                        test_failure
                            .render_error(&self.test_plan)
                            .replace("\n", "\n│ ")
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
            self.final_statistics.passed_tests + num_failed_tests,
            self.final_statistics.passed_tests,
            num_failed_tests
        )?;
        Ok(())
    }
}
