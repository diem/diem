// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use codespan_reporting::term::termcolor::Buffer;

use libra_temppath::TempPath;
use move_prover::{cli::Options, run_move_prover};
use std::path::PathBuf;
use test_utils::baseline_test::verify_or_update_baseline;

use itertools::Itertools;
#[allow(unused_imports)]
use log::debug;
use std::{fs::File, io::Read};

const FLAGS: &[&str] = &[
    "--verbose=warn",
    // This currently points to the legacy stdlib copy in prover tests. Replace this
    // by real stdlib once we have a good example based on it.
    "--dependency=../tests/sources/stdlib/modules",
    "--docgen",
];

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let mut args = vec!["mvp_test".to_string()];
    args.extend(FLAGS.iter().map(|s| (*s).to_string()).collect_vec());
    args.push(path.to_string_lossy().to_string());

    let mut options = Options::create_from_args(&args)?;
    options.setup_logging_for_test();

    options.docgen.include_specs = true;
    options.docgen.include_impl = true;
    options.docgen.include_private_fun = true;

    options.docgen.specs_inlined = true;
    test_docgen(path, options.clone(), "spec_inline.md")?;

    options.docgen.specs_inlined = false;
    test_docgen(path, options.clone(), "spec_separate.md")?;

    options.docgen.specs_inlined = true;
    options.docgen.collapsed_sections = false;
    test_docgen(path, options, "spec_inline_no_fold.md")?;

    Ok(())
}

fn test_docgen(path: &Path, mut options: Options, suffix: &str) -> anyhow::Result<()> {
    let mut temp_path = PathBuf::from(TempPath::new().path());
    options.docgen.output_directory = temp_path.to_string_lossy().to_string();
    let base_name = format!("{}.md", path.file_stem().unwrap().to_str().unwrap());
    temp_path.push(&base_name);

    let mut error_writer = Buffer::no_color();
    let mut output = match run_move_prover(&mut error_writer, options) {
        Ok(()) => {
            let mut contents = String::new();
            debug!("writing to {}", temp_path.display());
            let mut file = File::open(temp_path.as_path()).unwrap();
            file.read_to_string(&mut contents).unwrap();
            contents
        }
        Err(err) => format!("Move prover docgen returns: {}\n", err),
    };
    output += &String::from_utf8_lossy(&error_writer.into_inner()).to_string();
    let baseline_path = path.with_extension(suffix);
    verify_or_update_baseline(baseline_path.as_path(), &output)?;
    Ok(())
}

datatest_stable::harness!(test_runner, "tests/sources", r".*\.move",);
