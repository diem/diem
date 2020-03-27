// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod transaction_scripts;

use bytecode_verifier::{batch_verify_modules, VerifiedModule};
use move_lang::{compiled_unit::CompiledUnit, move_compile, shared::Address};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use vm::file_format::CompiledModule;

pub const STD_LIB_DIR: &str = "modules";
pub const MOVE_EXTENSION: &str = "move";

pub const NO_USE_STAGED: &str = "MOVE_NO_USE_STAGED";

pub const TRANSACTION_SCRIPTS: &str = "transaction_scripts";
/// The output path under which staged files will be put
pub const STAGED_OUTPUT_PATH: &str = "staged";
/// The file name for the staged stdlib
pub const STAGED_STDLIB_NAME: &str = "stdlib";
/// The extension for staged files
pub const STAGED_EXTENSION: &str = "mv";

// The current stdlib that is freshly built. This will never be used in deployment so we don't need
// to pull the same trick here in order to include this in the Rust binary.
static FRESH_MOVELANG_STDLIB: Lazy<Vec<VerifiedModule>> = Lazy::new(build_stdlib);

// This needs to be a string literal due to restrictions imposed by include_bytes.
/// The staged library needs to be included in the Rust binary due to Docker deployment issues.
/// This is why we include it here.
pub const STAGED_STDLIB_BYTES: &[u8] = std::include_bytes!("../staged/stdlib.mv");

// The staged version of the move standard library.
// Similarly to genesis, we keep a compiled version of the standard library and scripts around, and
// only periodically update these. This has the effect of decoupling the current leading edge of
// compiler development from the current stdlib used in genesis/scripts.  In particular, changes in
// the compiler will not affect the script hashes or stdlib until we have tested the changes to our
// satisfaction. Then we can generate a new staged version of the stdlib/scripts (and will need to
// regenerate genesis). The staged version of the stdlib/scripts are used unless otherwise
// specified either by the MOVE_NO_USE_STAGED env var, or by passing the "StdLibOptions::Fresh"
// option to `stdlib_modules`.
static STAGED_MOVELANG_STDLIB: Lazy<Vec<VerifiedModule>> = Lazy::new(|| {
    let modules = lcs::from_bytes::<Vec<Vec<u8>>>(STAGED_STDLIB_BYTES)
        .unwrap()
        .into_iter()
        .map(|bytes| CompiledModule::deserialize(&bytes).unwrap())
        .collect();
    batch_verify_modules(modules)
});

/// An enum specifying whether the staged stdlib/scripts should be used or freshly built versions
/// should be used.
#[derive(Debug, Eq, PartialEq)]
pub enum StdLibOptions {
    Staged,
    Fresh,
}

/// Returns a reference to the standard library. Depending upon the `option` flag passed in
/// either a staged version of the standard library will be returned or a new freshly built stdlib
/// will be used.
pub fn stdlib_modules(option: StdLibOptions) -> &'static [VerifiedModule] {
    match option {
        StdLibOptions::Staged => &*STAGED_MOVELANG_STDLIB,
        StdLibOptions::Fresh => &*FRESH_MOVELANG_STDLIB,
    }
}

/// Returns a reference to the standard library built by move-lang compiler, compiled with the
/// [default address](account_config::core_code_address).
///
/// The order the modules are presented in is important: later modules depend on earlier ones.
/// The defualt is to return a staged version of the stdlib unless it is otherwise specified by the
/// `MOVE_NO_USE_STAGED` environment variable.
pub fn env_stdlib_modules() -> &'static [VerifiedModule] {
    let option = if use_staged() {
        StdLibOptions::Staged
    } else {
        StdLibOptions::Fresh
    };
    stdlib_modules(option)
}

/// A predicate detailing whether the staged versions of scripts and the stdlib should be used or
/// not. The default is that the staged versions of the stdlib and transaction scripts should be
/// used.
pub fn use_staged() -> bool {
    std::env::var(NO_USE_STAGED).is_err()
}

pub fn filter_move_files(dir_iter: impl Iterator<Item = PathBuf>) -> impl Iterator<Item = String> {
    dir_iter.flat_map(|path| {
        if path.extension()?.to_str()? == MOVE_EXTENSION {
            path.into_os_string().into_string().ok()
        } else {
            None
        }
    })
}

pub fn stdlib_files() -> Vec<String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(STD_LIB_DIR);
    let dirfiles = datatest_stable::utils::iterate_directory(&path);
    filter_move_files(dirfiles).collect::<Vec<_>>()
}

pub fn build_stdlib() -> Vec<VerifiedModule> {
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
