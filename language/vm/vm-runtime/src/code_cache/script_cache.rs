// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Cache for commonly executed scripts

use crate::loaded_data::{
    function::{FunctionRef, FunctionReference},
    loaded_module::LoadedModule,
};
use bytecode_verifier::VerifiedScript;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::{
    transaction::SCRIPT_HASH_LENGTH,
    vm_error::{StatusCode, VMStatus},
};
use vm::{
    errors::{vm_error, Location, VMResult},
    file_format::CompiledScript,
};
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

    /// Compiles, verifies, caches and resolves `raw_bytes` into a `FunctionRef` that can be
    /// executed.
    pub fn cache_script(&self, raw_bytes: &[u8]) -> VMResult<FunctionRef<'alloc>> {
        let hash_value = HashValue::from_sha3_256(raw_bytes);

        // XXX We may want to put in some negative caching for scripts that fail verification.
        if let Some(f) = self.map.get(hash_value.as_ref()) {
            trace!("[VM] Script cache hit");
            Ok(f)
        } else {
            trace!("[VM] Script cache miss");
            let script = Self::deserialize_and_verify(raw_bytes)?;
            let fake_module = script.into_module();
            let loaded_module = LoadedModule::new(fake_module);
            Ok(self.map.or_insert_with_transform(
                *hash_value.as_ref(),
                move || loaded_module,
                |module_ref| FunctionRef::new(module_ref, CompiledScript::MAIN_INDEX),
            ))
        }
    }

    fn deserialize_and_verify(raw_bytes: &[u8]) -> VMResult<VerifiedScript> {
        let script = match CompiledScript::deserialize(raw_bytes) {
            Ok(script) => script,
            Err(err) => {
                warn!("[VM] deserializer returned error for script: {:?}", err);
                let error = vm_error(Location::default(), StatusCode::CODE_DESERIALIZATION_ERROR)
                    .append(err);
                return Err(error);
            }
        };

        match VerifiedScript::new(script) {
            Ok(script) => Ok(script),
            Err((_, mut errs)) => {
                warn!(
                    "[VM] bytecode verifier returned errors for script: {:?}",
                    errs
                );
                // If there are errors there should be at least one otherwise there's an internal
                // error in the verifier. We only give back the first error. If the user wants to
                // debug things, they can do that offline.
                let error = if errs.is_empty() {
                    VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION)
                } else {
                    errs.remove(0)
                };

                Err(error)
            }
        }
    }
}
