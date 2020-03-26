// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
//! Cache for modules published on chain.

use crate::{
    interpreter_context::InterpreterContext,
    loaded_data::{
        function::{FunctionRef, FunctionReference},
        loaded_module::LoadedModule,
    },
};
use bytecode_verifier::VerifiedModule;
use libra_logger::prelude::*;
use libra_types::{
    language_storage::ModuleId,
    vm_error::{StatusCode, VMStatus},
};
use move_vm_cache::{Arena, CacheRefMap};
use move_vm_types::loaded_data::types::{StructType, Type};
use vm::{
    access::ModuleAccess,
    errors::*,
    file_format::{
        FunctionHandleIndex, SignatureToken, StructDefinitionIndex, StructFieldInformation,
        StructHandleIndex,
    },
    views::{FunctionHandleView, StructHandleView},
    CompiledModule,
};

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

    /// Given a function handle index, resolves that handle into an internal representation of
    /// move function.
    ///
    /// Returns:
    ///
    /// * `Ok(FunctionRef)` if such function exists.
    /// * `Err(...)` for a verification issue in a resolved dependency, VM invariant violation, or
    ///   function not found.
    pub fn resolve_function_ref(
        &self,
        caller_module: &LoadedModule,
        idx: FunctionHandleIndex,
        data_view: &dyn InterpreterContext,
    ) -> VMResult<FunctionRef<'alloc>> {
        let function_handle = caller_module.function_handle_at(idx);
        let callee_name = caller_module.identifier_at(function_handle.name);
        let callee_module_id = FunctionHandleView::new(caller_module, function_handle).module_id();

        match self.get_loaded_module(&callee_module_id, data_view) {
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

    /// Loads a `StructDefinition`, constructs and caches the corresponding `StructType`.
    /// Note this does not perform type substitution for the type parameters.
    ///
    /// This process will be recursive so we may charge gas on each recursive step.
    ///
    /// Returns:
    ///
    /// * `Ok(StructType)` if such struct exists.
    /// * `Err(...)` for a verification or other issue in a resolved dependency, out of gas, or for
    ///   a VM invariant violation.
    fn load_struct_def(
        &self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        data_view: &dyn InterpreterContext,
    ) -> VMResult<StructType> {
        if let Some(ty) = module.cached_struct_def_at(idx) {
            return Ok(ty);
        }

        let struct_def = module.struct_def_at(idx);
        let struct_handle = module.struct_handle_at(struct_def.struct_handle);
        let struct_name = module.identifier_at(struct_handle.name);

        let (field_count, fields) = match &struct_def.field_information {
            StructFieldInformation::Native => unreachable!("native structs have been removed"),
            StructFieldInformation::Declared {
                field_count,
                fields,
            } => (*field_count, *fields),
        };

        let ty_args: Vec<_> = (0..struct_handle.type_formals.len())
            .map(|idx| Type::TyParam(idx as usize))
            .collect();

        let mut field_tys = vec![];
        for field in module.field_def_range(field_count, fields) {
            let ty = self.resolve_signature_token(
                module,
                &module.type_signature_at(field.signature).0,
                &ty_args,
                data_view,
            )?;
            // `field_types` is initally empty, a single element is pushed
            // per loop iteration and the number of iterations is bound to
            // the max size of `module.field_def_range()`.
            // MIRAI cannot currently check this bound in terms of
            // `field_count`.
            assume!(field_tys.len() < usize::max_value());
            field_tys.push(ty);
        }

        let struct_ty = StructType {
            address: *module.address(),
            module: module.name().to_owned(),
            name: struct_name.to_owned(),
            is_resource: struct_handle.is_nominal_resource,
            ty_args,
            layout: field_tys,
        };

        // If multiple writers write to def at the same time, the last one will win. It's possible
        // to have multiple copies of a struct def floating around, but that probably isn't going
        // to be a big deal.
        module.cache_struct_def(idx, struct_ty.clone());
        Ok(struct_ty)
    }

    /// Resolve a StructDefinitionIndex into a StructType.
    pub fn resolve_struct_def(
        &self,
        module: &LoadedModule,
        idx: StructDefinitionIndex,
        ty_args: &[Type],
        data_view: &dyn InterpreterContext,
    ) -> VMResult<StructType> {
        let struct_ty = self.load_struct_def(module, idx, data_view)?;
        if ty_args.len() != struct_ty.ty_args.len() {
            return Err(
                VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(
                    "number of type args does not match that of parameters".to_string(),
                ),
            );
        }
        if ty_args.is_empty() {
            Ok(struct_ty)
        } else {
            struct_ty.subst(ty_args)
        }
    }

    /// Resolve a ModuleId into a LoadedModule if the module has been cached already.
    ///
    /// Returns:
    ///
    /// * `Ok(LoadedModule)` if such module exists.
    /// * `Err(...)` for a verification issue in the module or for a VM invariant violation.
    pub fn get_loaded_module(
        &self,
        id: &ModuleId,
        data_view: &dyn InterpreterContext,
    ) -> VMResult<&'alloc LoadedModule> {
        if let Some(m) = self.map.get(id) {
            return Ok(&*m);
        }
        let module = load_and_verify_module_id(id, data_view)?;
        let loaded_module = LoadedModule::new(module);
        Ok(self.map.or_insert(id.clone(), loaded_module))
    }

    pub fn cache_module(&self, module: VerifiedModule) {
        let module_id = module.self_id();
        // TODO: Check ModuleId duplication in statedb
        let loaded_module = LoadedModule::new(module);
        self.map.or_insert(module_id, loaded_module);
    }

    /// Resolve a StructHandle into a StructType recursively in either the cache or the `fetcher`.
    fn resolve_struct_handle(
        &self,
        module: &LoadedModule,
        idx: StructHandleIndex,
        ty_args: &[Type],
        data_view: &dyn InterpreterContext,
    ) -> VMResult<StructType> {
        let struct_handle = module.struct_handle_at(idx);
        let struct_name = module.identifier_at(struct_handle.name);
        let struct_def_module_id = StructHandleView::new(module, struct_handle).module_id();
        match self.get_loaded_module(&struct_def_module_id, data_view) {
            Ok(module) => {
                let struct_def_idx = module.get_struct_def_index(struct_name)?;
                self.resolve_struct_def(module, *struct_def_idx, ty_args, data_view)
            }
            Err(errors) => Err(errors),
        }
    }

    /// Resolve a SignatureToken into a Type recursively in either the cache or the `fetcher`.
    pub fn resolve_signature_token(
        &self,
        module: &LoadedModule,
        tok: &SignatureToken,
        ty_args: &[Type],
        data_view: &dyn InterpreterContext,
    ) -> VMResult<Type> {
        let res = match tok {
            SignatureToken::Bool => Type::Bool,
            SignatureToken::U8 => Type::U8,
            SignatureToken::U64 => Type::U64,
            SignatureToken::U128 => Type::U128,
            SignatureToken::Address => Type::Address,
            SignatureToken::TypeParameter(idx) => match ty_args.get(*idx as usize) {
                Some(ty) => ty.clone(),
                None => {
                    return Err(VMStatus::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!(
                            "type parameter index out of bounds, len {} got {}",
                            ty_args.len(),
                            idx
                        )));
                }
            },
            SignatureToken::Vector(sub_tok) => {
                let inner_ty = self.resolve_signature_token(module, sub_tok, ty_args, data_view)?;
                Type::Vector(Box::new(inner_ty))
            }
            SignatureToken::Reference(sub_tok) => {
                let inner_ty = self.resolve_signature_token(module, sub_tok, ty_args, data_view)?;
                Type::Reference(Box::new(inner_ty))
            }
            SignatureToken::MutableReference(sub_tok) => {
                let inner_ty = self.resolve_signature_token(module, sub_tok, ty_args, data_view)?;
                Type::MutableReference(Box::new(inner_ty))
            }
            SignatureToken::Struct(sh_idx, tys) => {
                let ty_args: Vec<_> = tys
                    .iter()
                    .map(|tok| self.resolve_signature_token(module, tok, ty_args, data_view))
                    .collect::<VMResult<_>>()?;
                let struct_ty = self.resolve_struct_handle(module, *sh_idx, &ty_args, data_view)?;
                Type::Struct(Box::new(struct_ty))
            }
        };
        Ok(res)
    }
}

pub fn load_and_verify_module_id(
    id: &ModuleId,
    data_view: &dyn InterpreterContext,
) -> VMResult<VerifiedModule> {
    let comp_module = match data_view.load_module(id) {
        Ok(blob) => match CompiledModule::deserialize(&blob) {
            Ok(module) => module,
            Err(err) => {
                crit!("[VM] Storage contains a malformed module with id {:?}", id);
                return Err(err);
            }
        },
        Err(err) => {
            crit!("[VM] Error fetching module with id {:?}", id);
            return Err(err);
        }
    };
    match VerifiedModule::new(comp_module) {
        Ok(module) => Ok(module),
        Err((_, mut errors)) => {
            // If there are errors there should be at least one otherwise there's an internal
            // error in the verifier. We only give back the first error. If the user wants to
            // debug things, they can do that offline.
            Err(errors
                .pop()
                .unwrap_or_else(|| VMStatus::new(StatusCode::VERIFIER_INVARIANT_VIOLATION)))
        }
    }
}
