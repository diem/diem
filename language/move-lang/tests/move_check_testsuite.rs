// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_lang::{
    command_line::read_bool_env_var,
    errors::*,
    move_continue_up_to, move_parse, parser,
    shared::{CompilationEnv, Flags},
    unit_test, CommentMap, Pass, PassResult,
};
use move_lang_test_utils::*;
use std::{fs, path::Path};

const OUT_EXT: &str = "out";
const EXP_EXT: &str = "exp";
const TEST_EXT: &str = "unit_test";

const KEEP_TMP: &str = "KEEP";
const UPDATE_BASELINE: &str = "UPDATE_BASELINE";
const UB: &str = "UB";

fn format_diff(expected: String, actual: String) -> String {
    use difference::*;

    let changeset = Changeset::new(&expected, &actual, "\n");

    let mut ret = String::new();

    for seq in changeset.diffs {
        match &seq {
            Difference::Same(x) => {
                ret.push_str(x);
                ret.push('\n');
            }
            Difference::Add(x) => {
                ret.push_str("\x1B[92m");
                ret.push_str(x);
                ret.push_str("\x1B[0m");
                ret.push('\n');
            }
            Difference::Rem(x) => {
                ret.push_str("\x1B[91m");
                ret.push_str(x);
                ret.push_str("\x1B[0m");
                ret.push('\n');
            }
        }
    }
    ret
}

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
    run_test(path, &exp_path, &out_path, Flags::empty())
}

// Runs all tests under the test/testsuite directory.
fn run_test(
    path: &Path,
    exp_path: &Path,
    out_path: &Path,
    flags: Flags,
) -> datatest_stable::Result<()> {
    let targets: Vec<String> = vec![path.to_str().unwrap().to_owned()];
    let deps = move_stdlib::move_stdlib_files();

    let (files, pprog_and_comments_res) = move_parse(&targets, &deps, None, false)?;
    let errors = match move_check_for_errors(pprog_and_comments_res, flags) {
        Err(errors) | Ok(errors) => errors,
    };

    let has_errors = !errors.is_empty();
    let error_buffer = if has_errors {
        move_lang::errors::report_errors_to_buffer(files, errors)
    } else {
        vec![]
    };

    let save_errors = read_bool_env_var(KEEP_TMP);
    let update_baseline = read_bool_env_var(UPDATE_BASELINE) || read_bool_env_var(UB);

    fs::write(out_path, error_buffer)?;
    let rendered_errors = fs::read_to_string(out_path)?;
    if !save_errors {
        fs::remove_file(out_path)?;
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
            error(msg)
        }
        (false, true) => {
            let msg = format!(
                "Unexpected success. Expected errors:\n{}",
                fs::read_to_string(exp_path)?
            );
            error(msg)
        }
        (true, true) => {
            let expected_errors = fs::read_to_string(exp_path)?;
            if rendered_errors != expected_errors {
                let msg = format!(
                    "Expected errors differ from actual errors:\n{}",
                    format_diff(expected_errors, rendered_errors),
                );
                error(msg)
            } else {
                Ok(())
            }
        }
    }
}

fn move_check_for_errors(
    pprog_and_comments_res: Result<(CommentMap, parser::ast::Program), Errors>,
    flags: Flags,
) -> Result<Errors, Errors> {
    let mut compilation_env = CompilationEnv::new(flags);
    let (_, prog) = pprog_and_comments_res?;
    let cfgir = match move_continue_up_to(
        &mut compilation_env,
        None,
        PassResult::Parser(prog),
        Pass::CFGIR,
    )? {
        PassResult::CFGIR(cfgir) => cfgir,
        _ => unreachable!(),
    };

    if compilation_env.flags().is_testing() {
        unit_test::plan_builder::construct_test_plan(&mut compilation_env, &cfgir);
    }

    compilation_env.check_errors()?;

    match move_continue_up_to(
        &mut compilation_env,
        None,
        PassResult::CFGIR(cfgir),
        Pass::Compilation,
    )? {
        PassResult::Compilation(units) => Ok(move_lang::compiled_unit::verify_units(units).1),
        _ => unreachable!(),
    }
}

datatest_stable::harness!(move_check_testsuite, MOVE_CHECK_DIR, r".*\.move$");
