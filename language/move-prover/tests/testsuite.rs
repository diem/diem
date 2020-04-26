// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::{Path, PathBuf};

use codespan_reporting::term::termcolor::Buffer;

use anyhow::anyhow;
use libra_temppath::TempPath;
use move_prover::{cli::Options, run_move_prover};
use std::collections::{BTreeMap, BTreeSet};
use test_utils::{
    baseline_test::verify_or_update_baseline, extract_matches, extract_test_directives,
    read_env_var,
};

const FUNCTIONAL_SEARCH_PATH: &[&str] = &["tests/sources/stdlib/modules"];
const FUNCTIONAL_FLAGS: &[&str] = &["--verify=all"];
const STDLIB_SEARCH_PATH: &[&str] = &["tests/sources/stdlib/modules"];
const STDLIB_FLAGS: &[&str] = &["--verify=all"];
const NEW_STDLIB_SEARCH_PATH: &[&str] = &["tests/sources/new_stdlib/modules"];
const NEW_STDLIB_FLAGS: &[&str] = &["--verify=none"];

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let no_boogie = read_env_var("BOOGIE_EXE").is_empty() || read_env_var("Z3_EXE").is_empty();
    let baseline_valid =
        !no_boogie || !extract_test_directives(path, "// no-boogie-test")?.is_empty();
    let (mut flags, deps, baseline_path) = get_flags_and_deps(path)?;
    if flags.iter().find(|a| a.contains("--verify")).is_none() {
        flags.push("--verify=all".to_string());
    }
    let mut args = vec!["mvp_test".to_string()];
    args.extend(flags);
    for dep in deps {
        args.push("--dep".to_string());
        args.push(dep);
    }
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

fn get_flags_and_deps(path: &Path) -> anyhow::Result<(Vec<String>, Vec<String>, PathBuf)> {
    // Determine the way how to configure tests based on directory of the path.
    let path_str = path.to_string_lossy();
    let mut flags = extract_test_directives(path, "// flag:")?;
    let baseline_path = path.with_extension("exp");
    if path_str.contains("/new_stdlib/") {
        let deps = calculate_deps(path, NEW_STDLIB_SEARCH_PATH)?;
        flags.extend(NEW_STDLIB_FLAGS.iter().map(|s| (*s).to_string()));
        Ok((flags, deps, baseline_path))
    } else if path_str.contains("/functional") {
        let deps = calculate_deps(path, FUNCTIONAL_SEARCH_PATH)?;
        flags.extend(FUNCTIONAL_FLAGS.iter().map(|s| (*s).to_string()));
        Ok((flags, deps, baseline_path))
    } else if path_str.contains("/stdlib") {
        let deps = calculate_deps(path, STDLIB_SEARCH_PATH)?;
        flags.extend(STDLIB_FLAGS.iter().map(|s| (*s).to_string()));
        Ok((flags, deps, baseline_path))
    } else {
        Err(anyhow!(
            "do not know how to run tests for `{}` because its directory is not configured",
            path_str
        ))
    }
}

fn calculate_deps(path: &Path, search_path: &[&str]) -> anyhow::Result<Vec<String>> {
    let file_map = calculate_file_map(search_path)?;
    let mut visited = BTreeSet::new();
    let mut deps = vec![];
    calculate_transitive_deps(path, &file_map, &mut visited, &mut deps)?;
    Ok(deps)
}

fn calculate_transitive_deps(
    path: &Path,
    file_map: &BTreeMap<String, PathBuf>,
    visited: &mut BTreeSet<String>,
    deps: &mut Vec<String>,
) -> anyhow::Result<()> {
    if !visited.insert(path.to_string_lossy().to_string()) {
        return Ok(());
    }
    for dep in extract_matches(path, r"use 0x0::([a-zA-Z0-9_]+);")? {
        if let Some(dep_path) = file_map.get(&dep) {
            let dep_str = dep_path.to_string_lossy().to_string();
            if !deps.contains(&dep_str) {
                deps.push(dep_str);
                calculate_transitive_deps(dep_path.as_path(), file_map, visited, deps)?;
            }
        } else {
            return Err(anyhow!(
                "cannot find source for module `{}` (file map: {:?})",
                dep,
                file_map,
            ));
        }
    }
    Ok(())
}

fn calculate_file_map(search_path: &[&str]) -> anyhow::Result<BTreeMap<String, PathBuf>> {
    let mut module_to_file = BTreeMap::new();
    // Walk all move sources in search path to determine which modules they define.
    for dir_str in search_path {
        let dir = Path::new(dir_str);
        for entry in dir.read_dir()? {
            let cand = entry?.path();
            if !cand.to_string_lossy().ends_with(".move") {
                continue;
            }
            for module in extract_matches(cand.as_path(), r"module ([a-zA-Z0-9_]+)")? {
                module_to_file.insert(module, cand.clone());
            }
        }
    }
    Ok(module_to_file)
}

datatest_stable::harness!(test_runner, "tests/sources", r".*\.move");
