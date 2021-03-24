// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use codespan_reporting::term::termcolor::Buffer;

use anyhow::anyhow;
use diem_temppath::TempPath;
use itertools::Itertools;
use move_prover::{cli::Options, run_move_prover};
use move_prover_test_utils::{
    baseline_test::verify_or_update_baseline, extract_test_directives, read_env_var,
};

use datatest_stable::Requirements;
#[allow(unused_imports)]
use log::{debug, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};

const ENV_FLAGS: &str = "MVP_TEST_FLAGS";
const ENV_TEST_EXTENDED: &str = "MVP_TEST_X";
const MVP_TEST_INCONSISTENCY: &str = "MVP_TEST_INCONSISTENCY";
const INCONSISTENCY_TEST_FLAGS: &[&str] = &[
    "--dependency=../move-stdlib/modules",
    "--dependency=../diem-framework/modules",
    "--check-inconsistency",
];
const REGULAR_TEST_FLAGS: &[&str] = &[
    "--dependency=../move-stdlib/modules",
    "--dependency=../diem-framework/modules",
];

static NOT_CONFIGURED_WARNED: AtomicBool = AtomicBool::new(false);

/// A list of different feature variants to test. Each test will be executed for each variant,
/// unless it (a) contains a line comment of the form `// exclude_for: <feature1> <feature2> ...` (b)
/// its path is in the blacklisted paths specified.
/// TODO: currently only default is active, as vnext is too unstable for release via sporadic
///   timeouts.
const TESTED_FEATURES: &[&str] = &["default" /*, "vnext"*/];
const DENYLISTED_PATHS: &[(&str, &str)] = &[];

/// Determines the runner based on feature name. Because the datatest framework requires a
/// function pointer (not a closure) to set up a test, we need to have a specific static function
/// for each feature.
fn get_runner(feature: &str) -> fn(&Path) -> datatest_stable::Result<()> {
    fn runner_default(p: &Path) -> datatest_stable::Result<()> {
        test_runner_variant(p, "default", "")
    }
    fn runner_vnext(p: &Path) -> datatest_stable::Result<()> {
        test_runner_variant(p, "vnext", "--vnext")
    }
    match feature {
        "default" => runner_default,
        "vnext" => runner_vnext,
        _ => panic!("unknown feature"),
    }
}

fn test_runner_variant(
    path: &Path,
    feature: &str,
    feature_flags: &str,
) -> datatest_stable::Result<()> {
    // Use the below + `cargo test -- --test-threads=1` to identify a long running test
    //println!(">>> testing {}", path.to_string_lossy().to_string());

    // Determine whether the test is excluded for the given feature.
    let feature_exclusion = extract_test_directives(path, "// exclude_for: ")?;
    if feature_exclusion.contains(&feature.to_string()) {
        info!("skipping {} for feature `{}`", path.display(), feature);
        return Ok(());
    }
    for (feature_name, path_frag) in DENYLISTED_PATHS {
        if feature == *feature_name && path.to_string_lossy().to_string().contains(path_frag) {
            info!("skipping {} for feature `{}`", path.display(), feature);
            return Ok(());
        }
    }
    info!(
        "testing {} with feature `{}` (flags = `{}`)",
        path.display(),
        feature,
        feature_flags
    );

    let no_boogie = read_env_var("BOOGIE_EXE").is_empty() || read_env_var("Z3_EXE").is_empty();
    let baseline_valid =
        !no_boogie || !extract_test_directives(path, "// no-boogie-test")?.is_empty();

    let temp_dir = TempPath::new();
    std::fs::create_dir_all(temp_dir.path())?;
    let (flags, baseline_path) = get_flags(temp_dir.path(), path)?;

    let mut args = vec!["mvp_test".to_string()];
    args.extend(flags);
    args.push("--verbose=warn".to_owned());
    // TODO: timeouts aren't handled correctly by the boogie wrapper but lead to hang. Determine
    //   reasons and reactivate.
    // args.push("--num-instances=2".to_owned()); // run two Boogie instances with different seeds
    // args.push("--sequential".to_owned());
    args.push(path.to_string_lossy().to_string());

    args.extend(shell_words::split(feature_flags)?);
    args.extend(shell_words::split(&read_env_var(ENV_FLAGS))?);

    let mut options = Options::create_from_args(&args)?;
    options.setup_logging_for_test();
    if no_boogie {
        options.prover.generate_only = true;
        if NOT_CONFIGURED_WARNED
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            warn!(
                "Prover tools are not configured, verification tests will be skipped. \
        See https://github.com/diem/diem/tree/main/language/move-prover/doc/user/install.md \
        for instructions."
            );
        }
    }
    options.prover.stable_test_output = true;
    options.backend.stable_test_output = true;

    let mut error_writer = Buffer::no_color();
    let mut diags = match run_move_prover(&mut error_writer, options) {
        Ok(()) => "".to_string(),
        Err(err) => format!("Move prover returns: {}\n", err),
    };
    if baseline_valid {
        if let Some(ref path) = baseline_path {
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

fn get_flags(temp_dir: &Path, path: &Path) -> anyhow::Result<(Vec<String>, Option<PathBuf>)> {
    // Determine the way how to configure tests based on directory of the path.
    let path_str = path.to_string_lossy();

    let stdlib_test_flags = if read_env_var(MVP_TEST_INCONSISTENCY).is_empty() {
        REGULAR_TEST_FLAGS
    } else {
        INCONSISTENCY_TEST_FLAGS
    };

    let (base_flags, baseline_path, modifier) =
        if path_str.contains("diem-framework/") || path_str.contains("move-stdlib/") {
            (stdlib_test_flags, None, "std_")
        } else {
            (
                REGULAR_TEST_FLAGS,
                Some(path.with_extension("exp")),
                "prover_",
            )
        };
    let mut flags = base_flags.iter().map(|s| (*s).to_string()).collect_vec();
    // Add any flags specified in the source.
    flags.extend(extract_test_directives(path, "// flag:")?);

    // Create a temporary file for output. We inject the modifier to potentially prevent
    // any races between similar named files in different directories, as it appears TempPath
    // isn't working always.
    let base_name = format!(
        "{}{}.bpl",
        modifier,
        path.file_stem().unwrap().to_str().unwrap()
    );
    let output = temp_dir.join(base_name).to_str().unwrap().to_string();
    flags.push(format!("--output={}", output));
    Ok((flags, baseline_path))
}

// Test entry point based on datatest runner.
fn main() {
    let mut reqs = vec![];
    for feature in TESTED_FEATURES {
        let runner = get_runner(*feature);
        if read_env_var(ENV_TEST_EXTENDED) == "1" {
            reqs.push(Requirements::new(
                runner,
                format!("extended[{}]", feature),
                "tests/xsources".to_string(),
                r".*\.move$".to_string(),
            ));
        } else {
            reqs.push(Requirements::new(
                runner,
                format!("unit[{}]", feature),
                "tests/sources".to_string(),
                r".*\.move$".to_string(),
            ));
            reqs.push(Requirements::new(
                runner,
                format!("stdlib[{}]", feature),
                "../move-stdlib".to_string(),
                r".*\.move$".to_string(),
            ));
            reqs.push(Requirements::new(
                runner,
                format!("diem[{}]", feature),
                "../diem-framework".to_string(),
                r".*\.move$".to_string(),
            ));
        }
    }
    datatest_stable::runner(&reqs);
}
