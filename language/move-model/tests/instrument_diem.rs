// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use std::{fs, path::PathBuf, process};

use move_model::run_spec_instrumenter;

#[test]
#[ignore]
fn run_on_diem() {
    // TODO (mengxu) re-enable this test once the instrumentation is complete
    let targets = vec![
        vec![env!("CARGO_MANIFEST_DIR"), "..", "move-stdlib", "modules"]
            .into_iter()
            .collect::<PathBuf>()
            .into_os_string()
            .into_string()
            .unwrap(),
        vec![
            env!("CARGO_MANIFEST_DIR"),
            "..",
            "diem-framework",
            "modules",
        ]
        .into_iter()
        .collect::<PathBuf>()
        .into_os_string()
        .into_string()
        .unwrap(),
    ];
    let workdir = vec![env!("CARGO_MANIFEST_DIR"), "tests", "tmp-diem"]
        .into_iter()
        .collect::<PathBuf>();
    fs::create_dir_all(&workdir).unwrap();
    let result = run_spec_instrumenter(
        targets,
        vec![],
        workdir.as_os_str().to_str().unwrap(),
        false,
    );
    fs::remove_dir_all(&workdir).unwrap();

    let env =
        result.expect("Failed to run the spec instrumenter on move-stdlib and diem-framework");
    if env.has_errors() {
        let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
        env.report_errors(&mut error_writer);
        process::abort();
    }
}
