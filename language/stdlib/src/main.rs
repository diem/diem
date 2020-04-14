// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use stdlib::{
    build_stdlib, compile_script, filter_move_files, stdlib_files, STAGED_EXTENSION,
    STAGED_OUTPUT_PATH, STAGED_STDLIB_NAME, TRANSACTION_SCRIPTS,
};

// Generates the staged stdlib and transaction scripts. Until this is run, changes to the source
// modules/scripts and changes in the Move compiler will not be reflected in the stdlib used for
// genesis, and everywhere else across the code-base unless otherwise specified.
fn main() {
    let mut scripts_path = PathBuf::from(STAGED_OUTPUT_PATH);
    scripts_path.push(TRANSACTION_SCRIPTS);

    std::fs::create_dir_all(&scripts_path).unwrap();

    // Save copies of the module source files
    let mut source_dir = PathBuf::from(STAGED_OUTPUT_PATH);
    source_dir.push("src");
    std::fs::create_dir_all(&source_dir).unwrap();
    for file in &stdlib_files() {
        let orig_file = PathBuf::from(file);
        let copied_file = source_dir.join(orig_file.file_name().unwrap());
        std::fs::copy(orig_file, copied_file).unwrap();
    }

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
