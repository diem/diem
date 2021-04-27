// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{ffi::OsStr, path::Path};

use codespan_reporting::term::termcolor::Buffer;
use move_prover::{cli::Options, run_move_prover};
use move_prover_test_utils::baseline_test::verify_or_update_baseline;
use std::path::PathBuf;
use tempfile::TempDir;

#[allow(unused_imports)]
use log::debug;
use std::{
    fs::{self, File},
    io::Read,
};

const FLAGS: &[&str] = &["--verbose=warn", "--abigen"];

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let mut args = vec!["mvp_test".to_string()];
    args.extend(FLAGS.iter().map(|s| (*s).to_string()));
    args.push(path.to_string_lossy().to_string());

    let mut options = Options::create_from_args(&args)?;
    options.setup_logging_for_test();
    options.abigen.compiled_script_directory = "tests/sources".to_string();

    test_abigen(path, options, "abi")?;

    Ok(())
}

fn get_generated_abis(dir: &Path) -> std::io::Result<Vec<String>> {
    let mut abi_paths = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                abi_paths.append(&mut get_generated_abis(&path)?);
            } else if let Some("abi") = path.extension().and_then(OsStr::to_str) {
                abi_paths.push(path.to_str().unwrap().to_string());
            }
        }
    }
    Ok(abi_paths)
}

fn test_abigen(path: &Path, mut options: Options, suffix: &str) -> anyhow::Result<()> {
    let temp_path = PathBuf::from(TempDir::new()?.path());
    options.abigen.output_directory = temp_path.to_string_lossy().to_string();

    let mut error_writer = Buffer::no_color();
    match run_move_prover(&mut error_writer, options) {
        Ok(()) => {
            for abi_path in get_generated_abis(&temp_path)?.iter() {
                let mut contents = String::new();
                if let Ok(mut file) = File::open(abi_path) {
                    file.read_to_string(&mut contents).unwrap();
                }
                let buf = PathBuf::from(abi_path);
                let mut baseline_file_name = PathBuf::from(path);
                baseline_file_name.pop();
                baseline_file_name.push(buf.strip_prefix(&temp_path)?);
                verify_or_update_baseline(&baseline_file_name, &contents)?;
            }
        }
        Err(err) => {
            let mut contents = format!("Move prover abigen returns: {}\n", err);
            contents += &String::from_utf8_lossy(&error_writer.into_inner()).to_string();
            let baseline_path = path.with_extension(suffix);
            verify_or_update_baseline(&baseline_path, &contents)?;
        }
    };
    Ok(())
}

datatest_stable::harness!(test_runner, "tests/sources", r".*\.move",);
