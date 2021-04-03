// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan_reporting::term::termcolor::Buffer;
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use move_lang::MOVE_COMPILED_EXTENSION;
use move_model::run_spec_instrumenter;
use move_prover_test_utils::baseline_test::verify_or_update_baseline;

const WORKDIR_SURFIX: &str = "workdir";
const MODULE_SUB_DIR: &str = "modules";
const MOVE_DISASSEMBLY_EXTENSION: &str = "disas";

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let targets = vec![path.to_str().unwrap().to_string()];
    let deps = vec![];
    let workdir = path.with_extension(WORKDIR_SURFIX);

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

    // find the instrumented bytecode disassembly
    let addr_dirs = fs::read_dir(workdir.join(MODULE_SUB_DIR))?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    assert_eq!(addr_dirs.len(), 1);
    let entries = fs::read_dir(&addr_dirs[0])?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    assert_eq!(entries.len(), 2);
    let disas_path = if (&entries[0]).extension().unwrap() == MOVE_COMPILED_EXTENSION {
        assert_eq!(
            (&entries[1]).extension().unwrap(),
            MOVE_DISASSEMBLY_EXTENSION
        );
        &entries[1]
    } else {
        assert_eq!(
            (&entries[0]).extension().unwrap(),
            MOVE_DISASSEMBLY_EXTENSION
        );
        assert_eq!((&entries[1]).extension().unwrap(), MOVE_COMPILED_EXTENSION);
        &entries[0]
    };

    // read and compare the disassembly
    //
    // TODO (mengxu): it might not be a good idea to directly compare the disassembly, as any change
    // to the 1) source compiler, 2) IR compiler, 3) file format, 4) disassembly will change the
    // expected output, and none of the changes are in our control. But for now, lacking a better
    // way of testing, and given all other components are relatively stable, we can tolerate this.
    let disas_output = fs::read_to_string(disas_path)?;

    // clean up the workdir before returning the diff result
    fs::remove_dir_all(workdir)?;
    verify_or_update_baseline(
        path.with_extension(MOVE_DISASSEMBLY_EXTENSION).as_path(),
        &disas_output,
    )?;
    Ok(())
}

datatest_stable::harness!(test_runner, "tests/instrumentation", r".*\.move");
