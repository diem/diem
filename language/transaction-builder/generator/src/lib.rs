// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::transaction::ScriptABI;
use serde_generate::CustomCode;
use std::io::Read;

/// Support for code-generation in C++17.
pub mod cpp;
/// Support for code-generation in Go >= 1.13.
pub mod golang;
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
        abis.push(bcs::from_bytes(&buffer)?);
    }
    // Sorting scripts by alphabetical order.
    #[allow(clippy::unnecessary_sort_by)]
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

/// How to read custom code to inject in Diem containers.
pub fn read_custom_code_from_paths<'a, I>(package: &'a [&'a str], paths: I) -> CustomCode
where
    I: Iterator<Item = std::path::PathBuf>,
{
    paths
        .map(|path| {
            let container_name = path
                .file_stem()
                .expect("file name must have a non-empty stem")
                .to_str()
                .expect("file names must be valid UTF8")
                .to_string();
            let mut container_path = package.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            container_path.push(container_name);
            let content = std::fs::read_to_string(path).expect("custom code file must be readable");
            // Skip initial comments (e.g. copyright headers) and empty lines.
            let lines = content.lines().skip_while(|line| {
                line.starts_with("// ") || line.starts_with("# ") || line.is_empty()
            });
            let mut code = lines.collect::<Vec<_>>().join("\n");
            if !code.ends_with('\n') {
                code += "\n";
            }
            (container_path, code)
        })
        .collect()
}
