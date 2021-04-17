// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use bytecode::pipeline_factory;
use bytecode_interpreter::{interpret, InterpreterOptions};
use move_model::run_model_builder;
use move_prover_test_utils::{baseline_test::verify_or_update_baseline, read_bool_env_var};

const ASM_EXP_EXTENSION: &str = "asm.exp";
const WORKDIR_EXTENSION: &str = "workdir";

fn prepare_workdir(path: &Path) -> io::Result<PathBuf> {
    let workdir = path.with_extension(WORKDIR_EXTENSION);
    if workdir.exists() {
        if workdir.is_dir() {
            fs::remove_dir_all(&workdir)?;
        } else {
            fs::remove_file(&workdir)?;
        }
    }
    fs::create_dir_all(&workdir)?;
    Ok(workdir)
}

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    // build the move model
    let srcs = vec![path.to_str().unwrap().to_string()];
    let env = run_model_builder(&srcs, &[])?;

    // short-circuit the test if there is any error
    assert!(!env.has_errors());
    assert!(!env.has_warnings());

    // create a workdir for dumping of needed
    let workdir = prepare_workdir(path)?;

    // run the interpreter
    let stepwise = read_bool_env_var("STEPWISE");
    let options = InterpreterOptions {
        stepwise,
        dump_workdir: Some(workdir.clone().into_os_string().into_string().unwrap()),
    };
    let pipeline = pipeline_factory::default_pipeline();
    interpret(&options, &env, pipeline)?;

    // make sure that there is no errors in the process
    assert!(!env.has_errors());
    assert!(!env.has_warnings());

    if !stepwise {
        // grab the final assembly before cleaning the workdir
        let generated_asm = fs::read_to_string(workdir.join("0_final.asm"))?;
        fs::remove_dir_all(workdir)?;

        // compare the final assembly matches with expectation
        verify_or_update_baseline(
            path.with_extension(ASM_EXP_EXTENSION).as_path(),
            &generated_asm,
        )?;
    }

    Ok(())
}

datatest_stable::harness!(test_runner, "tests", r".*\.move");
