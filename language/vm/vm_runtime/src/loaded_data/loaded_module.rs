// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Loaded representation for Move modules.

use crate::loaded_data::{function::FunctionDef, struct_def::StructDef};
use bytecode_verifier::VerifiedModule;
use std::{collections::HashMap, sync::RwLock};
use vm::{
    access::ModuleAccess,
    errors::VMInvariantViolation,
    file_format::{
        CompiledModule, FieldDefinitionIndex, FunctionDefinitionIndex, StructDefinitionIndex,
        StructFieldInformation, TableIndex,
    },
    internals::ModuleIndex,
};

/// Defines a loaded module in the memory. Currently we just store module itself with a bunch of
/// reverse mapping that allows querying definition of struct/function by name.
#[derive(Debug, Eq, PartialEq)]
pub struct LoadedModule {
    module: VerifiedModule,
    #[allow(dead_code)]
    pub struct_defs_table: HashMap<String, StructDefinitionIndex>,
    #[allow(dead_code)]
    pub field_defs_table: HashMap<String, FieldDefinitionIndex>,

    pub function_defs_table: HashMap<String, FunctionDefinitionIndex>,

    pub function_defs: Vec<FunctionDef>,

    pub field_offsets: Vec<TableIndex>,

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
    struct_defs: Vec<RwLock<Option<StructDef>>>,
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
        let mut field_defs_table = HashMap::new();
        let mut function_defs_table = HashMap::new();
        let mut function_defs = vec![];
        let struct_defs = module
            .struct_defs()
            .iter()
            .map(|_| RwLock::new(None))
            .collect();
        let cache = LoadedModuleCache { struct_defs };

        let mut field_offsets: Vec<TableIndex> = module.field_defs().iter().map(|_| 0).collect();

        for (idx, struct_def) in module.struct_defs().iter().enumerate() {
            let name = module
                .string_at(module.struct_handle_at(struct_def.struct_handle).name)
                .to_string();
            let sd_idx = StructDefinitionIndex::new(idx as TableIndex);
            struct_defs_table.insert(name, sd_idx);

            if let StructFieldInformation::Declared {
                field_count,
                fields,
            } = &struct_def.field_information
            {
                for i in 0..*field_count {
                    field_offsets[fields.into_index() + i as usize] = i;
                }
            }
        }
        for (idx, field_def) in module.field_defs().iter().enumerate() {
            let name = module.string_at(field_def.name).to_string();
            let fd_idx = FieldDefinitionIndex::new(idx as TableIndex);
            field_defs_table.insert(name, fd_idx);
        }
        for (idx, function_def) in module.function_defs().iter().enumerate() {
            let name = module
                .string_at(module.function_handle_at(function_def.function).name)
                .to_string();
            let fd_idx = FunctionDefinitionIndex::new(idx as TableIndex);
            function_defs_table.insert(name, fd_idx);
            function_defs.push(FunctionDef::new(&module, fd_idx));
        }
        LoadedModule {
            module,
            struct_defs_table,
            field_defs_table,
            function_defs_table,
            function_defs,
            field_offsets,
            cache,
        }
    }

    /// Return a cached copy of the struct def at this index, if available.
    pub fn cached_struct_def_at(&self, idx: StructDefinitionIndex) -> Option<StructDef> {
        let cached = self.cache.struct_defs[idx.into_index()]
            .read()
            .expect("lock poisoned");
        cached.clone()
    }

    /// Cache this struct def at this location.
    pub fn cache_struct_def(&self, idx: StructDefinitionIndex, def: StructDef) {
        let mut cached = self.cache.struct_defs[idx.into_index()]
            .write()
            .expect("lock poisoned");
        // XXX If multiple writers call this at the same time, the last write wins. Is this
        // desirable?
        cached.replace(def);
    }

    pub fn get_field_offset(
        &self,
        idx: FieldDefinitionIndex,
    ) -> Result<TableIndex, VMInvariantViolation> {
        self.field_offsets
            .get(idx.into_index())
            .cloned()
            .ok_or(VMInvariantViolation::LinkerError)
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
