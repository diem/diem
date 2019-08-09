// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Cache for modules published on chain.

use crate::{
    code_cache::module_adapter::{ModuleFetcher, NullFetcher},
    gas_meter::GasMeter,
    loaded_data::{
        function::{FunctionRef, FunctionReference},
        loaded_module::LoadedModule,
    },
};
use bytecode_verifier::VerifiedModule;
use std::marker::PhantomData;
use types::language_storage::ModuleId;
use vm::{
    access::ModuleAccess,
    errors::*,
    file_format::{
        FunctionHandleIndex, SignatureToken, StructDefinitionIndex, StructFieldInformation,
        StructHandleIndex,
    },
    views::{FunctionHandleView, StructHandleView},
};
use vm_cache_map::{Arena, CacheRefMap};
use vm_runtime_types::loaded_data::{struct_def::StructDef, types::Type};

#[cfg(test)]
#[path = "../unit_tests/module_cache_tests.rs"]
mod module_cache_tests;

/// Trait that describe a cache for modules. The idea is that this trait will in charge of
/// loading resolving all dependencies of needed module from the storage.
pub trait ModuleCache<'alloc> {
    /// Given a function handle index, resolves that handle into an internal representation of
    /// move function.
    ///
    /// Returns:
    ///
    /// * `Ok(Ok(Some(FunctionRef)))` if such function exists.
    /// * `Ok(Ok(None))` if such function doesn't exists.
    /// * `Ok(Err(...))` or `Err(...)` for a verification issue in a resolved dependency.
    /// * `Err(...)` for a VM invariant violation.
    fn resolve_function_ref(
        &self,
        caller_module: &LoadedModule,
        idx: FunctionHandleIndex,
    ) -> VMResult<Option<FunctionRef<'alloc>>>;

    /// Resolve a StructDefinitionIndex into a StructDef. This process will be recursive so we may
    /// charge gas on each recursive step.
    ///
    /// Returns:
    ///
    /// * `Ok(Ok(Some(StructDef)))` if such struct exists.
    /// * `Ok(Ok(None))` if such function doesn't exists.
    /// * `Ok(Err(...))` for a verification or other issue in a resolved dependency, or out of gas.
    /// * `Err(...)` for a VM invariant violation.
    fn resolve_struct_def(
        &self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        gas_meter: &GasMeter,
    ) -> VMResult<Option<StructDef>>;

    /// Resolve a ModuleId into a LoadedModule if the module has been cached already.
    ///
    /// Returns:
    ///
    /// * `Ok(Ok(Some(LoadedModule)))` if such module exists.
    /// * `Ok(Ok(None))` if such module doesn't exists.
    /// * `Ok(Err(...))` for a verification issue in the module.
    /// * `Err(...)` for a VM invariant violation.
    fn get_loaded_module(&self, id: &ModuleId) -> VMResult<Option<&'alloc LoadedModule>>;

    fn cache_module(&self, module: VerifiedModule);

    /// Recache the list of previously resolved modules. Think of the cache as a generational
    /// cache and we need to move modules across generations.
    fn reclaim_cached_module(&self, v: Vec<LoadedModule>);
}

/// `ModuleCache` is also implemented for references.
impl<'alloc, P> ModuleCache<'alloc> for &P
where
    P: ModuleCache<'alloc>,
{
    fn resolve_function_ref(
        &self,
        caller_module: &LoadedModule,
        idx: FunctionHandleIndex,
    ) -> VMResult<Option<FunctionRef<'alloc>>> {
        (*self).resolve_function_ref(caller_module, idx)
    }

    fn resolve_struct_def(
        &self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        gas_meter: &GasMeter,
    ) -> VMResult<Option<StructDef>> {
        (*self).resolve_struct_def(module, idx, gas_meter)
    }

    fn get_loaded_module(&self, id: &ModuleId) -> VMResult<Option<&'alloc LoadedModule>> {
        (*self).get_loaded_module(id)
    }

    fn cache_module(&self, module: VerifiedModule) {
        (*self).cache_module(module)
    }

    fn reclaim_cached_module(&self, v: Vec<LoadedModule>) {
        (*self).reclaim_cached_module(v)
    }
}

/// Cache for modules that resides in a VM. It is an internally mutable map from module
/// identifier to a reference to loaded module, where the actual module is owned by the Arena
/// allocator so that it will guarantee to outlive the lifetime of the transaction.
pub struct VMModuleCache<'alloc> {
    map: CacheRefMap<'alloc, ModuleId, LoadedModule>,
}

impl<'alloc> VMModuleCache<'alloc> {
    /// In order
    /// to get a cleaner lifetime, the loaded program trait will take an input parameter of Arena
    /// allocator to store so that every allocated element in the loaded program can have the same
    /// lifetime.
    pub fn new(allocator: &'alloc Arena<LoadedModule>) -> Self {
        VMModuleCache {
            map: CacheRefMap::new(allocator),
        }
    }

    /// Resolve a ModuleId into a LoadedModule. If there is a cache miss, try to fetch the module
    /// from the `fetcher` and insert it into the cache if found. If nothing is found, it will
    /// return Ok(None).
    pub fn get_loaded_module_with_fetcher<F: ModuleFetcher>(
        &self,
        id: &ModuleId,
        fetcher: &F,
    ) -> VMRuntimeResult<Option<&'alloc LoadedModule>> {
        // Currently it is still possible for a script to invoke a nonsense module id function.
        // However, once we have the verifier that checks the well-formedness of the all the linked
        // module id, we should get rid of that ok_or case here.
        if let Some(m) = self.map.get(id) {
            return Ok(Some(&*m));
        }
        let module = match fetcher.get_module(id) {
            Some(module) => module,
            None => return Ok(None),
        };

        // Verify the module before using it.
        let module = match VerifiedModule::new(module) {
            Ok(module) => module,
            Err((_, errors)) => {
                return Err(VMRuntimeError {
                    loc: Location::new(),
                    err: VMErrorKind::Verification(
                        errors
                            .into_iter()
                            .map(|error| VerificationStatus::Dependency(id.clone(), error))
                            .collect(),
                    ),
                })
            }
        };

        let loaded_module = LoadedModule::new(module);
        Ok(Some(self.map.or_insert(id.clone(), loaded_module)))
    }

    #[cfg(test)]
    pub fn new_from_module(
        module: VerifiedModule,
        allocator: &'alloc Arena<LoadedModule>,
    ) -> Result<Self, VMInvariantViolation> {
        let module_id = module.self_id();
        let map = CacheRefMap::new(allocator);
        let loaded_module = LoadedModule::new(module);
        map.or_insert(module_id, loaded_module);
        Ok(VMModuleCache { map })
    }

    /// Resolve a FunctionHandleIndex into a FunctionRef in either the cache or the `fetcher`.
    /// An Ok(None) will be returned if no such function is found.
    pub fn resolve_function_ref_with_fetcher<F>(
        &self,
        caller_module: &LoadedModule,
        idx: FunctionHandleIndex,
        fetcher: &F,
    ) -> VMResult<Option<FunctionRef<'alloc>>>
    where
        F: ModuleFetcher,
    {
        let function_handle = caller_module.function_handle_at(idx);
        let callee_name = caller_module.string_at(function_handle.name);
        let callee_module_id = FunctionHandleView::new(caller_module, function_handle).module_id();

        match self.get_loaded_module_with_fetcher(&callee_module_id, fetcher) {
            Ok(Some(callee_module)) => {
                let callee_func_id = callee_module
                    .function_defs_table
                    .get(callee_name)
                    .ok_or(VMInvariantViolation::LinkerError)?;
                Ok(Ok(Some(FunctionRef::new(callee_module, *callee_func_id))))
            }
            Ok(None) => Ok(Ok(None)),
            Err(errors) => Ok(Err(errors)),
        }
    }

    /// Resolve a StructHandle into a StructDef recursively in either the cache or the `fetcher`.
    pub fn resolve_struct_handle_with_fetcher<F: ModuleFetcher>(
        &self,
        module: &LoadedModule,
        idx: StructHandleIndex,
        gas_meter: &GasMeter,
        fetcher: &F,
    ) -> VMResult<Option<StructDef>> {
        let struct_handle = module.struct_handle_at(idx);
        let struct_name = module.string_at(struct_handle.name);
        let struct_def_module_id = StructHandleView::new(module, struct_handle).module_id();
        match self.get_loaded_module_with_fetcher(&struct_def_module_id, fetcher) {
            Ok(Some(module)) => {
                let struct_def_idx = module
                    .struct_defs_table
                    .get(struct_name)
                    .ok_or(VMInvariantViolation::LinkerError)?;
                self.resolve_struct_def_with_fetcher(module, *struct_def_idx, gas_meter, fetcher)
            }
            Ok(None) => Ok(Ok(None)),
            Err(errors) => Ok(Err(errors)),
        }
    }

    /// Resolve a SignatureToken into a Type recursively in either the cache or the `fetcher`.
    pub fn resolve_signature_token_with_fetcher<'txn, F: ModuleFetcher>(
        &'txn self,
        module: &LoadedModule,
        tok: &SignatureToken,
        gas_meter: &GasMeter,
        fetcher: &F,
    ) -> VMResult<Option<Type>> {
        match tok {
            SignatureToken::Bool => Ok(Ok(Some(Type::Bool))),
            SignatureToken::U64 => Ok(Ok(Some(Type::U64))),
            SignatureToken::String => Ok(Ok(Some(Type::String))),
            SignatureToken::ByteArray => Ok(Ok(Some(Type::ByteArray))),
            SignatureToken::Address => Ok(Ok(Some(Type::Address))),
            SignatureToken::TypeParameter(_) => unimplemented!(),
            SignatureToken::Struct(sh_idx, _) => {
                let struct_def =
                    try_runtime!(self
                        .resolve_struct_handle_with_fetcher(module, *sh_idx, gas_meter, fetcher));
                Ok(Ok(struct_def.map(Type::Struct)))
            }
            SignatureToken::Reference(sub_tok) => {
                let inner_ty =
                    try_runtime!(self
                        .resolve_signature_token_with_fetcher(module, sub_tok, gas_meter, fetcher));
                Ok(Ok(inner_ty.map(|t| Type::Reference(Box::new(t)))))
            }
            SignatureToken::MutableReference(sub_tok) => {
                let inner_ty =
                    try_runtime!(self
                        .resolve_signature_token_with_fetcher(module, sub_tok, gas_meter, fetcher));
                Ok(Ok(inner_ty.map(|t| Type::MutableReference(Box::new(t)))))
            }
        }
    }

    /// Resolve a StructDefinition into a StructDef recursively in either the cache or the
    /// `fetcher`.
    pub fn resolve_struct_def_with_fetcher<'txn, F: ModuleFetcher>(
        &'txn self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        gas_meter: &GasMeter,
        fetcher: &F,
    ) -> VMResult<Option<StructDef>> {
        if let Some(def) = module.cached_struct_def_at(idx) {
            return Ok(Ok(Some(def)));
        }
        let def = {
            let struct_def = module.struct_def_at(idx);
            match &struct_def.field_information {
                // TODO we might want a more informative error here
                StructFieldInformation::Native => return Err(VMInvariantViolation::LinkerError),
                StructFieldInformation::Declared {
                    field_count,
                    fields,
                } => {
                    let mut field_types = vec![];
                    for field in module.field_def_range(*field_count, *fields) {
                        let ty = try_runtime!(self.resolve_signature_token_with_fetcher(
                            module,
                            &module.type_signature_at(field.signature).0,
                            gas_meter,
                            fetcher
                        ));
                        if let Some(t) = ty {
                            // `field_types` is initally empty, a single element is pushed
                            // per loop iteration and the number of iterations is bound to
                            // the max size of `module.field_def_range()`.
                            // MIRAI cannot currently check this bound in terms of
                            // `field_count`.
                            assume!(field_types.len() < usize::max_value());
                            field_types.push(t);
                        } else {
                            return Ok(Ok(None));
                        }
                    }
                    StructDef::new(field_types)
                }
            }
        };
        // If multiple writers write to def at the same time, the last one will win. It's possible
        // to have multiple copies of a struct def floating around, but that probably isn't going
        // to be a big deal.
        module.cache_struct_def(idx, def.clone());
        Ok(Ok(Some(def)))
    }
}

impl<'alloc> ModuleCache<'alloc> for VMModuleCache<'alloc> {
    fn resolve_function_ref(
        &self,
        caller_module: &LoadedModule,
        idx: FunctionHandleIndex,
    ) -> VMResult<Option<FunctionRef<'alloc>>> {
        self.resolve_function_ref_with_fetcher(caller_module, idx, &NullFetcher())
    }

    fn resolve_struct_def(
        &self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        gas_meter: &GasMeter,
    ) -> VMResult<Option<StructDef>> {
        self.resolve_struct_def_with_fetcher(module, idx, gas_meter, &NullFetcher())
    }

    fn get_loaded_module(&self, id: &ModuleId) -> VMResult<Option<&'alloc LoadedModule>> {
        // Currently it is still possible for a script to invoke a nonsense module id function.
        // However, once we have the verifier that checks the well-formedness of the all the linked
        // module id, we should get rid of that ok_or case here.
        Ok(Ok(self.map.get(id)))
    }

    fn cache_module(&self, module: VerifiedModule) {
        let module_id = module.self_id();
        // TODO: Check ModuleId duplication in statedb
        let loaded_module = LoadedModule::new(module);
        self.map.or_insert(module_id, loaded_module);
    }

    fn reclaim_cached_module(&self, v: Vec<LoadedModule>) {
        for m in v.into_iter() {
            let module_id = m.self_id();
            self.map.or_insert(module_id, m);
        }
    }
}

/// A cache for all modules stored on chain. `vm_cache` holds the local cached modules whereas
/// `storage` should implement trait ModuleFetcher that can fetch the modules that aren't in the
/// cache yet. In production, it will usually provide a connection to the StateStore client to fetch
/// the needed data. `alloc` is the lifetime for the entire VM and `blk` is the lifetime for the
/// current block we are executing.
pub struct BlockModuleCache<'alloc, 'blk, F>
where
    'alloc: 'blk,
    F: ModuleFetcher,
{
    vm_cache: &'blk VMModuleCache<'alloc>,
    storage: F,
}

impl<'alloc, 'blk, F> BlockModuleCache<'alloc, 'blk, F>
where
    'alloc: 'blk,
    F: ModuleFetcher,
{
    pub fn new(vm_cache: &'blk VMModuleCache<'alloc>, module_fetcher: F) -> Self {
        BlockModuleCache {
            vm_cache,
            storage: module_fetcher,
        }
    }
}

impl<'alloc, 'blk, F: ModuleFetcher> ModuleCache<'alloc> for BlockModuleCache<'alloc, 'blk, F> {
    fn resolve_function_ref(
        &self,
        caller_module: &LoadedModule,
        idx: FunctionHandleIndex,
    ) -> VMResult<Option<FunctionRef<'alloc>>> {
        self.vm_cache
            .resolve_function_ref_with_fetcher(caller_module, idx, &self.storage)
    }

    fn resolve_struct_def(
        &self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        gas_meter: &GasMeter,
    ) -> VMResult<Option<StructDef>> {
        self.vm_cache
            .resolve_struct_def_with_fetcher(module, idx, gas_meter, &self.storage)
    }

    fn get_loaded_module(&self, id: &ModuleId) -> VMResult<Option<&'alloc LoadedModule>> {
        Ok(self
            .vm_cache
            .get_loaded_module_with_fetcher(id, &self.storage))
    }

    fn cache_module(&self, module: VerifiedModule) {
        self.vm_cache.cache_module(module)
    }

    fn reclaim_cached_module(&self, v: Vec<LoadedModule>) {
        self.vm_cache.reclaim_cached_module(v)
    }
}

/// A temporary cache for module published by a single transaction. This cache allows the
/// transaction script to refer to either those newly published modules in `local_cache` or those
/// existing on chain modules in `block_cache`. VM can choose to discard those newly published
/// modules if there is an error during execution.
pub struct TransactionModuleCache<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    block_cache: P,
    local_cache: VMModuleCache<'txn>,

    phantom: PhantomData<&'alloc ()>,
}

impl<'alloc, 'txn, P> TransactionModuleCache<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    pub fn new(block_cache: P, allocator: &'txn Arena<LoadedModule>) -> Self {
        TransactionModuleCache {
            block_cache,
            local_cache: VMModuleCache::new(allocator),
            phantom: PhantomData,
        }
    }
}

impl<'alloc, 'txn, P> ModuleCache<'txn> for TransactionModuleCache<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    fn resolve_function_ref(
        &self,
        caller_module: &LoadedModule,
        idx: FunctionHandleIndex,
    ) -> VMResult<Option<FunctionRef<'txn>>> {
        if let Some(f) = try_runtime!(self.local_cache.resolve_function_ref(caller_module, idx)) {
            Ok(Ok(Some(f)))
        } else {
            self.block_cache.resolve_function_ref(caller_module, idx)
        }
    }

    fn resolve_struct_def(
        &self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        gas_meter: &GasMeter,
    ) -> VMResult<Option<StructDef>> {
        if let Some(f) = try_runtime!(self.local_cache.resolve_struct_def(module, idx, gas_meter)) {
            Ok(Ok(Some(f)))
        } else {
            self.block_cache.resolve_struct_def(module, idx, gas_meter)
        }
    }

    fn get_loaded_module(&self, id: &ModuleId) -> VMResult<Option<&'txn LoadedModule>> {
        if let Some(m) = try_runtime!(self.local_cache.get_loaded_module(id)) {
            Ok(Ok(Some(m)))
        } else {
            self.block_cache.get_loaded_module(id)
        }
    }

    fn cache_module(&self, module: VerifiedModule) {
        self.local_cache.cache_module(module)
    }

    fn reclaim_cached_module(&self, _v: Vec<LoadedModule>) {
        panic!("reclaim_cached_module should never be called on TransactionModuleCache");
    }
}
