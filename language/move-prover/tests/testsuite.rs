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
use walkdir::WalkDir;

const ENV_FLAGS: &str = "MVP_TEST_FLAGS";
const ENV_TEST_EXTENDED: &str = "MVP_TEST_X";
const ENV_TEST_INCONSISTENCY: &str = "MVP_TEST_INCONSISTENCY";
const ENV_TEST_FEATURE: &str = "MVP_TEST_FEATURE";
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

/// A list of different features to test.
const TESTED_FEATURES: &[(&str, InclusionMode, bool /*whether to enable in CI*/)] = &[
    (DEFAULT_FEATURE, InclusionMode::Implicit, true),
    ("cvc4", InclusionMode::Explicit, true),
];
const DEFAULT_FEATURE: &str = "default";

/// An inclusion mode. A feature may be run in one of these modes.
#[derive(Clone, Copy)]
enum InclusionMode {
    /// Only a test which has the comment `// also_include_for: <feature>` will be included.
    Explicit,
    /// Every test will be included unless it has the comment `// exclude_for: <feature>`.
    Implicit,
}

/// Determines the runner based on feature name. Because the datatest framework requires a
/// function pointer (not a closure) to set up a test, we need to have a specific static function
/// for each feature and implement our own dispatcher based on feature.
fn get_runner(feature: &str) -> fn(&Path) -> datatest_stable::Result<()> {
    fn runner_default(p: &Path) -> datatest_stable::Result<()> {
        test_runner_for_feature(p, "default", "")
    }
    fn runner_cvc4(p: &Path) -> datatest_stable::Result<()> {
        test_runner_for_feature(p, "cvc4", "--use-cvc4")
    }
    match feature {
        "default" => runner_default,
        "cvc4" => runner_cvc4,
        _ => panic!("unknown feature"),
    }
}

/// Test runner for a given feature.
fn test_runner_for_feature(
    path: &Path,
    feature: &str,
    feature_flags: &str,
) -> datatest_stable::Result<()> {
    // Use the below + `cargo test -- --test-threads=1` to identify a long running test
    //println!(">>> testing {}", path.to_string_lossy().to_string());

    info!(
        "testing {} with feature `{}` (flags = `{}`)",
        path.display(),
        feature,
        feature_flags
    );

    let temp_dir = TempPath::new();
    std::fs::create_dir_all(temp_dir.path())?;
    let (flags, baseline_path) = get_path_dependent_flags(temp_dir.path(), path, feature)?;

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
    let no_tools = read_env_var("BOOGIE_EXE").is_empty()
        || !options.backend.use_cvc4 && read_env_var("Z3_EXE").is_empty()
        || options.backend.use_cvc4 && read_env_var("CVC4_EXE").is_empty();
    let baseline_valid =
        !no_tools || !extract_test_directives(path, "// no-boogie-test")?.is_empty();

    if no_tools {
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
    options.backend.check_tool_versions()?;
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

fn get_path_dependent_flags(
    temp_dir: &Path,
    path: &Path,
    feature: &str,
) -> anyhow::Result<(Vec<String>, Option<PathBuf>)> {
    // Determine the way how to configure tests based on directory of the path.
    let path_str = path.to_string_lossy();

    let stdlib_test_flags = if read_env_var(ENV_TEST_INCONSISTENCY).is_empty() {
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
                Some(path.with_extension(if feature == DEFAULT_FEATURE {
                    "exp".to_string()
                } else {
                    format!("{}_exp", feature)
                })),
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

/// Collects the enabled tests, accumulating them as datatest requirements.
/// We collect the test data sources ourselves instead of letting datatest
/// do it because we want to select them based on enabled feature as indicated
/// in the source. We still use datatest to finally run the tests to utilize its
/// execution engine.
fn collect_enabled_tests(
    reqs: &mut Vec<Requirements>,
    group: &str,
    feature: &str,
    inclusion_mode: InclusionMode,
    in_ci: bool,
    path: &str,
) {
    let mut p = PathBuf::new(); // PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push(path);
    for e in WalkDir::new(p).min_depth(1).into_iter() {
        if let Ok(entry) = e {
            if !entry.file_name().to_string_lossy().ends_with(".move") {
                continue;
            }
            let path = entry.path();
            let mut included = match inclusion_mode {
                InclusionMode::Implicit => !extract_test_directives(path, "// exclude_for: ")
                    .unwrap_or_default()
                    .iter()
                    .any(|s| s.as_str() == feature),
                InclusionMode::Explicit => extract_test_directives(path, "// also_include_for: ")
                    .unwrap_or_default()
                    .iter()
                    .any(|s| s.as_str() == feature),
            };
            if included && read_env_var("MVP_TEST_ON_CI") == "1" {
                included = in_ci
                    && extract_test_directives(path, "// no_ci:")
                        .unwrap_or_default()
                        .is_empty();
            }
            if included {
                let runner = get_runner(feature);
                reqs.push(Requirements::new(
                    runner,
                    format!("prover {}[{}]", group, feature),
                    path.parent().unwrap().to_string_lossy().to_string(),
                    path.file_name().unwrap().to_string_lossy().to_string(),
                ));
            }
        }
    }
}

// Test entry point based on datatest runner.
fn main() {
    let mut reqs = vec![];
    for (feature, inclusion_mode, in_ci) in TESTED_FEATURES {
        // Evaluate whether the user narrowed which feature to test.
        let feature_narrow = read_env_var(ENV_TEST_FEATURE);
        if !feature_narrow.is_empty() && feature != &feature_narrow {
            continue;
        }
        // Check whether we are running extended tests
        if read_env_var(ENV_TEST_EXTENDED) == "1" {
            collect_enabled_tests(
                &mut reqs,
                "extended",
                feature,
                *inclusion_mode,
                *in_ci,
                "tests/xsources",
            );
        } else {
            collect_enabled_tests(
                &mut reqs,
                "unit",
                feature,
                *inclusion_mode,
                *in_ci,
                "tests/sources",
            );
            collect_enabled_tests(
                &mut reqs,
                "stdlib",
                feature,
                *inclusion_mode,
                *in_ci,
                "../move-stdlib",
            );
            collect_enabled_tests(
                &mut reqs,
                "diem",
                feature,
                *inclusion_mode,
                *in_ci,
                "../diem-framework",
            );
        }
    }
    datatest_stable::runner(&reqs);
}
