// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{DEFAULT_BUILD_DIR, DEFAULT_PACKAGE_DIR, DEFAULT_STORAGE_DIR};
use anyhow::anyhow;
use move_coverage::coverage_map::{CoverageMap, ExecCoverageMapWithModules};
use move_lang::{
    command_line::{read_bool_env_var, COLOR_MODE_ENV_VAR},
    extension_equals, path_to_string, MOVE_COMPILED_EXTENSION,
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env,
    fs::{self, File},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    process::Command,
};
use vm::file_format::CompiledModule;

/// Basic datatest testing framework for the CLI. The `run_one` entrypoint expects
/// an `args.txt` file with arguments that the `move` binary understands (one set
/// of arguments per line). The testing framework runs the commands, compares the
/// result to the expected output, and runs `move clean` to discard resources,
/// modules, and event data created by running the test.

const EXP_EXT: &str = "exp";

/// If this env var is set, `move clean` will not be run after each test.
/// this is useful if you want to look at the `storage` or `move_events`
/// produced by a test. However, you'll have to manually run `move clean`
/// before re-running the test.
const NO_MOVE_CLEAN: &str = "NO_MOVE_CLEAN";
/// If either of these env vars is set, the test harness overwrites the
/// old .exp files with the output instead of checking them against the
/// output.
const UPDATE_BASELINE: &str = "UPDATE_BASELINE";
const UB: &str = "UB";

/// Name of the environment variable we need to set in order to get tracing
/// enabled in the move VM.
const MOVE_VM_TRACING_ENV_VAR_NAME: &str = "MOVE_VM_TRACE";

/// The default file name (inside the build output dir) for the runtime to
/// dump the execution trace to. The trace will be used by the coverage tool
/// if --track-cov is set. If --track-cov is not set, then no trace file will
/// be produced.
const DEFAULT_TRACE_FILE: &str = "trace";

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

fn collect_coverage(
    trace_file: &Path,
    build_dir: &Path,
    storage_dir: &Path,
) -> anyhow::Result<ExecCoverageMapWithModules> {
    fn find_compiled_move_filenames(path: &Path) -> anyhow::Result<Vec<String>> {
        if path.exists() {
            move_lang::find_filenames(&[path_to_string(path)?], |fpath| {
                extension_equals(fpath, MOVE_COMPILED_EXTENSION)
            })
        } else {
            Ok(vec![])
        }
    }

    // collect modules compiled for packages (to be filtered out)
    let pkg_modules: HashSet<_> =
        find_compiled_move_filenames(&build_dir.join(DEFAULT_PACKAGE_DIR))?
            .into_iter()
            .map(|entry| PathBuf::from(entry).file_name().unwrap().to_owned())
            .collect();

    // collect modules published minus modules compiled for packages
    let src_module_files = move_lang::find_filenames(&[path_to_string(storage_dir)?], |fpath| {
        extension_equals(fpath, MOVE_COMPILED_EXTENSION)
            && !pkg_modules.contains(fpath.file_name().unwrap())
    })?;
    let src_modules = src_module_files
        .iter()
        .map(|entry| {
            let bytecode_bytes = fs::read(entry)?;
            let compiled_module = CompiledModule::deserialize(&bytecode_bytes)
                .map_err(|e| anyhow!("Failure deserializing module {:?}: {:?}", entry, e))?;

            // use absolute path to the compiled module file
            let module_absolute_path = path_to_string(&PathBuf::from(entry).canonicalize()?)?;
            Ok((module_absolute_path, compiled_module))
        })
        .collect::<anyhow::Result<HashMap<_, _>>>()?;

    // build the filter
    let mut filter = BTreeMap::new();
    for (entry, module) in src_modules.into_iter() {
        let module_id = module.self_id();
        filter
            .entry(*module_id.address())
            .or_insert_with(BTreeMap::new)
            .insert(module_id.name().to_owned(), (entry, module));
    }

    // collect filtered trace
    let coverage_map = CoverageMap::from_trace_file(trace_file)
        .to_unified_exec_map()
        .into_coverage_map_with_modules(filter);

    Ok(coverage_map)
}

/// Run the `args_path` batch file with`cli_binary`
pub fn run_one(
    args_path: &Path,
    cli_binary: &str,
    track_cov: bool,
) -> anyhow::Result<Option<ExecCoverageMapWithModules>> {
    let args_file = io::BufReader::new(File::open(args_path)?).lines();
    // path where we will run the binary
    let exe_dir = args_path.parent().unwrap();
    let cli_binary_path = Path::new(cli_binary).canonicalize()?;
    let storage_dir = Path::new(exe_dir).join(DEFAULT_STORAGE_DIR);
    let build_output = Path::new(exe_dir).join(DEFAULT_BUILD_DIR);
    if storage_dir.exists() || build_output.exists() {
        // need to clean before testing
        Command::new(cli_binary_path.clone())
            .current_dir(exe_dir)
            .arg("clean")
            .output()?;
    }
    let mut output = "".to_string();

    // for tracing file path: always use the absolute path so we do not need to worry about where
    // the VM is executed.
    let trace_file = env::current_dir()?
        .join(&build_output)
        .join(DEFAULT_TRACE_FILE);

    // Disable colors in error reporting from the Move compiler
    env::set_var(COLOR_MODE_ENV_VAR, "NONE");
    for args_line in args_file {
        let args_line = args_line?;
        if args_line.starts_with('#') {
            // allow comments in args.txt
            continue;
        }
        let args_iter: Vec<&str> = args_line.split_whitespace().collect();
        if args_iter.is_empty() {
            // allow blank lines in args.txt
            continue;
        }

        // enable tracing in the VM by setting the env var.
        if track_cov {
            env::set_var(MOVE_VM_TRACING_ENV_VAR_NAME, trace_file.as_os_str());
        } else if env::var_os(MOVE_VM_TRACING_ENV_VAR_NAME).is_some() {
            // this check prevents cascading the coverage tracking flag.
            // in particular, if
            //   1. we run with move-cli test <path-to-args-A.txt> --track-cov, and
            //   2. in this <args-A.txt>, there is another command: test <args-B.txt>
            // then, when running <args-B.txt>, coverage will not be tracked nor printed
            env::remove_var(MOVE_VM_TRACING_ENV_VAR_NAME);
        }

        let cmd_output = Command::new(cli_binary_path.clone())
            .current_dir(exe_dir)
            .args(args_iter)
            .output()?;
        output += &format!("Command `{}`:\n", args_line);
        output += std::str::from_utf8(&cmd_output.stdout)?;
        output += std::str::from_utf8(&cmd_output.stderr)?;
    }

    // collect coverage information
    let cov_info = if track_cov && trace_file.exists() {
        if !trace_file.exists() {
            eprintln!(
                "Trace file {:?} not found: coverage is only available with at least one `run` \
                command in the args.txt (after a `clean`, if there is one)",
                trace_file
            );
            None
        } else {
            Some(collect_coverage(&trace_file, &build_output, &storage_dir)?)
        }
    } else {
        None
    };

    // post-test cleanup and cleanup checks
    // check that the test command didn't create a src dir

    let run_move_clean = !read_bool_env_var(NO_MOVE_CLEAN);
    if run_move_clean {
        // run `move clean` to ensure that temporary state is cleaned up
        Command::new(cli_binary_path)
            .current_dir(exe_dir)
            .arg("clean")
            .output()?;
        // check that storage was deleted
        assert!(
            !storage_dir.exists(),
            "`move clean` failed to eliminate {} directory",
            DEFAULT_STORAGE_DIR
        );
        assert!(
            !storage_dir.exists(),
            "`move clean` failed to eliminate {} directory",
            DEFAULT_BUILD_DIR
        );
    }

    let update_baseline = read_bool_env_var(UPDATE_BASELINE) || read_bool_env_var(UB);
    let exp_path = args_path.with_extension(EXP_EXT);
    if update_baseline {
        fs::write(exp_path, &output)?;
        return Ok(cov_info);
    }
    // compare output and exp_file
    let expected_output = fs::read_to_string(exp_path).unwrap_or_else(|_| "".to_string());
    if expected_output != output {
        anyhow::bail!(
            "Expected output differs from actual output:\n{}",
            format_diff(expected_output, output)
        )
    } else {
        Ok(cov_info)
    }
}

pub fn run_all(args_path: &str, cli_binary: &str, track_cov: bool) -> anyhow::Result<()> {
    let mut test_total: u64 = 0;
    let mut test_passed: u64 = 0;
    let mut cov_info = ExecCoverageMapWithModules::empty();

    // find `args.txt` and iterate over them
    for entry in move_lang::find_filenames(&[args_path.to_owned()], |fpath| {
        fpath.file_name().expect("unexpected file entry path") == "args.txt"
    })? {
        match run_one(Path::new(&entry), cli_binary, track_cov) {
            Ok(cov_opt) => {
                test_passed = test_passed.checked_add(1).unwrap();
                if let Some(cov) = cov_opt {
                    cov_info.merge(cov);
                }
            }
            Err(ex) => eprintln!("Test {} failed with error: {}", entry, ex),
        }
        test_total = test_total.checked_add(1).unwrap();
    }
    println!("{} / {} test(s) passed.", test_passed, test_total);

    // if any test fails, bail
    let test_failed = test_total.checked_sub(test_passed).unwrap();
    if test_failed != 0 {
        anyhow::bail!("{} / {} test(s) failed.", test_failed, test_total)
    }

    // show coverage information if requested
    if track_cov {
        let mut summary_writer: Box<dyn Write> = Box::new(io::stdout());
        for (_, module_summary) in cov_info.into_module_summaries() {
            module_summary.summarize_human(&mut summary_writer, true)?;
        }
    }

    Ok(())
}
