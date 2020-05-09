// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use codespan_reporting::term::termcolor::Buffer;

use anyhow::anyhow;
use itertools::Itertools;
use libra_temppath::TempPath;
use move_prover::{cli::Options, run_move_prover};
use test_utils::{baseline_test::verify_or_update_baseline, extract_test_directives, read_env_var};

const STDLIB_FLAGS: &[&str] = &["--search-path=../stdlib/modules"];
const STDLIB_FLAGS_UNVERIFIED: &[&str] = &["--search-path=../stdlib/modules", "--verify=none"];
const LEGACY_STDLIB_FLAGS: &[&str] = &["--search-path=tests/sources/stdlib/modules"];

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let no_boogie = read_env_var("BOOGIE_EXE").is_empty() || read_env_var("Z3_EXE").is_empty();
    let baseline_valid =
        !no_boogie || !extract_test_directives(path, "// no-boogie-test")?.is_empty();
    let (flags, baseline_path) = get_flags(path)?;
    let mut args = vec!["mvp_test".to_string()];
    args.extend(flags);
    args.push("--verbose=warn".to_owned());
    args.push(path.to_string_lossy().to_string());

    let mut options = Options::default();
    options.initialize_from_args(&args)?;
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
        if let Some(path) = baseline_path {
            diags += &String::from_utf8_lossy(&error_writer.into_inner()).to_string();
            verify_or_update_baseline(path.as_path(), &diags)?
        } else if !diags.is_empty() {
            return Err(anyhow!(
                "Unexpected prover output (expected none): {}{}",
                diags,
                String::from_utf8_lossy(&error_writer.into_inner())
            )
            .into());
        }
    }
    Ok(())
}

fn test_runner_stdlib(path: &Path) -> datatest_stable::Result<()> {
    // Gives the standard test runner a different name in test output, which is useful because
    // the datatest infrastructure drops `..` in test file paths.
    test_runner(path)
}

fn get_flags(path: &Path) -> anyhow::Result<(Vec<String>, Option<PathBuf>)> {
    // Determine the way how to configure tests based on directory of the path.
    let path_str = path.to_string_lossy();
    let (base_flags, baseline_path) = if path_str.contains("../stdlib/") {
        (STDLIB_FLAGS_UNVERIFIED, None)
    } else if path_str.contains("tests/sources/functional/")
        || path_str.contains("tests/sources/regression/")
    {
        (STDLIB_FLAGS, Some(path.with_extension("exp")))
    } else if path_str.contains("tests/sources/stdlib/") {
        (LEGACY_STDLIB_FLAGS, Some(path.with_extension("exp")))
    } else {
        return Err(anyhow!(
            "do not know how to run tests for `{}` because its directory is not configured",
            path_str
        ));
    };
    let mut flags = base_flags.iter().map(|s| (*s).to_string()).collect_vec();
    // Add any flags specified in the source.
    flags.extend(extract_test_directives(path, "// flag:")?);
    Ok((flags, baseline_path))
}

datatest_stable::harness!(
    // Run tests for the content of our tests directory.
    test_runner,
    "tests/sources",
    r".*\.move",
    // Run tests for the content of the stdlib directory.
    test_runner_stdlib,
    "../stdlib",
    r".*\.move"
);
