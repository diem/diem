// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_lang::{self, shared::Flags, Compiler};

/// Compile the user modules in `sources` against the dependencies in `interface_files`, placing
/// the resulting binaries in `output_dir`.
pub fn compile(
    interface_files: &[String],
    output_dir: &str,
    sources_shadow_deps: bool,
    sources: &[String],
    emit_source_map: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Compiling Move files...");
    }
    let (files, compiled_units) = Compiler::new(sources, interface_files)
        .set_flags(Flags::empty().set_sources_shadow_deps(sources_shadow_deps))
        .build_and_report()?;
    move_lang::output_compiled_units(emit_source_map, files, compiled_units, &output_dir)
}
