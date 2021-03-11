// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytecode_verifier::{cyclic_dependencies, dependencies, verify_module};
use errmapgen::ErrmapOptions;
use log::LevelFilter;
use move_lang::{compiled_unit::CompiledUnit, move_compile_and_report, shared::Address};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use vm::{access::ModuleAccess, file_format::CompiledModule};

pub use move_stdlib::{COMPILED_EXTENSION, ERROR_DESC_EXTENSION, MOVE_EXTENSION};

const MODULES_DIR: &str = "modules";

/// The extension for compiled files

pub const TRANSACTION_SCRIPTS: &str = "transaction_scripts";
/// The output path under which compiled files will be put
pub const COMPILED_OUTPUT_PATH: &str = "compiled";
/// The file name for the compiled stdlib
pub const COMPILED_STDLIB_DIR: &str = "stdlib";
/// The file name of the debug module
pub const DEBUG_MODULE_FILE_NAME: &str = "debug.move";

/// The output path for stdlib documentation.
pub const STD_LIB_DOC_DIR: &str = "modules/doc";
/// The output path for transaction script documentation.
pub const TRANSACTION_SCRIPTS_DOC_DIR: &str = "transaction_scripts/doc";
/// The documentation root template for stdlib.
pub const STD_LIB_DOC_TEMPLATE: &str = "modules/overview_template.md";
/// The documentation root template for scripts.
pub const TRANSACTION_SCRIPT_DOC_TEMPLATE: &str =
    "transaction_scripts/transaction_script_documentation_template.md";
/// The specification root template for scripts and stdlib.
pub const SPEC_DOC_TEMPLATE: &str = "transaction_scripts/spec_documentation_template.md";
/// Path to the references template.
pub const REFERENCES_DOC_TEMPLATE: &str = "modules/references_template.md";

pub const ERROR_DESC_DIR: &str = "error_descriptions";
pub const ERROR_DESC_FILENAME: &str = "error_descriptions";

pub const PACKED_TYPES_DIR: &str = "packed_types";
pub const PACKED_TYPES_FILENAME: &str = "packed_types";
pub const PACKED_TYPES_EXTENSION: &str = "txt";

/// The output path under which compiled script files can be found
pub const LEGACY_COMPILED_TRANSACTION_SCRIPTS_DIR: &str = "compiled/legacy/transaction_scripts";
/// The output path for transaction script ABIs.
pub const COMPILED_TRANSACTION_SCRIPTS_ABI_DIR: &str = "compiled/transaction_scripts/abi";
/// Location of legacy transaction script ABIs
pub const LEGACY_COMPILED_TRANSACTION_SCRIPTS_ABI_DIR: &str =
    "compiled/legacy/transaction_scripts/abi";
/// Where to write generated transaction builders.
pub const TRANSACTION_BUILDERS_GENERATED_SOURCE_PATH: &str =
    "compiled/src/shim/tmp_new_transaction_script_builders.rs";

pub fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: Into<String>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative.into());
    path
}

pub fn diem_stdlib_modules_full_path() -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), MODULES_DIR)
}

pub fn diem_stdlib_files_no_dependencies() -> Vec<String> {
    let path = path_in_crate(MODULES_DIR);
    let dirfiles = move_stdlib::utils::iterate_directory(&path);
    move_stdlib::filter_move_files(dirfiles)
        .flat_map(|path| path.into_os_string().into_string())
        .collect()
}

pub fn diem_stdlib_files() -> Vec<String> {
    let mut files = move_stdlib::move_stdlib_files();
    files.extend(diem_stdlib_files_no_dependencies());
    files
}

pub fn stdlib_bytecode_files() -> Vec<String> {
    let path = path_in_crate(COMPILED_OUTPUT_PATH);
    let names = diem_stdlib_files();
    let dirfiles = move_stdlib::utils::iterate_directory(&path);
    let res: Vec<String> = move_stdlib::filter_move_bytecode_files(dirfiles)
        .filter(|path| {
            for name in &names {
                let suffix = "_".to_owned()
                    + Path::new(name)
                        .with_extension(COMPILED_EXTENSION)
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap();
                if path
                    .file_name()
                    .map(|f| f.to_str())
                    .flatten()
                    .map_or(false, |s| s.ends_with(&suffix))
                {
                    return true;
                }
            }
            false
        })
        .map(|path| path.into_os_string().into_string().unwrap())
        .collect();
    assert!(
        !res.is_empty(),
        "Unexpected: no stdlib bytecode files found"
    );
    res
}

pub fn script_files() -> Vec<String> {
    let path = path_in_crate(TRANSACTION_SCRIPTS);
    let dirfiles = move_stdlib::utils::iterate_directory(&path);
    move_stdlib::filter_move_files(dirfiles)
        .flat_map(|path| path.into_os_string().into_string())
        .collect()
}

pub fn build_stdlib() -> BTreeMap<String, CompiledModule> {
    let (_files, compiled_units) = move_compile_and_report(
        &diem_stdlib_files(),
        &[],
        Some(Address::DIEM_CORE),
        None,
        false,
    )
    .unwrap();
    let mut modules = BTreeMap::new();
    for (i, compiled_unit) in compiled_units.into_iter().enumerate() {
        let name = compiled_unit.name();
        match compiled_unit {
            CompiledUnit::Module { module, .. } => {
                verify_module(&module).expect("stdlib module failed to verify");
                dependencies::verify_module(&module, modules.values())
                    .expect("stdlib module dependency failed to verify");
                // Tag each module with its index in the module dependency order. Needed for
                // when they are deserialized and verified later on.
                modules.insert(format!("{:0>3}_{}", i, name), module);
            }
            CompiledUnit::Script { .. } => panic!("Unexpected Script in stdlib"),
        }
    }
    let modules_by_id: BTreeMap<_, _> = modules
        .values()
        .map(|module| (module.self_id(), module))
        .collect();
    for module in modules_by_id.values() {
        cyclic_dependencies::verify_module(
            module,
            |module_id| {
                Ok(modules_by_id
                    .get(module_id)
                    .expect("missing module in stdlib")
                    .immediate_dependencies())
            },
            |module_id| {
                Ok(modules_by_id
                    .get(module_id)
                    .expect("missing module in stdlib")
                    .immediate_friends())
            },
        )
        .expect("stdlib module has cyclic dependencies");
    }
    modules
}

pub fn compile_script(source_file_str: String) -> Vec<u8> {
    let (_files, mut compiled_program) = move_compile_and_report(
        &[source_file_str],
        &diem_stdlib_files(),
        Some(Address::DIEM_CORE),
        None,
        false,
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

pub fn build_stdlib_doc(with_diagram: bool) {
    move_stdlib::build_doc(
        STD_LIB_DOC_DIR,
        // FIXME: use absolute path when the bug in docgen is fixed.
        // &move_stdlib::move_stdlib_docs_full_path(),
        "../move-stdlib/docs",
        vec![path_in_crate(STD_LIB_DOC_TEMPLATE)
            .to_string_lossy()
            .to_string()],
        Some(
            path_in_crate(REFERENCES_DOC_TEMPLATE)
                .to_string_lossy()
                .to_string(),
        ),
        diem_stdlib_files_no_dependencies().as_slice(),
        vec![move_stdlib::move_stdlib_modules_full_path()],
        with_diagram,
    )
}

pub fn build_transaction_script_doc(script_files: &[String], with_diagram: bool) {
    move_stdlib::build_doc(
        TRANSACTION_SCRIPTS_DOC_DIR,
        // FIXME: links to move stdlib modules are broken since the tool does not currently
        // support multiple paths.
        STD_LIB_DOC_DIR,
        vec![
            path_in_crate(TRANSACTION_SCRIPT_DOC_TEMPLATE)
                .to_string_lossy()
                .to_string(),
            path_in_crate(SPEC_DOC_TEMPLATE)
                .to_string_lossy()
                .to_string(),
        ],
        Some(
            path_in_crate(REFERENCES_DOC_TEMPLATE)
                .to_string_lossy()
                .to_string(),
        ),
        script_files,
        vec![
            move_stdlib::move_stdlib_modules_full_path(),
            diem_stdlib_modules_full_path(),
        ],
        with_diagram,
    )
}

pub fn build_script_abis(script_file_str: String) {
    build_abi(
        COMPILED_TRANSACTION_SCRIPTS_ABI_DIR,
        &[script_file_str],
        vec![
            move_stdlib::move_stdlib_modules_full_path(),
            diem_stdlib_modules_full_path(),
        ],
        // The only code that we should be using for transaction scripts is the legacy bytes
        LEGACY_COMPILED_TRANSACTION_SCRIPTS_DIR,
    )
}

pub fn build_stdlib_error_code_map() {
    let mut path = PathBuf::from(COMPILED_OUTPUT_PATH);
    path.push(ERROR_DESC_DIR);
    fs::create_dir_all(&path).unwrap();
    path.push(ERROR_DESC_FILENAME);
    path.set_extension(ERROR_DESC_EXTENSION);
    build_error_code_map(path.to_str().unwrap(), diem_stdlib_files().as_slice(), "")
}

fn build_abi(
    output_path: &str,
    sources: &[String],
    dep_paths: Vec<String>,
    compiled_script_path: &str,
) {
    let options = move_prover::cli::Options {
        move_sources: sources.to_vec(),
        move_deps: dep_paths,
        verbosity_level: LevelFilter::Warn,
        run_abigen: true,
        abigen: abigen::AbigenOptions {
            output_directory: output_path.to_string(),
            compiled_script_directory: compiled_script_path.to_string(),
        },
        ..Default::default()
    };
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

pub fn get_packed_types_path() -> PathBuf {
    let mut path = PathBuf::from(COMPILED_OUTPUT_PATH);
    path.push(PACKED_TYPES_DIR);
    path.push(PACKED_TYPES_FILENAME);
    path.set_extension(PACKED_TYPES_EXTENSION);
    path
}

fn build_error_code_map(output_path: &str, sources: &[String], dep_path: &str) {
    let options = move_prover::cli::Options {
        move_sources: sources.to_vec(),
        move_deps: if !dep_path.is_empty() {
            vec![dep_path.to_string()]
        } else {
            vec![]
        },
        verbosity_level: LevelFilter::Warn,
        run_errmapgen: true,
        errmapgen: ErrmapOptions {
            output_file: output_path.to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

pub fn generate_rust_transaction_builders() {
    let mut abis =
        transaction_builder_generator::read_abis(&Path::new(COMPILED_TRANSACTION_SCRIPTS_ABI_DIR))
            .expect("Failed to read generated ABIs");
    abis.extend(
        transaction_builder_generator::read_abis(&Path::new(
            LEGACY_COMPILED_TRANSACTION_SCRIPTS_ABI_DIR,
        ))
        .expect("Failed to read legacy ABIs")
        .into_iter(),
    );
    {
        let mut file = std::fs::File::create(TRANSACTION_BUILDERS_GENERATED_SOURCE_PATH)
            .expect("Failed to open file for Rust script build generation");
        transaction_builder_generator::rust::output(&mut file, &abis, /* local types */ true)
            .expect("Failed to generate Rust builders for Diem");
    }

    std::process::Command::new("rustfmt")
        .arg("--config")
        .arg("merge_imports=true")
        .arg(TRANSACTION_BUILDERS_GENERATED_SOURCE_PATH)
        .status()
        .expect("Failed to run rustfmt on generated code");
}
