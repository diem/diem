use anyhow::format_err;
use move_testing_flow::entry::run_move_expected_output_test;
use std::path::Path;

fn run_test(path: &Path) -> datatest_stable::Result<()> {
    match run_move_expected_output_test(path) {
        Some(err_msg) => Err(format_err!("\n{}", err_msg).into()),
        None => Ok(()),
    }
}

datatest_stable::harness!(run_test, "tests", r".*\.yaml");
