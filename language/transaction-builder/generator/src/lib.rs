// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::transaction::ScriptABI;
use std::io::Read;

/// Support for code-generation in C++17.
pub mod cpp;
/// Support for code-generation in Java 8.
pub mod java;
/// Support for code-generation in Python 3.
pub mod python3;
/// Support for code-generation in Rust.
pub mod rust;

/// Internals shared between languages.
mod common;

/// Read all ABI files in a directory.
pub fn read_abis<P: AsRef<std::path::Path>>(dir_path: P) -> anyhow::Result<Vec<ScriptABI>> {
    let mut abis = Vec::<ScriptABI>::new();
    for entry in std::fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let mut buffer = Vec::new();
        let mut f = std::fs::File::open(path)?;
        f.read_to_end(&mut buffer)?;
        abis.push(lcs::from_bytes(&buffer)?);
    }
    // Sorting scripts by alphabetical order.
    abis.sort_by(|a, b| a.name().cmp(b.name()));
    Ok(abis)
}

/// How to copy ABI-generated source code for a given language.
pub trait SourceInstaller {
    type Error;

    /// Create a module exposing the transaction builders for the given ABIs.
    fn install_transaction_builders(
        &self,
        name: &str,
        abis: &[ScriptABI],
    ) -> std::result::Result<(), Self::Error>;
}
