// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod transaction_scripts;

use bytecode_verifier::{batch_verify_modules, VerifiedModule};
use move_lang::{move_compile, shared::Address, to_bytecode::translate::CompiledUnit};
use once_cell::sync::Lazy;
use std::path::PathBuf;

pub const STD_LIB_DIR: &str = "modules";
pub const MOVE_EXTENSION: &str = "move";

static MOVELANG_STDLIB: Lazy<Vec<VerifiedModule>> = Lazy::new(build_stdlib);

/// Returns a reference to the standard library built by move-lang compiler, compiled with the
/// [default address](account_config::core_code_address).
///
/// The order the modules are presented in is important: later modules depend on earlier ones.
pub fn stdlib_modules() -> &'static [VerifiedModule] {
    &*MOVELANG_STDLIB
}

pub fn stdlib_files() -> Vec<String> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(STD_LIB_DIR);
    let dirfiles = datatest_stable::utils::iterate_directory(&path);
    dirfiles
        .flat_map(|path| {
            if path.extension()?.to_str()? == MOVE_EXTENSION {
                path.into_os_string().into_string().ok()
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

pub fn build_stdlib() -> Vec<VerifiedModule> {
    let (_, compiled_units) =
        move_compile(&stdlib_files(), &[], Some(Address::LIBRA_CORE)).unwrap();
    batch_verify_modules(
        compiled_units
            .into_iter()
            .map(|compiled_unit| match compiled_unit {
                CompiledUnit::Module(_, m) => m,
                CompiledUnit::Script(_, _) => panic!("Unexpected Script in stdlib"),
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
        CompiledUnit::Module(_, _) => panic!("Unexpected module when compiling script"),
        CompiledUnit::Script(_, s) => s.serialize(&mut script_bytes).unwrap(),
    };
    script_bytes
}
