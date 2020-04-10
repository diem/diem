// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod transaction_scripts;

use bytecode_verifier::{batch_verify_modules, VerifiedModule};
use once_cell::sync::Lazy;
use vm::file_format::CompiledModule;

// This needs to be a string literal due to restrictions imposed by include_bytes.
/// The staged library needs to be included in the Rust binary due to Docker deployment issues.
/// This is why we include it here.
const STAGED_STDLIB_BYTES: &[u8] = std::include_bytes!("../staged/stdlib.mv");

// The staged version of the move standard library.
// Similarly to genesis, we keep a compiled version of the standard library and scripts around, and
// only periodically update these. This has the effect of decoupling the current leading edge of
// compiler development from the current stdlib used in genesis/scripts.  In particular, changes in
// the compiler will not affect the script hashes or stdlib until we have tested the changes to our
// satisfaction. Then we can generate a new staged version of the stdlib/scripts (and will need to
// regenerate genesis).
static STAGED_MOVELANG_STDLIB: Lazy<Vec<VerifiedModule>> = Lazy::new(|| {
    let modules = lcs::from_bytes::<Vec<Vec<u8>>>(STAGED_STDLIB_BYTES)
        .unwrap()
        .into_iter()
        .map(|bytes| CompiledModule::deserialize(&bytes).unwrap())
        .collect();
    batch_verify_modules(modules)
});

/// Returns a reference to the standard library modules. This will be stale if the Move source code
/// for the standard library has changed, but the modules have not been rebuilt
pub fn stdlib_modules() -> &'static [VerifiedModule] {
    &*STAGED_MOVELANG_STDLIB
}
