// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{DEFAULT_BUILD_DIR, DEFAULT_STORAGE_DIR};
use move_coverage::{coverage_map::CoverageMap, summary::summarize_inst_cov};
use move_lang::test_utils::*;
use std::{
    env,
    fs::{self, File},
    io::{self, BufRead, Write},
    path::Path,
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

/// The default file name for the runtime to dump the execution trace to.
/// The trace will be used by the coverage tool if --track-cov is set.
/// If --track-cov is not set, then no trace file will be produced.
const DEFAULT_TRACE_FILE: &str = "trace";

fn format_diff(expected: String, actual: String) -> String {
    use difference::*;

    let changeset = Changeset::new(&expected, &actual, "\n");

    let mut ret = String::new();

    for seq in changeset.diffs {
        match &seq {
            Difference::Same(x) => {
                ret.push_str(x);
                ret.push_str("\n");
            }
            Difference::Add(x) => {
                ret.push_str("\x1B[92m");
                ret.push_str(x);
                ret.push_str("\x1B[0m");
                ret.push_str("\n");
            }
            Difference::Rem(x) => {
                ret.push_str("\x1B[91m");
                ret.push_str(x);
                ret.push_str("\x1B[0m");
                ret.push_str("\n");
            }
        }
    }
    ret
}

fn show_coverage(trace_file: &Path, storage_dir: &Path) -> anyhow::Result<()> {
    // collect modules
    let mut modules: Vec<CompiledModule> = Vec::new();
    for entry in move_lang::find_filenames(&[storage_dir.to_str().unwrap().to_owned()], |fpath| {
        fpath.extension().map_or(false, |e| e == "mv")
    })? {
        let bytecode_bytes = fs::read(entry).unwrap();
        let compiled_module = CompiledModule::deserialize(&bytecode_bytes).unwrap();
        modules.push(compiled_module);
    }

    // collect trace
    let coverage_map = CoverageMap::from_trace_file(trace_file).to_unified_exec_map();

    // summarize
    let mut summary_writer: Box<dyn Write> = Box::new(io::stdout());
    for module in modules.iter() {
        let module_summary = summarize_inst_cov(module, &coverage_map);
        module_summary.summarize_human(&mut summary_writer, true)?;
    }

    Ok(())
}

/// Run the `args_path` batch file with`cli_binary`
pub fn run_one(args_path: &Path, cli_binary: &str, track_cov: bool) -> anyhow::Result<()> {
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
        // for tracing file path: always use the absolute path so we do not need
        // to worry about where the VM is executed.
        let mut trace_file = env::current_dir()?;
        trace_file.push(&storage_dir);
        trace_file.push(DEFAULT_TRACE_FILE);
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

        // show coverage information
        if track_cov && trace_file.exists() {
            if !trace_file.exists() {
                eprintln!("Trace file {} not found", trace_file.to_string_lossy());
                eprintln!("Coverage is only applicable to the RUN command in args.txt");
            } else {
                show_coverage(Path::new(&trace_file), Path::new(&storage_dir))?;
            }
        }
    }

    // post-test cleanup and cleanup checks
    // check that the test command didn't create a src dir

    let run_move_clean = !read_bool_var(NO_MOVE_CLEAN);
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

    let update_baseline = read_bool_var(UPDATE_BASELINE) || read_bool_var(UB);
    let exp_path = args_path.with_extension(EXP_EXT);
    if update_baseline {
        fs::write(exp_path, &output)?;
        return Ok(());
    }
    // compare output and exp_file
    let expected_output = fs::read_to_string(exp_path).unwrap_or_else(|_| "".to_string());
    if expected_output != output {
        anyhow::bail!(
            "Expected output differs from actual output:\n{}",
            format_diff(expected_output, output)
        )
    } else {
        Ok(())
    }
}

pub fn run_all(args_path: &str, cli_binary: &str, track_cov: bool) -> anyhow::Result<()> {
    let mut test_total = 0;
    let mut test_passed = 0;
    for entry in move_lang::find_filenames(&[args_path.to_owned()], |fpath| {
        fpath.file_name().expect("unexpected file entry path") == "args.txt"
    })? {
        match run_one(Path::new(&entry), cli_binary, track_cov) {
            Ok(_) => test_passed += 1,
            Err(ex) => eprintln!("Test {} failed with error: {}", entry, ex),
        }
        test_total += 1;
    }
    println!("{} / {} test(s) passed.", test_total, test_passed);

    // if any test fails, bail
    let test_failed = test_total - test_passed;
    if test_failed != 0 {
        anyhow::bail!("{} / {} test(s) failed.", test_failed, test_total)
    } else {
        Ok(())
    }
}
