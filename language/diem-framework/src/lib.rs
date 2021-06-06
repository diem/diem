// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytecode_verifier::{cyclic_dependencies, dependencies, verify_module};
use move_binary_format::{access::ModuleAccess, file_format::CompiledModule};
use move_lang::{compiled_unit::CompiledUnit, move_compile_and_report, shared::Flags};
use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub use move_stdlib::{COMPILED_EXTENSION, ERROR_DESC_EXTENSION, MOVE_EXTENSION};

pub mod release;

const MODULES_DIR: &str = "modules";

/// The output path under which compiled files will be put
pub const COMPILED_OUTPUT_PATH: &str = "compiled";

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

pub(crate) fn build_stdlib() -> BTreeMap<String, CompiledModule> {
    let (_files, compiled_units) =
        move_compile_and_report(&diem_stdlib_files(), &[], None, Flags::empty()).unwrap();
    let mut modules = BTreeMap::new();
    for compiled_unit in compiled_units {
        let name = compiled_unit.name();
        match compiled_unit {
            CompiledUnit::Module { module, .. } => {
                verify_module(&module).expect("stdlib module failed to verify");
                dependencies::verify_module(&module, modules.values())
                    .expect("stdlib module dependency failed to verify");
                modules.insert(name, module);
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

static MODULES: Lazy<Vec<CompiledModule>> = Lazy::new(|| {
    build_stdlib()
        .into_iter()
        .map(|(_key, value)| value)
        .collect()
});

static MODULE_BLOBS: Lazy<Vec<Vec<u8>>> = Lazy::new(|| {
    MODULES
        .iter()
        .map(|module| {
            let mut bytes = vec![];
            module.serialize(&mut bytes).unwrap();
            bytes
        })
        .collect()
});

pub fn modules() -> &'static [CompiledModule] {
    &MODULES
}

pub fn module_blobs() -> &'static [Vec<u8>] {
    &MODULE_BLOBS
}

fn save_binary(path: &Path, binary: &[u8]) -> bool {
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
