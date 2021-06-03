// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_lang::{self, shared::Flags};

/// Type-check the user modules in `files` and the dependencies in `interface_files`
pub fn check(
    interface_files: &[String],
    sources_shadow_deps: bool,
    files: &[String],
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Checking Move files...");
    }
    move_lang::move_check_and_report(
        files,
        interface_files,
        None,
        Flags::empty().set_sources_shadow_deps(sources_shadow_deps),
    )?;
    Ok(())
}
