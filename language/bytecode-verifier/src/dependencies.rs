// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains verification of usage of dependencies for modules and scripts.
use crate::binary_views::BinaryIndexedView;
use diem_types::vm_status::StatusCode;
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use std::collections::{BTreeMap, HashMap};
use vm::{
    access::ModuleAccess,
    errors::{verification_error, Location, PartialVMError, PartialVMResult, VMResult},
    file_format::{
        CompiledModule, CompiledScript, FunctionHandleIndex, ModuleHandleIndex, SignatureToken,
        StructHandleIndex, TableIndex, Visibility,
    },
    IndexKind,
};

struct Context<'a, 'b> {
    resolver: BinaryIndexedView<'a>,
    // (Module -> CompiledModule) for (at least) all immediate dependencies
    dependency_map: BTreeMap<ModuleId, &'b CompiledModule>,
    // (Module::StructName -> handle) for all types of all dependencies
    struct_id_to_handle_map: HashMap<(ModuleId, Identifier), StructHandleIndex>,
    // (Module::FunctionName -> handle) for all non-private functions of all dependencies
    func_id_to_handle_map: HashMap<(ModuleId, Identifier), FunctionHandleIndex>,
}

impl<'a, 'b> Context<'a, 'b> {
    fn module(
        module: &'a CompiledModule,
        dependencies: impl IntoIterator<Item = &'b CompiledModule>,
    ) -> Self {
        Self::new(BinaryIndexedView::Module(module), dependencies)
    }

    fn script(
        script: &'a CompiledScript,
        dependencies: impl IntoIterator<Item = &'b CompiledModule>,
    ) -> Self {
        Self::new(BinaryIndexedView::Script(script), dependencies)
    }

    fn new(
        resolver: BinaryIndexedView<'a>,
        dependencies: impl IntoIterator<Item = &'b CompiledModule>,
    ) -> Self {
        let self_module = resolver.self_id();
        let dependency_map = dependencies
            .into_iter()
            .filter(|d| Some(d.self_id()) != self_module)
            .map(|d| (d.self_id(), d))
            .collect();

        let mut context = Self {
            resolver,
            dependency_map,
            struct_id_to_handle_map: HashMap::new(),
            func_id_to_handle_map: HashMap::new(),
        };

        for (module_id, module) in &context.dependency_map {
            // Module::StructName -> def handle idx
            for struct_def in module.struct_defs() {
                let struct_handle = module.struct_handle_at(struct_def.struct_handle);
                let struct_name = module.identifier_at(struct_handle.name);
                context.struct_id_to_handle_map.insert(
                    (module_id.clone(), struct_name.to_owned()),
                    struct_def.struct_handle,
                );
            }
            // Module::FuncName -> def handle idx
            for func_def in module.function_defs() {
                if !matches!(func_def.visibility, Visibility::Public) {
                    continue;
                }
                let func_handle = module.function_handle_at(func_def.function);
                let func_name = module.identifier_at(func_handle.name);
                context
                    .func_id_to_handle_map
                    .insert((module_id.clone(), func_name.to_owned()), func_def.function);
            }
        }
        context
    }
}

pub fn verify_module<'a>(
    module: &CompiledModule,
    dependencies: impl IntoIterator<Item = &'a CompiledModule>,
) -> VMResult<()> {
    verify_module_impl(module, dependencies)
        .map_err(|e| e.finish(Location::Module(module.self_id())))
}

fn verify_module_impl<'a>(
    module: &CompiledModule,
    dependencies: impl IntoIterator<Item = &'a CompiledModule>,
) -> PartialVMResult<()> {
    let context = &Context::module(module, dependencies);

    verify_imported_modules(context)?;
    verify_imported_structs(context)?;
    verify_imported_functions(context)
}

pub fn verify_script<'a>(
    script: &CompiledScript,
    dependencies: impl IntoIterator<Item = &'a CompiledModule>,
) -> VMResult<()> {
    verify_script_impl(script, dependencies).map_err(|e| e.finish(Location::Script))
}

pub fn verify_script_impl<'a>(
    script: &CompiledScript,
    dependencies: impl IntoIterator<Item = &'a CompiledModule>,
) -> PartialVMResult<()> {
    let context = &Context::script(script, dependencies);

    verify_imported_modules(context)?;
    verify_imported_structs(context)?;
    verify_imported_functions(context)
}

fn verify_imported_modules(context: &Context) -> PartialVMResult<()> {
    let self_module = context.resolver.self_handle_idx();
    for (idx, module_handle) in context.resolver.module_handles().iter().enumerate() {
        let module_id = context.resolver.module_id_for_handle(module_handle);
        if Some(ModuleHandleIndex(idx as u16)) != self_module
            && !context.dependency_map.contains_key(&module_id)
        {
            return Err(verification_error(
                StatusCode::MISSING_DEPENDENCY,
                IndexKind::ModuleHandle,
                idx as TableIndex,
            ));
        }
    }
    Ok(())
}

fn verify_imported_structs(context: &Context) -> PartialVMResult<()> {
    let self_module = context.resolver.self_handle_idx();
    for (idx, struct_handle) in context.resolver.struct_handles().iter().enumerate() {
        if Some(struct_handle.module) == self_module {
            continue;
        }
        let owner_module_id = context
            .resolver
            .module_id_for_handle(context.resolver.module_handle_at(struct_handle.module));
        // TODO: remove unwrap
        let owner_module = context.dependency_map.get(&owner_module_id).unwrap();
        let struct_name = context.resolver.identifier_at(struct_handle.name);
        match context
            .struct_id_to_handle_map
            .get(&(owner_module_id, struct_name.to_owned()))
        {
            Some(def_idx) => {
                let def_handle = owner_module.struct_handle_at(*def_idx);
                if struct_handle.is_nominal_resource != def_handle.is_nominal_resource
                    || struct_handle.type_parameters != def_handle.type_parameters
                {
                    return Err(verification_error(
                        StatusCode::TYPE_MISMATCH,
                        IndexKind::StructHandle,
                        idx as TableIndex,
                    ));
                }
            }
            None => {
                return Err(verification_error(
                    StatusCode::LOOKUP_FAILED,
                    IndexKind::StructHandle,
                    idx as TableIndex,
                ))
            }
        }
    }
    Ok(())
}

fn verify_imported_functions(context: &Context) -> PartialVMResult<()> {
    let self_module = context.resolver.self_handle_idx();
    for (idx, function_handle) in context.resolver.function_handles().iter().enumerate() {
        if Some(function_handle.module) == self_module {
            continue;
        }
        let owner_module_id = context
            .resolver
            .module_id_for_handle(context.resolver.module_handle_at(function_handle.module));
        let function_name = context.resolver.identifier_at(function_handle.name);
        // TODO: remove unwrap
        let owner_module = context.dependency_map.get(&owner_module_id).unwrap();
        match context
            .func_id_to_handle_map
            .get(&(owner_module_id.clone(), function_name.to_owned()))
        {
            Some(def_idx) => {
                let def_handle = owner_module.function_handle_at(*def_idx);
                // same type parameter constraints
                if function_handle.type_parameters != def_handle.type_parameters {
                    return Err(verification_error(
                        StatusCode::TYPE_MISMATCH,
                        IndexKind::FunctionHandle,
                        idx as TableIndex,
                    ));
                }
                // same parameters
                let handle_params = context.resolver.signature_at(function_handle.parameters);
                let def_params = match context.dependency_map.get(&owner_module_id) {
                    Some(module) => module.signature_at(def_handle.parameters),
                    None => {
                        return Err(verification_error(
                            StatusCode::LOOKUP_FAILED,
                            IndexKind::FunctionHandle,
                            idx as TableIndex,
                        ))
                    }
                };

                compare_cross_module_signatures(
                    context,
                    &handle_params.0,
                    &def_params.0,
                    owner_module,
                )
                .map_err(|e| e.at_index(IndexKind::FunctionHandle, idx as TableIndex))?;

                // same return_
                let handle_return = context.resolver.signature_at(function_handle.return_);
                let def_return = match context.dependency_map.get(&owner_module_id) {
                    Some(module) => module.signature_at(def_handle.return_),
                    None => {
                        return Err(verification_error(
                            StatusCode::LOOKUP_FAILED,
                            IndexKind::FunctionHandle,
                            idx as TableIndex,
                        ))
                    }
                };

                compare_cross_module_signatures(
                    context,
                    &handle_return.0,
                    &def_return.0,
                    owner_module,
                )
                .map_err(|e| e.at_index(IndexKind::FunctionHandle, idx as TableIndex))?;
            }
            None => {
                return Err(verification_error(
                    StatusCode::LOOKUP_FAILED,
                    IndexKind::FunctionHandle,
                    idx as TableIndex,
                ));
            }
        }
    }
    Ok(())
}

fn compare_cross_module_signatures(
    context: &Context,
    handle_sig: &[SignatureToken],
    def_sig: &[SignatureToken],
    def_module: &CompiledModule,
) -> PartialVMResult<()> {
    if handle_sig.len() != def_sig.len() {
        return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH));
    }
    for (handle_type, def_type) in handle_sig.iter().zip(def_sig) {
        compare_types(context, handle_type, def_type, def_module)?;
    }
    Ok(())
}

fn compare_types(
    context: &Context,
    handle_type: &SignatureToken,
    def_type: &SignatureToken,
    def_module: &CompiledModule,
) -> PartialVMResult<()> {
    match (handle_type, def_type) {
        (SignatureToken::Bool, SignatureToken::Bool)
        | (SignatureToken::U8, SignatureToken::U8)
        | (SignatureToken::U64, SignatureToken::U64)
        | (SignatureToken::U128, SignatureToken::U128)
        | (SignatureToken::Address, SignatureToken::Address)
        | (SignatureToken::Signer, SignatureToken::Signer) => Ok(()),
        (SignatureToken::Vector(ty1), SignatureToken::Vector(ty2)) => {
            compare_types(context, ty1, ty2, def_module)
        }
        (SignatureToken::Struct(idx1), SignatureToken::Struct(idx2)) => {
            compare_structs(context, *idx1, *idx2, def_module)
        }
        (
            SignatureToken::StructInstantiation(idx1, inst1),
            SignatureToken::StructInstantiation(idx2, inst2),
        ) => {
            compare_structs(context, *idx1, *idx2, def_module)?;
            compare_cross_module_signatures(context, inst1, inst2, def_module)
        }
        (SignatureToken::Reference(ty1), SignatureToken::Reference(ty2))
        | (SignatureToken::MutableReference(ty1), SignatureToken::MutableReference(ty2)) => {
            compare_types(context, ty1, ty2, def_module)
        }
        (SignatureToken::TypeParameter(idx1), SignatureToken::TypeParameter(idx2)) => {
            if idx1 != idx2 {
                Err(PartialVMError::new(StatusCode::TYPE_MISMATCH))
            } else {
                Ok(())
            }
        }
        _ => Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)),
    }
}

fn compare_structs(
    context: &Context,
    idx1: StructHandleIndex,
    idx2: StructHandleIndex,
    def_module: &CompiledModule,
) -> PartialVMResult<()> {
    // grab ModuleId and struct name for the module being verified
    let struct_handle = context.resolver.struct_handle_at(idx1);
    let module_handle = context.resolver.module_handle_at(struct_handle.module);
    let module_id = context.resolver.module_id_for_handle(module_handle);
    let struct_name = context.resolver.identifier_at(struct_handle.name);

    // grab ModuleId and struct name for the definition
    let def_struct_handle = def_module.struct_handle_at(idx2);
    let def_module_handle = def_module.module_handle_at(def_struct_handle.module);
    let def_module_id = def_module.module_id_for_handle(def_module_handle);
    let def_struct_name = def_module.identifier_at(def_struct_handle.name);

    if module_id != def_module_id || struct_name != def_struct_name {
        Err(PartialVMError::new(StatusCode::TYPE_MISMATCH))
    } else {
        Ok(())
    }
}
