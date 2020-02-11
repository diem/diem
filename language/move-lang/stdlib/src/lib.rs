// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::{batch_verify_modules, VerifiedModule};
use move_lang::{move_compile, shared::Address, to_bytecode::translate::CompiledUnit};
use once_cell::sync::Lazy;
use std::path::PathBuf;

pub const STD_LIB_DIR: &str = "modules";
pub const MOVE_EXTENSION: &str = "move";

static MOVELANG_STDLIB: Lazy<Vec<VerifiedModule>> = Lazy::new(build_move_lang_stdlib);

/// Returns a reference to the standard library built by move-lang compiler, compiled with the
/// [default address](account_config::core_code_address).
///
/// The order the modules are presented in is important: later modules depend on earlier ones.
pub fn move_lang_stdlib_modules() -> &'static [VerifiedModule] {
    &*MOVELANG_STDLIB
}

pub fn build_move_lang_stdlib() -> Vec<VerifiedModule> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(STD_LIB_DIR);
    let dirfiles = datatest_stable::utils::iterate_directory(&path);
    let stdlib_target = dirfiles
        .flat_map(|path| {
            if path.extension()?.to_str()? == MOVE_EXTENSION {
                path.into_os_string().into_string().ok()
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let (_, compiled_units) = move_compile(&stdlib_target, &[], Some(Address::LIBRA_CORE)).unwrap();
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
