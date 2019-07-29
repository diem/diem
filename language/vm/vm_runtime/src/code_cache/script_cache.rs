// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Cache for commonly executed scripts

use crate::loaded_data::{
    function::{FunctionRef, FunctionReference},
    loaded_module::LoadedModule,
};
use bytecode_verifier::VerifiedScript;
use logger::prelude::*;
use sha3::{Digest, Sha3_256};
use types::transaction::SCRIPT_HASH_LENGTH;
use vm::{
    errors::{Location, VMErrorKind, VMResult, VMRuntimeError, VerificationStatus},
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
        let hash: [u8; SCRIPT_HASH_LENGTH] = Sha3_256::digest(raw_bytes).into();
        // XXX We may want to put in some negative caching for scripts that fail verification.
        if let Some(f) = self.map.get(&hash) {
            trace!("[VM] Script cache hit");
            Ok(Ok(f))
        } else {
            trace!("[VM] Script cache miss");
            let script = try_runtime!(Self::deserialize_and_verify(raw_bytes));
            let fake_module = script.into_module();
            let loaded_module = LoadedModule::new(fake_module);
            Ok(Ok(self.map.or_insert_with_transform(
                hash,
                move || loaded_module,
                |module_ref| FunctionRef::new(module_ref, CompiledScript::MAIN_INDEX),
            )))
        }
    }

    fn deserialize_and_verify(raw_bytes: &[u8]) -> VMResult<VerifiedScript> {
        let script = match CompiledScript::deserialize(raw_bytes) {
            Ok(script) => script,
            Err(err) => {
                warn!("[VM] deserializer returned error for script: {:?}", err);
                return Ok(Err(VMRuntimeError {
                    loc: Location::default(),
                    err: VMErrorKind::CodeDeserializerError(err),
                }));
            }
        };

        match VerifiedScript::new(script) {
            Ok(script) => Ok(Ok(script)),
            Err((_, errs)) => {
                warn!(
                    "[VM] bytecode verifier returned errors for script: {:?}",
                    errs
                );
                let statuses = errs.into_iter().map(VerificationStatus::Script).collect();
                Ok(Err(VMRuntimeError {
                    loc: Location::default(),
                    err: VMErrorKind::Verification(statuses),
                }))
            }
        }
    }
}
