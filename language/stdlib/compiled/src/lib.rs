// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod transaction_scripts;

use bytecode_verifier::{verify_module, DependencyChecker};
use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;
use std::collections::BTreeMap;
use stdlib::build_stdlib;
use vm::file_format::CompiledModule;

pub const NO_USE_COMPILED: &str = "MOVE_NO_USE_COMPILED";

// The current stdlib that is freshly built. This will never be used in deployment so we don't need
// to pull the same trick here in order to include this in the Rust binary.
static FRESH_MOVELANG_STDLIB: Lazy<Vec<CompiledModule>> =
    Lazy::new(|| build_stdlib().values().cloned().collect());

// This needs to be a string literal due to restrictions imposed by include_bytes.
/// The compiled library needs to be included in the Rust binary due to Docker deployment issues.
/// This is why we include it here.
pub const COMPILED_STDLIB_DIR: Dir = include_dir!("stdlib");

pub const ERROR_DESCRIPTIONS: &[u8] =
    std::include_bytes!("../error_descriptions/error_descriptions.errmap");

// The compiled version of the Move standard library.
// Similarly to genesis, we keep a compiled version of the standard library and scripts around, and
// only periodically update these. This has the effect of decoupling the current leading edge of
// compiler development from the current stdlib used in genesis/scripts.  In particular, changes in
// the compiler will not affect the script hashes or stdlib until we have tested the changes to our
// satisfaction. Then we can generate a new compiled version of the stdlib/scripts (and will need to
// regenerate genesis). The compiled version of the stdlib/scripts are used unless otherwise
// specified either by the MOVE_NO_USE_COMPILED env var, or by passing the "StdLibOptions::Fresh"
// option to `stdlib_modules`.
static COMPILED_MOVELANG_STDLIB: Lazy<Vec<CompiledModule>> = Lazy::new(|| {
    let modules: BTreeMap<&str, CompiledModule> = COMPILED_STDLIB_DIR
        .files()
        .iter()
        .map(|file| {
            (
                file.path().to_str().unwrap(),
                CompiledModule::deserialize(&file.contents()).unwrap(),
            )
        })
        .collect();

    let mut verified_modules = vec![];
    for (_, module) in modules.into_iter() {
        verify_module(&module).expect("stdlib module failed to verify");
        DependencyChecker::verify_module(&module, &verified_modules)
            .expect("stdlib module dependency failed to verify");
        verified_modules.push(module)
    }
    verified_modules
});

/// An enum specifying whether the compiled stdlib/scripts should be used or freshly built versions
/// should be used.
#[derive(Debug, Eq, PartialEq)]
pub enum StdLibOptions {
    Compiled,
    Fresh,
}

/// Returns a reference to the standard library. Depending upon the `option` flag passed in
/// either a compiled version of the standard library will be returned or a new freshly built stdlib
/// will be used.
pub fn stdlib_modules(option: StdLibOptions) -> &'static [CompiledModule] {
    match option {
        StdLibOptions::Compiled => &*COMPILED_MOVELANG_STDLIB,
        StdLibOptions::Fresh => &*FRESH_MOVELANG_STDLIB,
    }
}

/// Returns a reference to the standard library built by move-lang compiler, compiled with the
/// [default address](account_config::core_code_address).
///
/// The order the modules are presented in is important: later modules depend on earlier ones.
/// The defualt is to return a compiled version of the stdlib unless it is otherwise specified by the
/// `MOVE_NO_USE_COMPILED` environment variable.
pub fn env_stdlib_modules() -> &'static [CompiledModule] {
    let option = if use_compiled() {
        StdLibOptions::Compiled
    } else {
        StdLibOptions::Fresh
    };
    stdlib_modules(option)
}

/// A predicate detailing whether the compiled versions of scripts and the stdlib should be used or
/// not. The default is that the compiled versions of the stdlib and transaction scripts should be
/// used.
pub fn use_compiled() -> bool {
    std::env::var(NO_USE_COMPILED).is_err()
}
