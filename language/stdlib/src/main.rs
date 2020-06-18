// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use clap::{App, Arg};
use rayon::prelude::*;
use std::{
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};
use stdlib::{
    build_stdlib, build_stdlib_doc, build_transaction_script_abi, build_transaction_script_doc,
    compile_script, filter_move_files, save_binary, COMPILED_EXTENSION, COMPILED_OUTPUT_PATH,
    COMPILED_STDLIB_NAME, TRANSACTION_SCRIPTS,
};

// Generates the compiled stdlib and transaction scripts. Until this is run changes to the source
// modules/scripts, and changes in the Move compiler will not be reflected in the stdlib used for
// genesis, and everywhere else across the code-base unless otherwise specified.
fn main() {
    let cli = App::new("stdlib")
        .name("Move standard library")
        .author("The Libra Core Contributors")
        .arg(
            Arg::with_name("no-doc")
                .long("no-doc")
                .help("do not generate documentation"),
        )
        .arg(
            Arg::with_name("no-script-abi")
                .long("no-script-abi")
                .help("do not generate script ABIs"),
        )
        .arg(
            Arg::with_name("no-compiler")
                .long("no-compiler")
                .help("do not compile modules and scripts"),
        );
    let matches = cli.get_matches();
    let no_doc = matches.is_present("no-doc");
    let no_script_abi = matches.is_present("no-script-abi");
    let no_compiler = matches.is_present("no-compiler");

    #[cfg(debug_assertions)]
    {
        println!("NOTE: run this program in --release mode for better speed");
    }

    let mut scripts_path = PathBuf::from(COMPILED_OUTPUT_PATH);
    scripts_path.push(TRANSACTION_SCRIPTS);

    std::fs::create_dir_all(&scripts_path).unwrap();

    if !no_compiler {
        time_it("Creating stdlib blob", || {
            let mut module_path = PathBuf::from(COMPILED_OUTPUT_PATH);
            module_path.push(COMPILED_STDLIB_NAME);
            module_path.set_extension(COMPILED_EXTENSION);
            let modules: Vec<Vec<u8>> = build_stdlib()
                .into_iter()
                .map(|verified_module| {
                    let mut ser = Vec::new();
                    verified_module.into_inner().serialize(&mut ser).unwrap();
                    ser
                })
                .collect();
            let bytes = lcs::to_bytes(&modules).unwrap();
            if save_binary(&module_path, &bytes) {
                println!("Compiled module binary has changed");
            };
        });
    }

    let txn_source_files =
        datatest_stable::utils::iterate_directory(Path::new(TRANSACTION_SCRIPTS));
    let transaction_files = filter_move_files(txn_source_files)
        .flat_map(|path| path.into_os_string().into_string().ok())
        .collect::<Vec<_>>();
    if !no_compiler {
        time_it("Staging transaction scripts", || {
            transaction_files.par_iter().for_each(|txn_file| {
                let compiled_script = compile_script(txn_file.clone());
                let mut txn_path = PathBuf::from(COMPILED_OUTPUT_PATH);
                txn_path.push(txn_file.clone());
                txn_path.set_extension(COMPILED_EXTENSION);
                if save_binary(&txn_path, &compiled_script) {
                    println!("Compiled script binary {} has changed", txn_file);
                }
            })
        });
    }

    // Generate documentation
    if !no_doc {
        time_it("Generating stdlib documentation", || {
            build_stdlib_doc();
        });
        time_it("Generating script documentation", || {
            transaction_files
                .par_iter()
                .for_each(|txn_file| build_transaction_script_doc(txn_file.clone()));
        });
    }

    // Generate script ABIs
    if !no_script_abi {
        time_it("Generating script ABIs", || {
            transaction_files
                .par_iter()
                .for_each(|txn_file| build_transaction_script_abi(txn_file.clone()));
        });
    }
}

fn time_it<F>(msg: &str, f: F)
where
    F: Fn(),
{
    let now = Instant::now();
    print!("{} ... ", msg);
    let _ = std::io::stdout().flush();
    f();
    println!("(took {:.3}s)", now.elapsed().as_secs_f64());
}
