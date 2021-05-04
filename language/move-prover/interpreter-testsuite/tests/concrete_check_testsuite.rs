// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::{env, path::Path};

use move_prover_test_utils::{
    baseline_test::{diff, verify_or_update_baseline},
    read_bool_env_var,
};
use move_unit_test::UnitTestingConfig;

const BASELINE_EXTENSION: &str = "exp";

fn run_unit_test(path: &Path, use_stackless_vm: bool) -> Result<String> {
    let config = UnitTestingConfig {
        instruction_execution_bound: 5000,
        filter: None,
        num_threads: 1,
        source_files: vec![path.to_str().unwrap().to_owned()],
        use_stackless_vm,
        verbose: read_bool_env_var("VERBOSE"),
    };

    let test_plan = config.build_test_plan().unwrap();
    let mut buffer = vec![];
    config.run_and_report_unit_tests(test_plan, &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    env::set_var("NO_COLOR", "1");
    // first do a cross-vm comparison
    let move_vm_output = run_unit_test(path, false)?;
    let stackless_vm_output = run_unit_test(path, true)?;
    if move_vm_output != stackless_vm_output {
        diff(&move_vm_output, &stackless_vm_output)?;
    }
    // second compare against the checked-in baseline
    let baseline_path = path.with_extension(BASELINE_EXTENSION);
    verify_or_update_baseline(&baseline_path, &stackless_vm_output)?;
    Ok(())
}

datatest_stable::harness!(test_runner, "tests/concrete_check", r".*\.move$");
