// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use codespan_reporting::term::termcolor::Buffer;

use move_prover::{cli::Options, run_move_prover};
use move_prover_test_utils::baseline_test::verify_or_update_baseline;
use std::path::PathBuf;
use tempfile::TempDir;

use itertools::Itertools;
#[allow(unused_imports)]
use log::debug;
use std::{fs::File, io::Read};

const FLAGS: &[&str] = &[
    "--verbose=warn",
    "--dependency=../../move-stdlib/modules",
    "--dependency=../../diem-framework/modules",
    "--docgen",
    "-a=Std=0x1",
    "-a=DiemFramework=0x1",
    "-a=DiemRoot=0xA550C18",
    "-a=CurrencyInfo=0xA550C18",
    "-a=TreasuryCompliance=0xB1E55ED",
    "-a=VMReserved=0x0",
];

fn test_runner(path: &Path) -> datatest_stable::Result<()> {
    let mut args = vec!["mvp_test".to_string()];
    args.extend(FLAGS.iter().map(|s| (*s).to_string()).collect_vec());

    let is_root_template = path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string()
        .ends_with("_template.md");
    if !is_root_template {
        args.push(path.to_string_lossy().to_string());
    } else {
        // add as sources all .notest_move files starting with the name of the template.
        let base_name = path.file_stem().unwrap().to_string_lossy().to_string();
        let dir_name = path.parent().unwrap();
        dir_name.read_dir().unwrap().for_each(|res| {
            let f = res.unwrap();
            let name = f.file_name().to_string_lossy().to_string();
            if name.starts_with(&base_name) && name.ends_with(".notest_move") {
                args.push(f.path().to_string_lossy().to_string());
            }
        });
    }

    let mut options = Options::create_from_args(&args)?;
    options.setup_logging_for_test();

    if is_root_template {
        options.docgen.root_doc_templates = vec![path.to_string_lossy().to_string()];
    }

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
    let mut temp_path = PathBuf::from(TempDir::new()?.path());
    options.docgen.output_directory = temp_path.to_string_lossy().to_string();
    let base_name = format!(
        "{}.md",
        path.file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .replace("_template", "")
    );
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
    output += &String::from_utf8_lossy(&error_writer.into_inner());
    let baseline_path = path.with_extension(suffix);
    verify_or_update_baseline(baseline_path.as_path(), &output)?;
    Ok(())
}

datatest_stable::harness!(test_runner, "tests/sources", r".*\.move|.*_template\.md",);
