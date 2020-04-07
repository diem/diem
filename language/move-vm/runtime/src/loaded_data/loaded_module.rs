// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for Move modules.

use crate::loaded_data::function::FunctionDef;
use bytecode_verifier::VerifiedModule;
use libra_types::vm_error::{StatusCode, VMStatus};
use move_core_types::identifier::{IdentStr, Identifier};
use move_vm_types::loaded_data::types::StructType;
use std::{collections::HashMap, sync::RwLock};
use vm::{
    access::ModuleAccess,
    errors::VMResult,
    file_format::{CompiledModule, FunctionDefinitionIndex, StructDefinitionIndex, TableIndex},
    internals::ModuleIndex,
};

/// Defines a loaded module in the memory. Currently we just store module itself with a bunch of
/// reverse mapping that allows querying definition of struct/function by name.
#[derive(Debug, Eq, PartialEq)]
pub struct LoadedModule {
    module: VerifiedModule,
    pub struct_defs_table: HashMap<Identifier, StructDefinitionIndex>,
    pub function_defs_table: HashMap<Identifier, FunctionDefinitionIndex>,
    pub function_defs: Vec<FunctionDef>,
    cache: LoadedModuleCache,
}

impl ModuleAccess for LoadedModule {
    fn as_module(&self) -> &CompiledModule {
        &self.module.as_inner()
    }
}

#[derive(Debug)]
struct LoadedModuleCache {
    // TODO: this can probably be made lock-free by using AtomicPtr or the "atom" crate. Consider
    // doing so in the future.
    struct_defs: Vec<RwLock<Option<StructType>>>,
}

impl PartialEq for LoadedModuleCache {
    fn eq(&self, _other: &Self) -> bool {
        // This is a cache so ignore equality checks.
        true
    }
}

impl Eq for LoadedModuleCache {}

impl LoadedModule {
    pub fn new(module: VerifiedModule) -> Self {
        let mut struct_defs_table = HashMap::new();
        let mut function_defs_table = HashMap::new();
        let mut function_defs = vec![];

        let struct_defs = module
            .struct_defs()
            .iter()
            .map(|_| RwLock::new(None))
            .collect();
        let cache = LoadedModuleCache { struct_defs };

        for (idx, struct_def) in module.struct_defs().iter().enumerate() {
            let name = module
                .identifier_at(module.struct_handle_at(struct_def.struct_handle).name)
                .into();
            let sd_idx = StructDefinitionIndex::new(idx as TableIndex);
            struct_defs_table.insert(name, sd_idx);
        }

        for (idx, function_def) in module.function_defs().iter().enumerate() {
            let name = module
                .identifier_at(module.function_handle_at(function_def.function).name)
                .into();
            let fd_idx = FunctionDefinitionIndex::new(idx as TableIndex);
            function_defs_table.insert(name, fd_idx);
            // `function_defs` is initally empty, a single element is pushed per loop iteration and
            // the number of iterations is bound to the max size of `module.function_defs()`
            // MIRAI currently cannot work with a bound based on the length of
            // `module.function_defs()`.
            assume!(function_defs.len() < usize::max_value());
            function_defs.push(FunctionDef::new(&module, fd_idx));
        }
        LoadedModule {
            module,
            struct_defs_table,
            function_defs_table,
            function_defs,
            cache,
        }
    }

    /// Return a cached copy of the struct def at this index, if available.
    pub fn cached_struct_def_at(&self, idx: StructDefinitionIndex) -> Option<StructType> {
        precondition!(idx.into_index() < self.cache.struct_defs.len());
        let cached = self.cache.struct_defs[idx.into_index()]
            .read()
            .expect("lock poisoned");
        cached.clone()
    }

    /// Cache this struct def at this location.
    pub fn cache_struct_def(&self, idx: StructDefinitionIndex, ty: StructType) {
        precondition!(idx.into_index() < self.cache.struct_defs.len());
        let mut cached = self.cache.struct_defs[idx.into_index()]
            .write()
            .expect("lock poisoned");
        // XXX If multiple writers call this at the same time, the last write wins. Is this
        // desirable?
        cached.replace(ty);
    }

    pub fn get_struct_def_index(&self, struct_name: &IdentStr) -> VMResult<&StructDefinitionIndex> {
        let result = self
            .struct_defs_table
            .get(struct_name)
            .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR));
        assumed_postcondition!(match result {
            Ok(idx) => idx.into_index() < self.cache.struct_defs.len(),
            Err(..) => true,
        }); // invariant
        result
    }
}

// Compile-time test to ensure that this struct stays thread-safe.
#[test]
fn assert_thread_safe() {
    fn assert_send<T: Send>() {};
    fn assert_sync<T: Sync>() {};

    assert_send::<LoadedModule>();
    assert_sync::<LoadedModule>();
}
