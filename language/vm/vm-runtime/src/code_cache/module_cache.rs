// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Cache for modules published on chain.

use crate::{
    code_cache::module_adapter::{ModuleFetcher, NullFetcher},
    execution_context::InterpreterContext,
    loaded_data::{
        function::{FunctionRef, FunctionReference},
        loaded_module::LoadedModule,
    },
};
use bytecode_verifier::VerifiedModule;
use libra_types::{
    language_storage::ModuleId,
    vm_error::{StatusCode, VMStatus},
};
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
use vm_runtime_types::{
    loaded_data::{struct_def::StructDef, types::Type},
    native_structs::dispatch::resolve_native_struct,
    type_context::TypeContext,
};

#[cfg(test)]
use crate::code_cache::module_adapter::FakeFetcher;

/// Trait that describe a cache for modules. The idea is that this trait will in charge of
/// loading resolving all dependencies of needed module from the storage.
pub trait ModuleCache<'alloc> {
    /// Given a function handle index, resolves that handle into an internal representation of
    /// move function.
    ///
    /// Returns:
    ///
    /// * `Ok(FunctionRef)` if such function exists.
    /// * `Err(...)` for a verification issue in a resolved dependency, VM invariant violation, or
    ///   function not found.
    fn resolve_function_ref(
        &self,
        caller_module: &LoadedModule,
        idx: FunctionHandleIndex,
    ) -> VMResult<FunctionRef<'alloc>>;

    /// Resolve a StructDefinitionIndex into a StructDef. This process will be recursive so we may
    /// charge gas on each recursive step.
    ///
    /// Returns:
    ///
    /// * `Ok(StructDef)` if such struct exists.
    /// * `Err(...)` for a verification or other issue in a resolved dependency, out of gas, or for
    ///   a VM invariant violation.
    fn resolve_struct_def(
        &self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        context: &mut dyn InterpreterContext,
    ) -> VMResult<StructDef>;

    /// Resolve a ModuleId into a LoadedModule if the module has been cached already.
    ///
    /// Returns:
    ///
    /// * `Ok(LoadedModule)` if such module exists.
    /// * `Err(...)` for a verification issue in the module or for a VM invariant violation.
    fn get_loaded_module(&self, id: &ModuleId) -> VMResult<&'alloc LoadedModule>;

    fn cache_module(&self, module: VerifiedModule);
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
    ) -> VMResult<FunctionRef<'alloc>> {
        (*self).resolve_function_ref(caller_module, idx)
    }

    fn resolve_struct_def(
        &self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        context: &mut dyn InterpreterContext,
    ) -> VMResult<StructDef> {
        (*self).resolve_struct_def(module, idx, context)
    }

    fn get_loaded_module(&self, id: &ModuleId) -> VMResult<&'alloc LoadedModule> {
        (*self).get_loaded_module(id)
    }

    fn cache_module(&self, module: VerifiedModule) {
        (*self).cache_module(module)
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
    ) -> VMResult<&'alloc LoadedModule> {
        // Currently it is still possible for a script to invoke a nonsense module id function.
        // However, once we have the verifier that checks the well-formedness of the all the linked
        // module id, we should get rid of that ok_or_else case here.
        if let Some(m) = self.map.get(id) {
            return Ok(&*m);
        }
        // TODO: Add a better error message for what failed to be loaded.
        let module = match fetcher.get_module(id) {
            Some(module) => module,
            None => return Err(VMStatus::new(StatusCode::LINKER_ERROR)),
        };

        // Verify the module before using it.
        let module = match VerifiedModule::new(module) {
            Ok(module) => module,
            Err((_, mut errors)) => {
                // If there are errors there should be at least one otherwise there's an internal
                // error in the verifier. We only give back the first error. If the user wants to
                // debug things, they can do that offline.
                let error = if errors.is_empty() {
                    VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION)
                } else {
                    errors.remove(0)
                };
                return Err(error);
            }
        };

        let loaded_module = LoadedModule::new(module);
        Ok(self.map.or_insert(id.clone(), loaded_module))
    }

    #[cfg(test)]
    pub fn new_from_module(
        module: VerifiedModule,
        allocator: &'alloc Arena<LoadedModule>,
    ) -> VMResult<Self> {
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
    ) -> VMResult<FunctionRef<'alloc>>
    where
        F: ModuleFetcher,
    {
        let function_handle = caller_module.function_handle_at(idx);
        let callee_name = caller_module.identifier_at(function_handle.name);
        let callee_module_id = FunctionHandleView::new(caller_module, function_handle).module_id();

        match self.get_loaded_module_with_fetcher(&callee_module_id, fetcher) {
            Ok(callee_module) => {
                let callee_func_id = callee_module
                    .function_defs_table
                    .get(callee_name)
                    .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?;
                Ok(FunctionRef::new(callee_module, *callee_func_id))
            }
            Err(errors) => Err(errors),
        }
    }

    /// Resolve a StructHandle into a StructDef recursively in either the cache or the `fetcher`.
    pub fn resolve_struct_handle_with_fetcher<F: ModuleFetcher>(
        &self,
        module: &LoadedModule,
        idx: StructHandleIndex,
        context: &mut dyn InterpreterContext,
        fetcher: &F,
    ) -> VMResult<StructDef> {
        let struct_handle = module.struct_handle_at(idx);
        let struct_name = module.identifier_at(struct_handle.name);
        let struct_def_module_id = StructHandleView::new(module, struct_handle).module_id();
        match self.get_loaded_module_with_fetcher(&struct_def_module_id, fetcher) {
            Ok(module) => {
                let struct_def_idx = module.get_struct_def_index(struct_name)?;
                self.resolve_struct_def_with_fetcher(module, *struct_def_idx, context, fetcher)
            }
            Err(errors) => Err(errors),
        }
    }

    /// Resolve a SignatureToken into a Type recursively in either the cache or the `fetcher`.
    pub fn resolve_signature_token_with_fetcher<'txn, F: ModuleFetcher>(
        &'txn self,
        module: &LoadedModule,
        tok: &SignatureToken,
        type_context: &TypeContext,
        context: &mut dyn InterpreterContext,
        fetcher: &F,
    ) -> VMResult<Type> {
        match tok {
            SignatureToken::Bool => Ok(Type::Bool),
            SignatureToken::U64 => Ok(Type::U64),
            SignatureToken::String => Ok(Type::String),
            SignatureToken::ByteArray => Ok(Type::ByteArray),
            SignatureToken::Address => Ok(Type::Address),
            SignatureToken::TypeParameter(idx) => Ok(type_context.get_type(*idx)?),
            SignatureToken::Struct(sh_idx, tys) => {
                let ctx = {
                    let mut ctx = vec![];
                    for ty in tys.iter() {
                        let resolved_type = self.resolve_signature_token_with_fetcher(
                            module,
                            ty,
                            type_context,
                            context,
                            fetcher,
                        )?;
                        ctx.push(resolved_type);
                    }
                    TypeContext::new(ctx)
                };
                let struct_def = ctx.subst_struct_def(
                    &self.resolve_struct_handle_with_fetcher(module, *sh_idx, context, fetcher)?,
                )?;
                Ok(Type::Struct(struct_def))
            }
            SignatureToken::Reference(sub_tok) => {
                let inner_ty = self.resolve_signature_token_with_fetcher(
                    module,
                    sub_tok,
                    type_context,
                    context,
                    fetcher,
                )?;
                Ok(Type::Reference(Box::new(inner_ty)))
            }
            SignatureToken::MutableReference(sub_tok) => {
                let inner_ty = self.resolve_signature_token_with_fetcher(
                    module,
                    sub_tok,
                    type_context,
                    context,
                    fetcher,
                )?;
                Ok(Type::MutableReference(Box::new(inner_ty)))
            }
        }
    }

    /// Resolve a StructDefinition into a StructDef recursively in either the cache or the
    /// `fetcher`.
    pub fn resolve_struct_def_with_fetcher<'txn, F: ModuleFetcher>(
        &'txn self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        context: &mut dyn InterpreterContext,
        fetcher: &F,
    ) -> VMResult<StructDef> {
        if let Some(def) = module.cached_struct_def_at(idx) {
            return Ok(def);
        }
        let def = {
            let struct_def = module.struct_def_at(idx);
            let struct_handle = module.struct_handle_at(struct_def.struct_handle);
            let type_context =
                TypeContext::identity_mapping(struct_handle.type_formals.len() as u16);
            match &struct_def.field_information {
                // TODO we might want a more informative error here
                StructFieldInformation::Native => {
                    let struct_name = module.identifier_at(struct_handle.name);
                    let struct_def_module_id =
                        StructHandleView::new(module, struct_handle).module_id();
                    StructDef::Native(
                        resolve_native_struct(&struct_def_module_id, struct_name)
                            .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))?
                            .struct_type
                            .clone(),
                    )
                }
                StructFieldInformation::Declared {
                    field_count,
                    fields,
                } => {
                    let mut field_types = vec![];
                    for field in module.field_def_range(*field_count, *fields) {
                        let ty = self.resolve_signature_token_with_fetcher(
                            module,
                            &module.type_signature_at(field.signature).0,
                            &type_context,
                            context,
                            fetcher,
                        )?;
                        // `field_types` is initally empty, a single element is pushed
                        // per loop iteration and the number of iterations is bound to
                        // the max size of `module.field_def_range()`.
                        // MIRAI cannot currently check this bound in terms of
                        // `field_count`.
                        assume!(field_types.len() < usize::max_value());
                        field_types.push(ty);
                    }
                    StructDef::new(field_types)
                }
            }
        };
        // If multiple writers write to def at the same time, the last one will win. It's possible
        // to have multiple copies of a struct def floating around, but that probably isn't going
        // to be a big deal.
        module.cache_struct_def(idx, def.clone());
        Ok(def)
    }
}

impl<'alloc> ModuleCache<'alloc> for VMModuleCache<'alloc> {
    fn resolve_function_ref(
        &self,
        caller_module: &LoadedModule,
        idx: FunctionHandleIndex,
    ) -> VMResult<FunctionRef<'alloc>> {
        self.resolve_function_ref_with_fetcher(caller_module, idx, &NullFetcher())
    }

    fn resolve_struct_def(
        &self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        context: &mut dyn InterpreterContext,
    ) -> VMResult<StructDef> {
        self.resolve_struct_def_with_fetcher(module, idx, context, &NullFetcher())
    }

    fn get_loaded_module(&self, id: &ModuleId) -> VMResult<&'alloc LoadedModule> {
        self.map
            .get(id)
            .ok_or_else(|| VMStatus::new(StatusCode::LINKER_ERROR))
    }

    fn cache_module(&self, module: VerifiedModule) {
        let module_id = module.self_id();
        // TODO: Check ModuleId duplication in statedb
        let loaded_module = LoadedModule::new(module);
        self.map.or_insert(module_id, loaded_module);
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

#[cfg(test)]
impl<'alloc, 'blk> BlockModuleCache<'alloc, 'blk, FakeFetcher> {
    pub(crate) fn clear(&mut self) {
        self.storage.clear();
    }
}

impl<'alloc, 'blk, F: ModuleFetcher> ModuleCache<'alloc> for BlockModuleCache<'alloc, 'blk, F> {
    fn resolve_function_ref(
        &self,
        caller_module: &LoadedModule,
        idx: FunctionHandleIndex,
    ) -> VMResult<FunctionRef<'alloc>> {
        self.vm_cache
            .resolve_function_ref_with_fetcher(caller_module, idx, &self.storage)
    }

    fn resolve_struct_def(
        &self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        context: &mut dyn InterpreterContext,
    ) -> VMResult<StructDef> {
        self.vm_cache
            .resolve_struct_def_with_fetcher(module, idx, context, &self.storage)
    }

    fn get_loaded_module(&self, id: &ModuleId) -> VMResult<&'alloc LoadedModule> {
        self.vm_cache
            .get_loaded_module_with_fetcher(id, &self.storage)
    }

    fn cache_module(&self, module: VerifiedModule) {
        self.vm_cache.cache_module(module)
    }
}
