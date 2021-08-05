// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use anyhow::Result;
use move_lang::{
    self,
    shared::{AddressBytes, Flags},
    Compiler,
};

/// Type-check the user modules in `files` and the dependencies in `interface_files`
pub fn check(
    interface_files: &[String],
    sources_shadow_deps: bool,
    files: &[String],
    named_addresses: BTreeMap<String, AddressBytes>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Checking Move files...");
    }
    Compiler::new(files, interface_files)
        .set_flags(Flags::empty().set_sources_shadow_deps(sources_shadow_deps))
        .set_named_address_values(named_addresses)
        .check_and_report()?;
    Ok(())
}
