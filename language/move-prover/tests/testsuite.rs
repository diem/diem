// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use codespan_reporting::term::termcolor::Buffer;

use anyhow::anyhow;
use itertools::Itertools;
use libra_temppath::TempPath;
use move_prover::{cli::Options, run_move_prover};
use test_utils::{baseline_test::verify_or_update_baseline, extract_test_directives, read_env_var};

const FUNCTIONAL_FLAGS: &[&str] = &["--search_path=tests/sources/stdlib/modules", "--verify=all"];
const STDLIB_FLAGS: &[&str] = &["--search_path=tests/sources/stdlib/modules", "--verify=all"];
const NEW_STDLIB_FLAGS: &[&str] = &[
    "--search_path=tests/sources/new_stdlib/modules",
    "--verify=none",
];

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let no_boogie = read_env_var("BOOGIE_EXE").is_empty() || read_env_var("Z3_EXE").is_empty();
    let baseline_valid =
        !no_boogie || !extract_test_directives(path, "// no-boogie-test")?.is_empty();
    let (mut flags, baseline_path) = get_flags(path)?;
    if flags.iter().find(|a| a.contains("--verify")).is_none() {
        flags.push("--verify=all".to_string());
    }
    let mut args = vec!["mvp_test".to_string()];
    args.extend(flags);
    args.push(path.to_string_lossy().to_string());

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
        verify_or_update_baseline(baseline_path.as_path(), &diags)?;
    }
    Ok(())
}

fn get_flags(path: &Path) -> anyhow::Result<(Vec<String>, PathBuf)> {
    // Determine the way how to configure tests based on directory of the path.
    let path_str = path.to_string_lossy();
    let base_flags = if path_str.contains("/new_stdlib/") {
        NEW_STDLIB_FLAGS
    } else if path_str.contains("/functional/") {
        FUNCTIONAL_FLAGS
    } else if path_str.contains("/stdlib/") {
        STDLIB_FLAGS
    } else {
        return Err(anyhow!(
            "do not know how to run tests for `{}` because its directory is not configured",
            path_str
        ));
    };
    let mut flags = base_flags.iter().map(|s| (*s).to_string()).collect_vec();
    // Add any flags specified in the source.
    flags.extend(extract_test_directives(path, "// flag:")?);
    let baseline_path = path.with_extension("exp");
    Ok((flags, baseline_path))
}

datatest_stable::harness!(test_runner, "tests/sources", r".*\.move");
