// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use codespan_reporting::term::termcolor::Buffer;

use libra_temppath::TempPath;
use move_prover::{cli::Options, run_move_prover};
use test_utils::{baseline_test::verify_or_update_baseline, extract_test_directives, read_env_var};

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let no_boogie = read_env_var("BOOGIE_EXE").is_empty() || read_env_var("Z3_EXE").is_empty();
    let baseline_valid =
        !no_boogie || !extract_test_directives(path, "// no-boogie-test")?.is_empty();
    let mut sources = extract_test_directives(path, "// dep:")?;
    sources.push(path.to_string_lossy().to_string());
    // The first argument in the args list is interpreted as program name, so set something here.
    let mut args = vec!["mvp_test".to_string()];
    args.extend_from_slice(&sources);
    let mut options = Options::default();
    options.initialize_from_args(&args);
    options.setup_logging_for_test();
    if no_boogie {
        options.generate_only = true;
    }
    options.stable_test_output = true;

    let temp_path = TempPath::new();
    temp_path.create_as_dir()?;
    let base_name = format!("{}.bpl", path.file_stem().unwrap().to_str().unwrap());
    options.output_path = temp_path
        .path()
        .join(base_name)
        .to_str()
        .unwrap()
        .to_string();

    let mut error_writer = Buffer::no_color();
    let mut diags = match run_move_prover(&mut error_writer, options) {
        Ok(()) => "".to_string(),
        Err(err) => format!("Move prover returns: {}\n", err),
    };
    if baseline_valid {
        diags += &String::from_utf8_lossy(&error_writer.into_inner()).to_string();
        if diags.is_empty() {
            diags = "Move prover all good, no errors!".to_string()
        };
        let baseline_path = path.with_extension("exp");
        verify_or_update_baseline(baseline_path.as_path(), &diags)?;
    }
    Ok(())
}

datatest_stable::harness!(test_runner, "tests/sources", r".*\.move");
