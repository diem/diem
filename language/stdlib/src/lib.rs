// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytecode_verifier::{verify_module, DependencyChecker};
use log::LevelFilter;
use move_lang::{compiled_unit::CompiledUnit, move_compile, shared::Address};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use vm::{normalized::Module, CompiledModule};

pub const STD_LIB_DIR: &str = "modules";
pub const MOVE_EXTENSION: &str = "move";
pub const ERROR_DESC_EXTENSION: &str = "errmap";
/// The extension for compiled files
pub const COMPILED_EXTENSION: &str = "mv";

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
pub const TRANSACTION_SCRIPT_DOC_TEMPLATE: &str = "transaction_scripts/overview_template.md";

pub const ERROR_DESC_DIR: &str = "error_descriptions";
pub const ERROR_DESC_FILENAME: &str = "error_descriptions";

pub const PACKED_TYPES_DIR: &str = "packed_types";
pub const PACKED_TYPES_FILENAME: &str = "packed_types";
pub const PACKED_TYPES_EXTENSION: &str = "txt";

/// The output path under which compiled script files can be found
pub const COMPILED_TRANSACTION_SCRIPTS_DIR: &str = "compiled/transaction_scripts";
/// The output path for transaction script ABIs.
pub const COMPILED_TRANSACTION_SCRIPTS_ABI_DIR: &str = "compiled/transaction_scripts/abi";

/// Where to write generated transaction builders.
pub const TRANSACTION_BUILDERS_GENERATED_SOURCE_PATH: &str =
    "../../client/transaction-builder/src/stdlib.rs";

pub fn filter_move_files(dir_iter: impl Iterator<Item = PathBuf>) -> impl Iterator<Item = PathBuf> {
    dir_iter.flat_map(|path| {
        if path.extension()?.to_str()? == MOVE_EXTENSION {
            Some(path)
        } else {
            None
        }
    })
}

pub fn filter_move_bytecode_files(
    dir_iter: impl Iterator<Item = PathBuf>,
) -> impl Iterator<Item = PathBuf> {
    dir_iter.flat_map(|path| {
        if path.extension()?.to_str()? == COMPILED_EXTENSION {
            Some(path)
        } else {
            None
        }
    })
}

pub fn path_in_crate<S>(relative: S) -> PathBuf
where
    S: Into<String>,
{
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative.into());
    path
}

pub fn stdlib_files() -> Vec<String> {
    let path = path_in_crate(STD_LIB_DIR);
    let dirfiles = datatest_stable::utils::iterate_directory(&path);
    filter_move_files(dirfiles)
        .flat_map(|path| path.into_os_string().into_string())
        .collect()
}

pub fn stdlib_bytecode_files() -> Vec<String> {
    let path = path_in_crate(COMPILED_OUTPUT_PATH);
    let names = stdlib_files();
    let dirfiles = datatest_stable::utils::iterate_directory(&path);
    let res: Vec<String> = filter_move_bytecode_files(dirfiles)
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
    let dirfiles = datatest_stable::utils::iterate_directory(&path);
    filter_move_files(dirfiles)
        .flat_map(|path| path.into_os_string().into_string())
        .collect()
}

pub fn build_stdlib() -> BTreeMap<String, CompiledModule> {
    let (_, compiled_units) =
        move_compile(&stdlib_files(), &[], Some(Address::LIBRA_CORE), None).unwrap();
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
                modules.insert(format!("{:0>3}_{}", i, name), module);
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
        None,
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
    build_doc(
        STD_LIB_DOC_DIR,
        "",
        Some(
            path_in_crate(STD_LIB_DOC_TEMPLATE)
                .to_string_lossy()
                .to_string(),
        ),
        stdlib_files().as_slice(),
        "",
    )
}

pub fn build_transaction_script_doc(script_files: &[String]) {
    build_doc(
        TRANSACTION_SCRIPTS_DOC_DIR,
        STD_LIB_DOC_DIR,
        Some(
            path_in_crate(TRANSACTION_SCRIPT_DOC_TEMPLATE)
                .to_string_lossy()
                .to_string(),
        ),
        script_files,
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

pub fn build_stdlib_error_code_map() {
    let mut path = PathBuf::from(COMPILED_OUTPUT_PATH);
    path.push(ERROR_DESC_DIR);
    fs::create_dir_all(&path).unwrap();
    path.push(ERROR_DESC_FILENAME);
    path.set_extension(ERROR_DESC_EXTENSION);
    build_error_code_map(path.to_str().unwrap(), stdlib_files().as_slice(), "")
}

fn build_doc(
    output_path: &str,
    doc_path: &str,
    template: Option<String>,
    sources: &[String],
    dep_path: &str,
) {
    let mut options = move_prover::cli::Options::default();
    options.move_sources = sources.to_vec();
    if !dep_path.is_empty() {
        options.move_deps = vec![dep_path.to_string()]
    }
    options.verbosity_level = LevelFilter::Warn;
    options.run_docgen = true;
    // Take the defaults here for docgen. Changes in options should be applied there so
    // command line and invocation here have same output.
    if template.is_some() {
        options.docgen.root_doc_template = template;
    }
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

pub fn get_packed_types_path() -> PathBuf {
    let mut path = PathBuf::from(COMPILED_OUTPUT_PATH);
    path.push(PACKED_TYPES_DIR);
    path.push(PACKED_TYPES_FILENAME);
    path.set_extension(PACKED_TYPES_EXTENSION);
    path
}

pub fn build_packed_types_map() {
    let mut options = move_prover::cli::Options::default();
    let mut path = PathBuf::from(COMPILED_OUTPUT_PATH);
    path.push(PACKED_TYPES_DIR);
    fs::create_dir_all(&path).unwrap();
    path.push(PACKED_TYPES_FILENAME);
    path.set_extension(PACKED_TYPES_EXTENSION);
    options.output_path = path.to_str().unwrap().to_string();
    let mut sources = stdlib_files();
    sources.append(&mut script_files());
    options.move_sources = sources.to_vec();
    options.verbosity_level = LevelFilter::Warn;
    options.run_packed_types_gen = true;
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

fn build_error_code_map(output_path: &str, sources: &[String], dep_path: &str) {
    let mut options = move_prover::cli::Options::default();
    options.move_sources = sources.to_vec();
    if !dep_path.is_empty() {
        options.move_deps = vec![dep_path.to_string()]
    }
    options.verbosity_level = LevelFilter::Warn;
    options.run_errmapgen = true;
    options.errmapgen.output_file = output_path.to_string();
    options.setup_logging_for_test();
    move_prover::run_move_prover_errors_to_stderr(options).unwrap();
}

pub fn generate_rust_transaction_builders() {
    let abis = transaction_builder_generator::read_abis(COMPILED_TRANSACTION_SCRIPTS_ABI_DIR)
        .expect("Failed to read generated ABIs");
    {
        let mut file = std::fs::File::create(TRANSACTION_BUILDERS_GENERATED_SOURCE_PATH)
            .expect("Failed to open file for Rust script build generation");
        transaction_builder_generator::rust::output(&mut file, &abis, /* local types */ true)
            .expect("Failed to generate Rust builders for Libra");
    }

    std::process::Command::new("rustfmt")
        .arg("--config")
        .arg("merge_imports=true")
        .arg(TRANSACTION_BUILDERS_GENERATED_SOURCE_PATH)
        .status()
        .expect("Failed to run rustfmt on generated code");
}

/// The result of a linking and layoutcompatibility check. Here is what the different combinations
/// mean:
/// `{ struct: true, struct_layout: true }`: fully backward compatible
/// `{ struct_and_function_linking: true, struct_layout: false }`: Dependent modules that reference functions or types in this module may not link. However, fixing, recompiling, and redeploying all dependent modules will work--no data migration needed.
/// `{ type_and_function_linking: true, struct_layout: false }`: Attempting to read structs published by this module will now fail at runtime. However, dependent modules will continue to link. Requires data migration, but no changes to dependent modules.
/// `{ type_and_function_linking: false, struct_layout: false }`: Everything is broken. Need both a data migration and changes to dependent modules.
pub struct Compatibility {
    /// If false, dependent modules that reference functions or structs in this module may not link
    pub struct_and_function_linking: bool,
    /// If false, attempting to read structs previously published by this module will fail at runtime
    pub struct_layout: bool,
}

impl Compatibility {
    /// Return true if the two module s compared in the compatiblity check are both linking and
    /// layout compatible.
    pub fn is_fully_compatible(&self) -> bool {
        self.struct_and_function_linking && self.struct_layout
    }

    /// Return compatibility assessment for `new_module` relative to old module `old_module`
    pub fn check(old_module: &Module, new_module: &Module) -> Compatibility {
        let mut struct_and_function_linking = true;
        let mut struct_layout = true;

        // module's name and address are unchanged
        if old_module.address != new_module.address || old_module.name != new_module.name {
            struct_and_function_linking = false;
        }

        // old module's structs are a subset of the new module's structs
        for old_struct in &old_module.structs {
            match new_module
                .structs
                .iter()
                .find(|s| s.name == old_struct.name)
            {
                Some(new_struct) => {
                    if new_struct.kind != old_struct.kind
                        || new_struct.type_parameters != old_struct.type_parameters
                    {
                        // Declared kind and/or type parameters changed. Existing modules that depend on this struct will fail to link with the new version of the module
                        struct_and_function_linking = false;
                        // This does not change the struct layout, but it may leave some published
                        // values "orphaned". For example: if
                        // `resource struct S<T: copyable> { t : T }` is changed to
                        // `resource struct S<T: resource> { t : T}`, code can no longer access
                        // published values of type (e.g.) `S<u64>`.
                    }
                    if new_struct.fields != old_struct.fields {
                        // Fields changed. Code in this module will fail at runtime if it tries to
                        // read a previously published struct value
                        // TODO: this is a stricter definition than required. We could in principle
                        // choose to label the following as compatible
                        // (1) changing the name (but not position or type) of a field. The VM does
                        //     not care about the name of a field (it's purely informational), but
                        //     clients presumably do.
                        // (2) changing the type of a field to a different, but layout and kind
                        //     compatible type. E.g. `struct S { b: bool }` to `struct S { b: B }`
                        // where
                        //     B is struct B { some_name: bool }. TODO: does this affect clients? I
                        //     think not--the serialization of the same data with these two types
                        //     will be the same.
                        struct_layout = false
                    }
                }
                None => {
                    // Struct not present in new . Existing modules that depend on this struct will fail to link with the new version of the module.
                    struct_and_function_linking = false;
                    // Note: we intentionally do *not* label this a layout compatibility violation.
                    // Existing modules can still successfully read previously published values of
                    // this struct `Parent::T`. That is, code like the function `foo` in
                    // ```
                    // struct S { t: Parent::T }
                    // public fun foo(a: addr): S { move_from<S>(addr) }
                    // ```
                    // in module `Child` will continue to run without error. But values of type
                    // `Parent::T` in `Child` are now "orphaned" in the sense that `Parent` no
                    // longer exposes any API for reading/writing them.
                }
            }
        }

        // old module's public functions are a subset of the new module's public functions
        for function in &old_module.public_functions {
            if !new_module.public_functions.contains(&function) {
                struct_and_function_linking = false;
            }
        }

        Compatibility {
            struct_and_function_linking,
            struct_layout,
        }
    }

    /// Return true if `new_module` can safely update `old_module`
    pub fn can_update(
        old_module: &Module,
        new_module: &CompiledModule,
        _new_module_dependencies: &[CompiledModule],
    ) -> bool {
        // (1) Verify new_module (TODO)
        // (2) Link new_module against new_module dependencies. (TODO)
        //     Note: this will *NOT* prevent cylic deps. We need to think about a different scheme
        //     if we care about this (wich we almost certainly do). One (probably too restrictive)
        //     solution would be: insist that deps(new_module) are a subset of deps(old_module).
        //     That would not only prevent cyclic deps, but also preclude the need for linking
        //     entirely.
        // (3) Extract the  for new_module and check compatibility with old_module
        Self::is_fully_compatible(&Self::check(old_module, &Module::new(new_module)))
            && panic!("TODO: implement verification, linking, cyclic deps checks")
    }
}
