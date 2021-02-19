// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod transaction_scripts;

use include_dir::{include_dir, Dir};
use move_package::package::{CompiledPackage, VerifiedPackage};
use once_cell::sync::Lazy;

// This needs to be a string literal due to restrictions imposed by include_bytes.
/// The compiled library needs to be included in the Rust binary due to Docker deployment issues.
/// This is why we include it here.
pub const COMPILED_STDLIB_DIR: Dir = include_dir!("stdlib");

pub const ERROR_DESCRIPTIONS: &[u8] =
    std::include_bytes!("../error_descriptions/error_descriptions.errmap");

static PACKAGE_NAME: &str = "diem-framework";

// The compiled version of the Move standard library.
// Similarly to genesis, we keep a compiled version of the standard library and scripts around, and
// only periodically update these. This has the effect of decoupling the current leading edge of
// compiler development from the current stdlib used in genesis/scripts.  In particular, changes in
// the compiler will not affect the script hashes or stdlib until we have tested the changes to our
// satisfaction. Then we can generate a new compiled version of the stdlib/scripts (and will need to
// regenerate genesis).
static PACKAGE_MOVELANG_STDLIB: Lazy<VerifiedPackage> = Lazy::new(|| {
    let module_bytes = COMPILED_STDLIB_DIR
        .files()
        .iter()
        .map(|file| file.contents().to_vec());
    CompiledPackage::from_bytes(PACKAGE_NAME.to_owned(), module_bytes)
        .unwrap_or_else(|_| panic!("{} modules failed to deserialize", PACKAGE_NAME))
        .verify(vec![])
        .unwrap_or_else(|_| panic!("{} modules failed to verify", PACKAGE_NAME))
});

/// Returns a reference to the standard library package.
pub fn stdlib_modules() -> &'static VerifiedPackage {
    &*PACKAGE_MOVELANG_STDLIB
}
