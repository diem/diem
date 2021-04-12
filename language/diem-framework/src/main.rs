// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use clap::{App, Arg};
use diem_framework::*;
use std::path::Path;

// Generates the compiled stdlib and transaction scripts. Until this is run changes to the source
// modules/scripts, and changes in the Move compiler will not be reflected in the stdlib used for
// genesis, and everywhere else across the code-base unless otherwise specified.
fn main() {
    let cli = App::new("stdlib")
        .name("Move standard library")
        .author("The Diem Core Contributors")
        .arg(Arg::with_name("output").long("output").help("use a custom output path").takes_value(true))
        .arg(
            Arg::with_name("no-doc")
                .long("no-doc")
                .help("do not generate documentation"),
        )
        .arg(
            Arg::with_name("no-script-abi")
                .long("no-script-abi")
                .requires("no-compiler")
                .help("do not generate script ABIs"),
        )
        .arg(
            Arg::with_name("no-script-builder")
                .long("no-script-builder")
                .help("do not generate script builders"),
        )
        .arg(
            Arg::with_name("no-compiler")
                .long("no-compiler")
                .help("do not compile modules and scripts"),
        )
        .arg(
            Arg::with_name("no-check-linking-layout-compatibility")
                .long("no-check-linking-layout-compatibility")
                .help("do not print information about linking and layout compatibility between the old and new standard library"),
        )
        .arg(Arg::with_name("no-errmap").long("no-errmap").help("do not generate error explanations"))
        .arg(
            Arg::with_name("with-diagram")
                .long("with-diagram")
                .help("include diagrams in the stdlib documentation")
        );
    let matches = cli.get_matches();
    let options = release::ReleaseOptions {
        build_modules: !matches.is_present("no-compiler"),
        check_layout_compatibility: !matches.is_present("no-check-linking-layout-compatibility"),
        module_docs: !matches.is_present("no-doc"),
        script_docs: !matches.is_present("no-doc"),
        with_diagram: matches.is_present("with-diagram"),
        script_abis: !matches.is_present("no-script-abi"),
        script_builder: !matches.is_present("no-script-builder"),
        errmap: !matches.is_present("no-errmap"),
        time_it: true,
    };

    // Make sure that the current directory is `language/diem-framework` from now on.
    let exec_path = std::env::args().next().expect("path of the executable");
    let base_path = std::path::Path::new(&exec_path)
        .parent()
        .unwrap()
        .join("../../language/diem-framework");
    std::env::set_current_dir(&base_path).expect("failed to change directory");

    #[cfg(debug_assertions)]
    {
        println!("NOTE: run this program in --release mode for better speed");
    }

    let output_path = matches
        .value_of("output")
        .unwrap_or("releases/artifacts/current");

    release::create_release(&Path::new(output_path), &options);

    // Sync the generated docs for the modules and docs to their old locations to maintain
    // documentation locations.
    if matches.value_of("output").is_none() {
        release::sync_doc_files(&output_path);
    }
}
