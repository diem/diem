// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{DEFAULT_BUILD_DIR, DEFAULT_PACKAGE_DIR, DEFAULT_SOURCE_DIR, DEFAULT_STORAGE_DIR};
use anyhow::anyhow;
use move_binary_format::file_format::CompiledModule;
use move_command_line_common::{
    env::read_bool_env_var,
    files::{extension_equals, find_filenames, path_to_string, MOVE_COMPILED_EXTENSION},
    testing::{format_diff, read_env_update_baseline, EXP_EXT},
};
use move_coverage::coverage_map::{CoverageMap, ExecCoverageMapWithModules};
use move_lang::command_line::COLOR_MODE_ENV_VAR;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env,
    fs::{self, File},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::tempdir;

/// Basic datatest testing framework for the CLI. The `run_one` entrypoint expects
/// an `args.txt` file with arguments that the `move` binary understands (one set
/// of arguments per line). The testing framework runs the commands, compares the
/// result to the expected output, and runs `move clean` to discard resources,
/// modules, and event data created by running the test.

/// If this env var is set, `move clean` will not be run after each test.
/// this is useful if you want to look at the `storage` or `move_events`
/// produced by a test. However, you'll have to manually run `move clean`
/// before re-running the test.
const NO_MOVE_CLEAN: &str = "NO_MOVE_CLEAN";

/// The filename that contains the arguments to the Move binary.
pub const TEST_ARGS_FILENAME: &str = "args.txt";

/// Name of the environment variable we need to set in order to get tracing
/// enabled in the move VM.
const MOVE_VM_TRACING_ENV_VAR_NAME: &str = "MOVE_VM_TRACE";

/// The default file name (inside the build output dir) for the runtime to
/// dump the execution trace to. The trace will be used by the coverage tool
/// if --track-cov is set. If --track-cov is not set, then no trace file will
/// be produced.
const DEFAULT_TRACE_FILE: &str = "trace";

fn collect_coverage(
    trace_file: &Path,
    build_dir: &Path,
    storage_dir: &Path,
) -> anyhow::Result<ExecCoverageMapWithModules> {
    fn find_compiled_move_filenames(path: &Path) -> anyhow::Result<Vec<String>> {
        if path.exists() {
            find_filenames(&[path_to_string(path)?], |fpath| {
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
    let src_module_files = find_filenames(&[path_to_string(storage_dir)?], |fpath| {
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
    cli_binary: &Path,
    use_temp_dir: bool,
    track_cov: bool,
) -> anyhow::Result<Option<ExecCoverageMapWithModules>> {
    fn simple_copy_dir(dst: &Path, src: &Path) -> io::Result<()> {
        for entry in fs::read_dir(src)? {
            let src_entry = entry?;
            let src_entry_path = src_entry.path();
            let dst_entry_path = dst.join(src_entry.file_name());
            if src_entry_path.is_dir() {
                fs::create_dir(&dst_entry_path)?;
                simple_copy_dir(&dst_entry_path, &src_entry_path)?;
            } else {
                fs::copy(&src_entry_path, &dst_entry_path)?;
            }
        }
        Ok(())
    }

    let args_file = io::BufReader::new(File::open(args_path)?).lines();
    let cli_binary_path = cli_binary.canonicalize()?;

    // path where we will run the binary
    let exe_dir = args_path.parent().unwrap();
    let temp_dir = if use_temp_dir {
        // symlink everything in the exe_dir into the temp_dir
        let dir = tempdir()?;
        simple_copy_dir(dir.path(), exe_dir)?;
        Some(dir)
    } else {
        None
    };
    let wks_dir = temp_dir.as_ref().map_or(exe_dir, |t| t.path());

    let storage_dir = wks_dir.join(DEFAULT_STORAGE_DIR);
    let build_output = wks_dir.join(DEFAULT_BUILD_DIR);

    // template for preparing a cli command
    let cli_command_template = || {
        let mut command = Command::new(cli_binary_path.clone());
        if let Some(work_dir) = temp_dir.as_ref() {
            command.current_dir(work_dir.path());
        } else {
            command.current_dir(exe_dir);
        }
        command
    };

    if storage_dir.exists() || build_output.exists() {
        // need to clean before testing
        cli_command_template()
            .arg("sandbox")
            .arg("clean")
            .output()?;
    }
    let mut output = "".to_string();

    // always use the absolute path for the trace file as we may change dirs in the process
    let trace_file = if track_cov {
        Some(wks_dir.canonicalize()?.join(DEFAULT_TRACE_FILE))
    } else {
        None
    };

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
        match &trace_file {
            None => {
                // this check prevents cascading the coverage tracking flag.
                // in particular, if
                //   1. we run with move-cli test <path-to-args-A.txt> --track-cov, and
                //   2. in this <args-A.txt>, there is another command: test <args-B.txt>
                // then, when running <args-B.txt>, coverage will not be tracked nor printed
                env::remove_var(MOVE_VM_TRACING_ENV_VAR_NAME);
            }
            Some(path) => env::set_var(MOVE_VM_TRACING_ENV_VAR_NAME, path.as_os_str()),
        }

        let cmd_output = cli_command_template().args(args_iter).output()?;
        output += &format!("Command `{}`:\n", args_line);
        output += std::str::from_utf8(&cmd_output.stdout)?;
        output += std::str::from_utf8(&cmd_output.stderr)?;
    }

    // collect coverage information
    let cov_info = match &trace_file {
        None => None,
        Some(trace_path) => {
            if trace_path.exists() {
                Some(collect_coverage(trace_path, &build_output, &storage_dir)?)
            } else {
                eprintln!(
                    "Trace file {:?} not found: coverage is only available with at least one `run` \
                    command in the args.txt (after a `clean`, if there is one)",
                    trace_path
                );
                None
            }
        }
    };

    // post-test cleanup and cleanup checks
    // check that the test command didn't create a src dir
    let run_move_clean = !read_bool_env_var(NO_MOVE_CLEAN);
    if run_move_clean {
        // run the clean command to ensure that temporary state is cleaned up
        cli_command_template()
            .arg("sandbox")
            .arg("clean")
            .output()?;

        // check that build and storage was deleted
        assert!(
            !storage_dir.exists(),
            "`move clean` failed to eliminate {} directory",
            DEFAULT_STORAGE_DIR
        );
        assert!(
            !build_output.exists(),
            "`move clean` failed to eliminate {} directory",
            DEFAULT_BUILD_DIR
        );

        // clean the trace file as well if it exists
        if let Some(trace_path) = &trace_file {
            if trace_path.exists() {
                fs::remove_file(trace_path)?;
            }
        }
    }

    // release the temporary workspace explicitly
    if let Some(t) = temp_dir {
        t.close()?;
    }

    // compare output and exp_file
    let update_baseline = read_env_update_baseline();
    let exp_path = args_path.with_extension(EXP_EXT);
    if update_baseline {
        fs::write(exp_path, &output)?;
        return Ok(cov_info);
    }

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

pub fn run_all(
    args_path: &Path,
    cli_binary: &Path,
    use_temp_dir: bool,
    track_cov: bool,
) -> anyhow::Result<()> {
    let mut test_total: u64 = 0;
    let mut test_passed: u64 = 0;
    let mut cov_info = ExecCoverageMapWithModules::empty();

    // find `args.txt` and iterate over them
    for entry in find_filenames(&[args_path], |fpath| {
        fpath.file_name().expect("unexpected file entry path") == TEST_ARGS_FILENAME
    })? {
        match run_one(Path::new(&entry), cli_binary, use_temp_dir, track_cov) {
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

/// Create a directory scaffold for writing a Move CLI test.
pub fn create_test_scaffold(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        anyhow::bail!("{:#?} already exists. Remove {:#?} and re-run this command if creating it as a test directory was intentional.", path, path);
    }

    let dirs = [DEFAULT_SOURCE_DIR, "scripts"];
    let files = [(
        TEST_ARGS_FILENAME,
        Some("# This is a batch file. To write an expected value test that runs `move <command1> <args1>;move <command2> <args2>`, write\n\
            # `<command1> <args1>`\n\
            # `<command2> <args2>`\n\
            # '#' is a comment.",
            ),
    )];

    fs::create_dir_all(&path)?;

    for dir in &dirs {
        fs::create_dir_all(&path.canonicalize()?.join(dir))?;
    }

    for (file, possible_contents) in &files {
        let mut file_handle = fs::File::create(path.canonicalize()?.join(file))?;
        if let Some(contents) = possible_contents {
            write!(file_handle, "{}", contents)?;
        }
    }

    Ok(())
}
