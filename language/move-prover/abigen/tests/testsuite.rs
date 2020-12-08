// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use codespan_reporting::term::termcolor::Buffer;
use diem_temppath::TempPath;
use move_prover::{cli::Options, run_move_prover};
use move_prover_test_utils::baseline_test::verify_or_update_baseline;
use std::path::PathBuf;

#[allow(unused_imports)]
use log::debug;
use std::{fs::File, io::Read};

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

fn test_abigen(path: &Path, mut options: Options, suffix: &str) -> anyhow::Result<()> {
    let mut temp_path = PathBuf::from(TempPath::new().path());
    options.abigen.output_directory = temp_path.to_string_lossy().to_string();
    let base_name = format!("{}.abi", path.file_stem().unwrap().to_str().unwrap());
    temp_path.push(&base_name);

    let mut error_writer = Buffer::no_color();
    let mut output = match run_move_prover(&mut error_writer, options) {
        Ok(()) => {
            let mut contents = String::new();
            debug!("writing to {}", temp_path.display());
            if let Ok(mut file) = File::open(temp_path.as_path()) {
                file.read_to_string(&mut contents).unwrap();
            }
            contents
        }
        Err(err) => format!("Move prover abigen returns: {}\n", err),
    };
    output += &String::from_utf8_lossy(&error_writer.into_inner()).to_string();
    let baseline_path = path.with_extension(suffix);
    verify_or_update_baseline(baseline_path.as_path(), &output)?;
    Ok(())
}

datatest_stable::harness!(test_runner, "tests/sources", r".*\.move",);
