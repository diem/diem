// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the public APIs supported by the bytecode verifier.
use crate::binary_views::BinaryIndexedView;
use diem_types::vm_status::StatusCode;
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use std::collections::{BTreeMap, HashMap};
use vm::{
    access::{ModuleAccess, ScriptAccess},
    errors::{verification_error, Location, PartialVMError, PartialVMResult, VMResult},
    file_format::{
        CompiledModule, CompiledScript, FunctionHandle, FunctionHandleIndex, ModuleHandle,
        ModuleHandleIndex, SignatureToken, StructHandle, StructHandleIndex, TableIndex,
    },
    IndexKind,
};

pub struct DependencyChecker<'a> {
    resolver: BinaryIndexedView<'a>,
    // (Module -> CompiledModule) for all dependencies
    dependency_map: BTreeMap<ModuleId, &'a CompiledModule>,
    // (Module::StructName -> handle) for all types of all dependencies
    struct_id_to_handle_map: HashMap<(ModuleId, Identifier), StructHandleIndex>,
    // (Module::FunctionName -> handle) for all public functions of all dependencies
    func_id_to_handle_map: HashMap<(ModuleId, Identifier), FunctionHandleIndex>,
}

impl<'a> DependencyChecker<'a> {
    pub fn verify_module(
        module: &'a CompiledModule,
        dependencies: impl IntoIterator<Item = &'a CompiledModule>,
    ) -> VMResult<()> {
        Self::verify_module_impl(module, dependencies)
            .map_err(|e| e.finish(Location::Module(module.self_id())))
    }

    pub fn verify_module_impl(
        module: &'a CompiledModule,
        dependencies: impl IntoIterator<Item = &'a CompiledModule>,
    ) -> PartialVMResult<()> {
        let module_id = module.self_id();
        let mut dependency_map = BTreeMap::new();
        for dependency in dependencies {
            let dependency_id = dependency.self_id();
            if module_id != dependency_id {
                dependency_map.insert(dependency_id, dependency);
            }
        }

        let mut checker = Self {
            resolver: BinaryIndexedView::Module(module),
            dependency_map,
            struct_id_to_handle_map: HashMap::new(),
            func_id_to_handle_map: HashMap::new(),
        };
        checker.build_deps_entry_point();

        // verify dependencies
        checker.verify_imported_modules(module.module_handles(), Some(module.self_handle_idx()))?;
        checker.verify_imported_structs(module.struct_handles(), Some(module.self_handle_idx()))?;
        checker.verify_imported_functions(module.function_handles(), Some(module.self_handle_idx()))
    }

    pub fn verify_script(
        script: &'a CompiledScript,
        dependencies: impl IntoIterator<Item = &'a CompiledModule>,
    ) -> VMResult<()> {
        Self::verify_script_impl(script, dependencies).map_err(|e| e.finish(Location::Script))
    }

    pub fn verify_script_impl(
        script: &'a CompiledScript,
        dependencies: impl IntoIterator<Item = &'a CompiledModule>,
    ) -> PartialVMResult<()> {
        let mut dependency_map = BTreeMap::new();
        for dependency in dependencies {
            let dependency_id = dependency.self_id();
            dependency_map.insert(dependency_id, dependency);
        }
        let mut checker = Self {
            resolver: BinaryIndexedView::Script(script),
            dependency_map,
            struct_id_to_handle_map: HashMap::new(),
            func_id_to_handle_map: HashMap::new(),
        };
        checker.build_deps_entry_point();

        checker.verify_imported_modules(script.module_handles(), None)?;
        checker.verify_imported_structs(script.struct_handles(), None)?;
        checker.verify_imported_functions(script.function_handles(), None)
    }

    fn build_deps_entry_point(&mut self) {
        for (module_id, module) in &self.dependency_map {
            // Module::StructName -> def handle idx
            for struct_def in module.struct_defs() {
                let struct_handle = module.struct_handle_at(struct_def.struct_handle);
                let struct_name = module.identifier_at(struct_handle.name);
                self.struct_id_to_handle_map.insert(
                    (module_id.clone(), struct_name.to_owned()),
                    struct_def.struct_handle,
                );
            }
            // Module::FuncName -> def handle idx
            for func_def in module.function_defs() {
                if !func_def.is_public {
                    continue;
                }
                let func_handle = module.function_handle_at(func_def.function);
                let func_name = module.identifier_at(func_handle.name);
                self.func_id_to_handle_map
                    .insert((module_id.clone(), func_name.to_owned()), func_def.function);
            }
        }
    }

    fn verify_imported_modules(
        &self,
        module_handles: &[ModuleHandle],
        self_module: Option<ModuleHandleIndex>,
    ) -> PartialVMResult<()> {
        for (idx, module_handle) in module_handles.iter().enumerate() {
            let module_id = self.resolver.module_id_for_handle(module_handle);
            if Some(ModuleHandleIndex(idx as u16)) != self_module
                && !self.dependency_map.contains_key(&module_id)
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

    fn verify_imported_structs(
        &self,
        struct_handles: &[StructHandle],
        self_module: Option<ModuleHandleIndex>,
    ) -> PartialVMResult<()> {
        for (idx, struct_handle) in struct_handles.iter().enumerate() {
            if Some(struct_handle.module) == self_module {
                continue;
            }
            let owner_module_id = self
                .resolver
                .module_id_for_handle(self.resolver.module_handle_at(struct_handle.module));
            // TODO: remove unwrap
            let owner_module = self.dependency_map.get(&owner_module_id).unwrap();
            let struct_name = self.resolver.identifier_at(struct_handle.name);
            match self
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

    fn verify_imported_functions(
        &self,
        function_handles: &[FunctionHandle],
        self_module: Option<ModuleHandleIndex>,
    ) -> PartialVMResult<()> {
        for (idx, function_handle) in function_handles.iter().enumerate() {
            if Some(function_handle.module) == self_module {
                continue;
            }
            let owner_module_id = self
                .resolver
                .module_id_for_handle(self.resolver.module_handle_at(function_handle.module));
            let function_name = self.resolver.identifier_at(function_handle.name);
            // TODO: remove unwrap
            let owner_module = self.dependency_map.get(&owner_module_id).unwrap();
            match self
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
                    let handle_params = self.resolver.signature_at(function_handle.parameters);
                    let def_params = match self.dependency_map.get(&owner_module_id) {
                        Some(module) => module.signature_at(def_handle.parameters),
                        None => {
                            return Err(verification_error(
                                StatusCode::LOOKUP_FAILED,
                                IndexKind::FunctionHandle,
                                idx as TableIndex,
                            ))
                        }
                    };
                    self.compare_cross_module_signatures(
                        &handle_params.0,
                        &def_params.0,
                        owner_module,
                    )
                    .map_err(|e| e.at_index(IndexKind::FunctionHandle, idx as TableIndex))?;

                    // same return_
                    let handle_return = self.resolver.signature_at(function_handle.return_);
                    let def_return = match self.dependency_map.get(&owner_module_id) {
                        Some(module) => module.signature_at(def_handle.return_),
                        None => {
                            return Err(verification_error(
                                StatusCode::LOOKUP_FAILED,
                                IndexKind::FunctionHandle,
                                idx as TableIndex,
                            ))
                        }
                    };
                    self.compare_cross_module_signatures(
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

    pub fn compare_cross_module_signatures(
        &self,
        handle_sig: &[SignatureToken],
        def_sig: &[SignatureToken],
        def_module: &CompiledModule,
    ) -> PartialVMResult<()> {
        if handle_sig.len() != def_sig.len() {
            return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH));
        }
        for (handle_type, def_type) in handle_sig.iter().zip(def_sig) {
            self.compare_types(handle_type, def_type, def_module)?;
        }
        Ok(())
    }

    fn compare_types(
        &self,
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
                self.compare_types(ty1, ty2, def_module)
            }
            (SignatureToken::Struct(idx1), SignatureToken::Struct(idx2)) => {
                self.compare_structs(*idx1, *idx2, def_module)
            }
            (
                SignatureToken::StructInstantiation(idx1, inst1),
                SignatureToken::StructInstantiation(idx2, inst2),
            ) => {
                self.compare_structs(*idx1, *idx2, def_module)?;
                self.compare_cross_module_signatures(inst1, inst2, def_module)
            }
            (SignatureToken::Reference(ty1), SignatureToken::Reference(ty2))
            | (SignatureToken::MutableReference(ty1), SignatureToken::MutableReference(ty2)) => {
                self.compare_types(ty1, ty2, def_module)
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
        &self,
        idx1: StructHandleIndex,
        idx2: StructHandleIndex,
        def_module: &CompiledModule,
    ) -> PartialVMResult<()> {
        // grab ModuleId and struct name for the module being verified
        let struct_handle = self.resolver.struct_handle_at(idx1);
        let module_handle = self.resolver.module_handle_at(struct_handle.module);
        let module_id = self.resolver.module_id_for_handle(module_handle);
        let struct_name = self.resolver.identifier_at(struct_handle.name);

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
}
