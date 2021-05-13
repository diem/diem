// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_command_line_common::{
    env::read_bool_env_var,
    testing::{format_diff, read_env_update_baseline, EXP_EXT, OUT_EXT},
};
use move_lang::{
    errors::*, shared::Flags, unit_test, CommentMap, Compiler, SteppedCompiler, PASS_CFGIR,
    PASS_PARSER,
};
use move_lang_test_utils::*;
use std::{fs, path::Path};

/// Shared flag to keep any temporary results of the test
const KEEP_TMP: &str = "KEEP";

const TEST_EXT: &str = "unit_test";

fn move_check_testsuite(path: &Path) -> datatest_stable::Result<()> {
    // A test is marked that it should also be compiled in test mode by having a `path.unit_test`
    // file.
    if path.with_extension(TEST_EXT).exists() {
        let test_exp_path = format!(
            "{}.unit_test.{}",
            path.with_extension("").to_string_lossy(),
            EXP_EXT
        );
        let test_out_path = format!(
            "{}.unit_test.{}",
            path.with_extension("").to_string_lossy(),
            OUT_EXT
        );
        run_test(
            path,
            &Path::new(&test_exp_path),
            &Path::new(&test_out_path),
            Flags::testing(),
        )?;
    }

    let exp_path = path.with_extension(EXP_EXT);
    let out_path = path.with_extension(OUT_EXT);
    run_test(path, &exp_path, &out_path, Flags::empty())?;
    Ok(())
}

// Runs all tests under the test/testsuite directory.
fn run_test(path: &Path, exp_path: &Path, out_path: &Path, flags: Flags) -> anyhow::Result<()> {
    let targets: Vec<String> = vec![path.to_str().unwrap().to_owned()];
    let mut deps = move_stdlib::move_stdlib_files();
    deps.extend(move_stdlib::unit_testing_files());

    let (files, comments_and_compiler_res) = Compiler::new(&targets, &deps)
        .set_flags(flags)
        .run::<PASS_PARSER>()?;
    let errors = match move_check_for_errors(comments_and_compiler_res) {
        Err(errors) | Ok(errors) => errors,
    };

    let has_errors = !errors.is_empty();
    let error_buffer = if has_errors {
        move_lang::errors::report_errors_to_buffer(files, errors)
    } else {
        vec![]
    };

    let save_errors = read_bool_env_var(KEEP_TMP);
    let update_baseline = read_env_update_baseline();

    let rendered_errors = std::str::from_utf8(&error_buffer)?;
    if save_errors {
        fs::write(out_path, &error_buffer)?;
    }

    if update_baseline {
        if has_errors {
            fs::write(exp_path, rendered_errors)?;
        } else if exp_path.is_file() {
            fs::remove_file(exp_path)?;
        }
        return Ok(());
    }

    let exp_exists = exp_path.is_file();
    match (has_errors, exp_exists) {
        (false, false) => Ok(()),
        (true, false) => {
            let msg = format!("Expected success. Unexpected errors:\n{}", rendered_errors);
            anyhow::bail!(msg)
        }
        (false, true) => {
            let msg = format!(
                "Unexpected success. Expected errors:\n{}",
                fs::read_to_string(exp_path)?
            );
            anyhow::bail!(msg)
        }
        (true, true) => {
            let expected_errors = fs::read_to_string(exp_path)?;
            if rendered_errors != expected_errors {
                let msg = format!(
                    "Expected errors differ from actual errors:\n{}",
                    format_diff(expected_errors, rendered_errors),
                );
                anyhow::bail!(msg)
            } else {
                Ok(())
            }
        }
    }
}

fn move_check_for_errors(
    comments_and_compiler_res: Result<(CommentMap, SteppedCompiler<'_, PASS_PARSER>), Errors>,
) -> Result<Errors, Errors> {
    let (_, compiler) = comments_and_compiler_res?;
    let (mut compiler, cfgir) = compiler.run::<PASS_CFGIR>()?.into_ast();
    let compilation_env = compiler.compilation_env();
    if compilation_env.flags().is_testing() {
        unit_test::plan_builder::construct_test_plan(compilation_env, &cfgir);
    }

    compilation_env.check_errors()?;
    let units = compiler.at_cfgir(cfgir).build()?;
    Ok(move_lang::compiled_unit::verify_units(units).1)
}

datatest_stable::harness!(move_check_testsuite, MOVE_CHECK_DIR, r".*\.move$");
