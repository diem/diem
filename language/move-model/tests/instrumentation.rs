// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan_reporting::term::termcolor::Buffer;
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use move_model::run_spec_instrumenter;
use move_prover_test_utils::baseline_test::verify_or_update_baseline;

const WORKDIR_SUFFIX: &str = "workdir";
const FILE_NEW_AST: &str = "program.ast.new";
const EXP_EXTENSION: &str = "exp";

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let targets = vec![path.to_str().unwrap().to_string()];
    let deps = vec![];
    let workdir = path.with_extension(WORKDIR_SUFFIX);

    let env = run_spec_instrumenter(
        targets,
        deps,
        workdir.to_str().unwrap(),
        /* dump_bytecode */ true,
    )?;

    // short-circuit the test if there is any error
    if env.has_errors() || env.has_warnings() {
        let mut writer = Buffer::ansi();
        env.report_errors(&mut writer);
        env.report_warnings(&mut writer);
        io::stderr().lock().write_all(writer.as_slice())?;
        unreachable!()
    }

    // read the generated AST file and clean up the workdir before diffing
    let generated_ast = fs::read_to_string(workdir.join(FILE_NEW_AST))?;
    fs::remove_dir_all(workdir)?;

    // compare or update the AST of the instrumented program with the expected file
    verify_or_update_baseline(path.with_extension(EXP_EXTENSION).as_path(), &generated_ast)?;
    Ok(())
}

datatest_stable::harness!(test_runner, "tests/instrumentation", r".*\.move$");
