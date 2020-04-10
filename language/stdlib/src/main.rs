// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytecode_verifier::{batch_verify_modules, VerifiedModule};
use move_lang::{compiled_unit::CompiledUnit, move_compile, shared::Address};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

/// Directory containing the Move transaction scripts
const TRANSACTION_SCRIPTS: &str = "transaction_scripts";
/// Directory containing the Move modules
const STD_LIB_DIR: &str = "modules";
/// The output path where the staged build will be placed
const STAGED_OUTPUT_PATH: &str = "staged";
/// The file name for the staged stdlib
const STAGED_STDLIB_NAME: &str = "stdlib";
/// The extension for staged files (Move bytecode)
const STAGED_EXTENSION: &str = "mv";
/// The extension for Move source files
const MOVE_EXTENSION: &str = "move";

fn filter_move_files(dir_iter: impl Iterator<Item = PathBuf>) -> impl Iterator<Item = PathBuf> {
    dir_iter.flat_map(|path| {
        if path.extension()?.to_str()? == MOVE_EXTENSION {
            Some(path)
        } else {
            None
        }
    })
}

pub fn stdlib_files() -> Vec<String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(STD_LIB_DIR);
    let dirfiles = datatest_stable::utils::iterate_directory(&path);
    filter_move_files(dirfiles)
        .flat_map(|path| path.into_os_string().into_string())
        .collect()
}

fn compile_script(source_file_str: String) -> Vec<u8> {
    let (_, mut compiled_program) = move_compile(
        &[source_file_str],
        &stdlib_files(),
        Some(Address::LIBRA_CORE),
    )
    .unwrap();
    let mut script_bytes = vec![];
    assert!(compiled_program.len() == 1);
    match compiled_program.pop().unwrap() {
        CompiledUnit::Module { .. } => panic!("Unexpected module when compiling script"),
        CompiledUnit::Script { script, .. } => script.serialize(&mut script_bytes).unwrap(),
    };
    script_bytes
}

fn build_stdlib() -> Vec<VerifiedModule> {
    let (_, compiled_units) =
        move_compile(&stdlib_files(), &[], Some(Address::LIBRA_CORE)).unwrap();
    batch_verify_modules(
        compiled_units
            .into_iter()
            .map(|compiled_unit| match compiled_unit {
                CompiledUnit::Module { module, .. } => module,
                CompiledUnit::Script { .. } => panic!("Unexpected Script in stdlib"),
            })
            .collect(),
    )
}

// modules/scripts, and changes in the Move compiler will not be reflected in the stdlib used for
// genesis, and everywhere else across the code-base unless otherwise specified.
fn main() {
    let mut scripts_path = PathBuf::from(STAGED_OUTPUT_PATH);
    scripts_path.push(TRANSACTION_SCRIPTS);

    std::fs::create_dir_all(&scripts_path).unwrap();

    // Write the stdlib blob
    let mut module_path = PathBuf::from(STAGED_OUTPUT_PATH);
    module_path.push(STAGED_STDLIB_NAME);
    module_path.set_extension(STAGED_EXTENSION);
    let modules: Vec<Vec<u8>> = build_stdlib()
        .into_iter()
        .map(|verified_module| {
            let mut ser = Vec::new();
            verified_module.into_inner().serialize(&mut ser).unwrap();
            ser
        })
        .collect();
    let bytes = lcs::to_bytes(&modules).unwrap();
    let mut module_file = File::create(module_path).unwrap();
    module_file.write_all(&bytes).unwrap();

    let txn_source_files =
        datatest_stable::utils::iterate_directory(Path::new(TRANSACTION_SCRIPTS));
    let transaction_files = filter_move_files(txn_source_files)
        .flat_map(|path| path.into_os_string().into_string().ok());
    for txn_file in transaction_files {
        let compiled_script = compile_script(txn_file.clone());
        let mut txn_path = PathBuf::from(STAGED_OUTPUT_PATH);
        txn_path.push(txn_file.clone());
        txn_path.set_extension(STAGED_EXTENSION);
        File::create(txn_path)
            .unwrap()
            .write_all(&compiled_script)
            .unwrap();
    }
}
