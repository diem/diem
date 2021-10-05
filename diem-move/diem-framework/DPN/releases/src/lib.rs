// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use diem_types::transaction::ScriptFunction;
use framework_releases::{Release, ReleaseFetcher};
use move_binary_format::file_format::CompiledModule;
use move_command_line_common::files::{extension_equals, find_filenames, MOVE_COMPILED_EXTENSION};
use once_cell::sync::Lazy;
use std::{convert::TryFrom, path::PathBuf};

pub mod legacy;

#[cfg(test)]
mod tests;

/// Return a list of all available releases.
pub fn list_all_releases() -> Result<Vec<String>> {
    Ok(ReleaseFetcher::list_releases(&Release::Experimental))
}

/// Load the serialized modules from the specified release.
pub fn load_modules_from_release(release_name: &str) -> Result<Vec<Vec<u8>>> {
    ReleaseFetcher::new(Release::Experimental, release_name).module_blobs()
}

/// Load the error descriptions from the specified release.
pub fn load_error_descriptions_from_release(release_name: &str) -> Result<Vec<u8>> {
    ReleaseFetcher::new(Release::Experimental, release_name).error_descriptions()
}

/// Load the serialized modules from the specified paths.
pub fn load_modules_from_paths(paths: &[PathBuf]) -> Vec<Vec<u8>> {
    find_filenames(paths, |path| {
        extension_equals(path, MOVE_COMPILED_EXTENSION)
    })
    .expect("module loading failed")
    .iter()
    .map(|file_name| std::fs::read(file_name).unwrap())
    .collect::<Vec<_>>()
}

static CURRENT_MODULE_BLOBS: Lazy<Vec<Vec<u8>>> =
    Lazy::new(|| load_modules_from_release("current").unwrap());

static CURRENT_MODULES: Lazy<Vec<CompiledModule>> = Lazy::new(|| {
    CURRENT_MODULE_BLOBS
        .iter()
        .map(|blob| CompiledModule::deserialize(blob).unwrap())
        .collect()
});

pub fn current_modules() -> &'static [CompiledModule] {
    &CURRENT_MODULES
}

pub fn current_module_blobs() -> &'static [Vec<u8>] {
    &CURRENT_MODULE_BLOBS
}

pub fn current_modules_with_blobs(
) -> impl Iterator<Item = (&'static Vec<u8>, &'static CompiledModule)> {
    CURRENT_MODULE_BLOBS.iter().zip(CURRENT_MODULES.iter())
}

static CURRENT_ERROR_DESCRIPTIONS: Lazy<Vec<u8>> =
    Lazy::new(|| load_error_descriptions_from_release("current").unwrap());

pub fn current_error_descriptions() -> &'static [u8] {
    &CURRENT_ERROR_DESCRIPTIONS
}

pub fn name_for_script(bytes: &[u8]) -> Result<String> {
    if let Ok(script) = legacy::transaction_scripts::LegacyStdlibScript::try_from(bytes) {
        Ok(format!("{}", script))
    } else {
        bcs::from_bytes::<ScriptFunction>(bytes)
            .map(|script| {
                format!(
                    "{}::{}::{}",
                    script.module().address().short_str_lossless(),
                    script.module().name(),
                    script.function()
                )
            })
            .map_err(|err| err.into())
    }
}
