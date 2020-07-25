// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytecode_verifier::{verify_module, DependencyChecker};
use log::LevelFilter;
use move_lang::{compiled_unit::CompiledUnit, move_compile, shared::Address};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use vm::CompiledModule;

pub const STD_LIB_DIR: &str = "modules";
pub const MOVE_EXTENSION: &str = "move";

pub const TRANSACTION_SCRIPTS: &str = "transaction_scripts";
/// The output path under which compiled files will be put
pub const COMPILED_OUTPUT_PATH: &str = "compiled";
/// The file name for the compiled stdlib
pub const COMPILED_STDLIB_DIR: &str = "stdlib";
/// The extension for compiled files
pub const COMPILED_EXTENSION: &str = "mv";
/// The file name of the debug module
pub const DEBUG_MODULE_FILE_NAME: &str = "debug.move";

/// The output path for stdlib documentation.
pub const STD_LIB_DOC_DIR: &str = "modules/doc";
/// The output path for transaction script documentation.
pub const TRANSACTION_SCRIPTS_DOC_DIR: &str = "transaction_scripts/doc";

/// The output path under which compiled script files can be found
pub const COMPILED_TRANSACTION_SCRIPTS_DIR: &str = "compiled/transaction_scripts";
/// The output path for transaction script ABIs.
pub const COMPILED_TRANSACTION_SCRIPTS_ABI_DIR: &str = "compiled/transaction_scripts/abi";

/// Where to write generated transaction builders.
pub const TRANSACTION_BUILDERS_GENERATED_SOURCE_PATH: &str =
    "../transaction-builder-generated/src/stdlib.rs";

pub fn filter_move_files(dir_iter: impl Iterator<Item = PathBuf>) -> impl Iterator<Item = PathBuf> {
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

pub fn build_stdlib() -> BTreeMap<String, CompiledModule> {
    let (_, compiled_units) =
        move_compile(&stdlib_files(), &[], Some(Address::LIBRA_CORE)).unwrap();
    let mut modules = BTreeMap::new();
    for (i, compiled_unit) in compiled_units.into_iter().enumerate() {
        let name = compiled_unit.name();
        match compiled_unit {
            CompiledUnit::Module { module, .. } => {
                verify_module(&module).expect("stdlib module failed to verify");
                DependencyChecker::verify_module(&module, modules.values())
                    .expect("stdlib module dependency failed to verify");
                // Tag each module with its index in the module dependency order. Needed for
                // when they are deserialized and verified later on.
                modules.insert(format!("{}_{}", i, name), module);
            }
            CompiledUnit::Script { .. } => panic!("Unexpected Script in stdlib"),
        }
    }
    modules
}

pub fn compile_script(source_file_str: String) -> Vec<u8> {
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

pub fn save_binary(path: &Path, binary: &[u8]) -> bool {
    if path.exists() {
        let mut bytes = vec![];
        File::open(path).unwrap().read_to_end(&mut bytes).unwrap();
        if Sha256::digest(binary) == Sha256::digest(&bytes) {
            return false;
        }
    }
    File::create(path).unwrap().write_all(binary).unwrap();
    true
}

pub fn build_stdlib_doc() {
    build_doc(STD_LIB_DOC_DIR, "", stdlib_files().as_slice(), "")
}

pub fn build_transaction_script_doc(script_file_str: String) {
    build_doc(
        TRANSACTION_SCRIPTS_DOC_DIR,
        STD_LIB_DOC_DIR,
        &[script_file_str],
        STD_LIB_DIR,
    )
}

pub fn build_transaction_script_abi(script_file_str: String) {
    build_abi(
        COMPILED_TRANSACTION_SCRIPTS_ABI_DIR,
        &[script_file_str],
        STD_LIB_DIR,
        COMPILED_TRANSACTION_SCRIPTS_DIR,
    )
}

fn build_doc(output_path: &str, doc_path: &str, sources: &[String], dep_path: &str) {
    let mut options = move_prover::cli::Options::default();
    options.move_sources = sources.to_vec();
    if !dep_path.is_empty() {
        options.move_deps = vec![dep_path.to_string()]
    }
    options.verbosity_level = LevelFilter::Warn;
    options.run_docgen = true;
    options.docgen.include_impl = true;
    options.docgen.include_private_fun = true;
    options.docgen.specs_inlined = false;
    if !doc_path.is_empty() {
        options.docgen.doc_path = vec![doc_path.to_string()];
    }
    options.docgen.output_directory = output_path.to_string();
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

fn build_abi(output_path: &str, sources: &[String], dep_path: &str, compiled_script_path: &str) {
    let mut options = move_prover::cli::Options::default();
    options.move_sources = sources.to_vec();
    if !dep_path.is_empty() {
        options.move_deps = vec![dep_path.to_string()]
    }
    options.verbosity_level = LevelFilter::Warn;
    options.run_abigen = true;
    options.abigen.output_directory = output_path.to_string();
    options.abigen.compiled_script_directory = compiled_script_path.to_string();
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

pub fn generate_rust_transaction_builders() {
    let abis = transaction_builder_generator::read_abis(COMPILED_TRANSACTION_SCRIPTS_ABI_DIR)
        .expect("Failed to read generated ABIs");
    let mut f = std::fs::File::create(TRANSACTION_BUILDERS_GENERATED_SOURCE_PATH)
        .expect("Failed to open file for Rust script build generation");
    transaction_builder_generator::rust::output(&mut f, &abis, /* local types */ true)
        .expect("Failed to generate Rust builders for Libra");

    std::process::Command::new("rustfmt")
        .arg("--config")
        .arg("merge_imports=true")
        .arg(TRANSACTION_BUILDERS_GENERATED_SOURCE_PATH)
        .status()
        .expect("Failed to run rustfmt on generated code");
}
