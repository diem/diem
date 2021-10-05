// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use framework_releases::{Release, ReleaseFetcher};
use move_binary_format::file_format::CompiledModule;
use once_cell::sync::Lazy;

/// Load the serialized modules from the specified release.
pub fn load_modules_from_release(release_name: &str) -> Result<Vec<Vec<u8>>> {
    ReleaseFetcher::new(Release::Experimental, release_name).module_blobs()
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
