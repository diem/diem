// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Cache for commonly executed scripts

use crate::loaded_data::{
    function::{FunctionRef, FunctionReference},
    loaded_module::LoadedModule,
};
use logger::prelude::*;
use tiny_keccak::Keccak;
use types::transaction::SCRIPT_HASH_LENGTH;
use vm::{errors::VMResult, file_format::CompiledScript};
use vm_cache_map::{Arena, CacheMap};

/// The cache for commonly executed scripts. Currently there's no eviction policy, and it maps
/// hash of script bytes into `FunctionRef`.
pub struct ScriptCache<'alloc> {
    map: CacheMap<'alloc, [u8; SCRIPT_HASH_LENGTH], LoadedModule, FunctionRef<'alloc>>,
}

impl<'alloc> ScriptCache<'alloc> {
    /// Create a new ScriptCache.
    pub fn new(allocator: &'alloc Arena<LoadedModule>) -> Self {
        ScriptCache {
            map: CacheMap::new(allocator),
        }
    }

    /// Cache and resolve `script` into a `FunctionRef` that can be executed
    pub fn cache_script(
        &self,
        script: CompiledScript,
        raw_bytes: &[u8],
    ) -> VMResult<FunctionRef<'alloc>> {
        let mut hash = [0u8; SCRIPT_HASH_LENGTH];
        let mut keccak = Keccak::new_sha3_256();

        keccak.update(raw_bytes);
        keccak.finalize(&mut hash);

        if let Some(f) = self.map.get(&hash) {
            trace!("[VM] Script cache hit");
            Ok(Ok(f))
        } else {
            trace!("[VM] Script cache miss");
            let fake_module = script.into_module();
            let loaded_module = LoadedModule::new(fake_module)?;
            self.map
                .or_insert_with_try_transform(
                    hash,
                    move || loaded_module,
                    |module_ref| FunctionRef::new(module_ref, CompiledScript::MAIN_INDEX),
                )
                .map(Ok)
        }
    }
}
