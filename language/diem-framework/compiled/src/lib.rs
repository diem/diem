// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod legacy;
pub mod tmp_new_transaction_script_builders;

use bytecode_verifier::{cyclic_dependencies, dependencies, verify_module};
use diem_framework::build_stdlib;
use include_dir::{include_dir, Dir};
use once_cell::sync::Lazy;
use std::collections::BTreeMap;
use vm::{access::ModuleAccess, file_format::CompiledModule};

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
static COMPILED_MOVELANG_STDLIB_WITH_BYTES: Lazy<(Vec<Vec<u8>>, Vec<CompiledModule>)> =
    Lazy::new(|| {
        let modules: BTreeMap<&str, (Vec<u8>, CompiledModule)> = COMPILED_STDLIB_DIR
            .files()
            .iter()
            .map(|file| {
                let name = file.path().to_str().unwrap();
                let bytes = file.contents().to_vec();
                let module = CompiledModule::deserialize(&bytes).unwrap();
                (name, (bytes, module))
            })
            .collect();

        let mut module_bytes = vec![];
        let mut verified_modules = vec![];
        for (_, (bytes, module)) in modules.into_iter() {
            verify_module(&module).expect("stdlib module failed to verify");
            dependencies::verify_module(&module, &verified_modules)
                .expect("stdlib module dependency failed to verify");
            module_bytes.push(bytes);
            verified_modules.push(module);
        }
        let modules_by_id: BTreeMap<_, _> = verified_modules
            .iter()
            .map(|module| (module.self_id(), module))
            .collect();
        for module in modules_by_id.values() {
            cyclic_dependencies::verify_module(module, |module_id| {
                Ok(modules_by_id
                    .get(module_id)
                    .expect("missing module in stdlib")
                    .immediate_module_dependencies())
            })
            .expect("stdlib module has cyclic dependencies");
        }
        (module_bytes, verified_modules)
    });

/// An enum specifying whether the compiled stdlib/scripts should be used or freshly built versions
/// should be used.
#[derive(Debug, Eq, PartialEq)]
pub enum StdLibOptions {
    Compiled,
    Fresh,
}

/// Return value of `stdlib_modules`. Gives a reference to the Compiled modules and a reference
/// to the on disk bytes, if the `Compiled` option was used
pub struct StdLibModules {
    pub compiled_modules: &'static [CompiledModule],
    pub bytes_opt: Option<&'static [Vec<u8>]>,
}

impl StdLibModules {
    /// If bytes_opt is Some, returns an owned copy of the the types
    /// If it is is None, it serializes `compiled_modules`
    pub fn bytes_vec(&self) -> Vec<Vec<u8>> {
        match self.bytes_opt {
            Some(bytes) => bytes.to_vec(),
            None => self
                .compiled_modules
                .iter()
                .map(|m| {
                    let mut bytes = vec![];
                    m.serialize(&mut bytes).unwrap();
                    bytes
                })
                .collect(),
        }
    }
}

/// Returns a reference to the standard library. Depending upon the `option` flag passed in
/// either a compiled version of the standard library will be returned or a new freshly built stdlib
/// will be used.
/// For the `Compiled` option, both the on disk bytes and the deserialized modules will be returned
/// For the `Fresh` option, only the compiled modules will be given as there are no serialized bytes
/// to borrow and give a reference to
pub fn stdlib_modules(option: StdLibOptions) -> StdLibModules {
    match option {
        StdLibOptions::Compiled => StdLibModules {
            bytes_opt: Some(&*COMPILED_MOVELANG_STDLIB_WITH_BYTES.0),
            compiled_modules: &*COMPILED_MOVELANG_STDLIB_WITH_BYTES.1,
        },
        StdLibOptions::Fresh => StdLibModules {
            bytes_opt: None,
            compiled_modules: &*FRESH_MOVELANG_STDLIB,
        },
    }
}

/// Returns a reference to the standard library built by move-lang compiler, compiled with the
/// [default address](account_config::core_code_address).
///
/// The order the modules are presented in is important: later modules depend on earlier ones.
/// The defualt is to return a compiled version of the stdlib unless it is otherwise specified by the
/// `MOVE_NO_USE_COMPILED` environment variable.
pub fn env_stdlib_modules() -> StdLibModules {
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
