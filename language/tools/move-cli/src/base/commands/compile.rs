// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use anyhow::Result;
use move_lang::{
    self,
    shared::{AddressBytes, Flags},
    Compiler,
};

/// Compile the user modules in `sources` against the dependencies in `interface_files`, placing
/// the resulting binaries in `output_dir`.
pub fn compile(
    interface_files: &[String],
    output_dir: &str,
    sources_shadow_deps: bool,
    sources: &[String],
    named_address_mapping: BTreeMap<String, AddressBytes>,
    emit_source_map: bool,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Compiling Move files...");
    }
    let (files, compiled_units) = Compiler::new(sources, interface_files)
        .set_flags(Flags::empty().set_sources_shadow_deps(sources_shadow_deps))
        .set_named_address_values(named_address_mapping)
        .build_and_report()?;
    move_lang::output_compiled_units(emit_source_map, files, compiled_units, output_dir)
}
