// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{logging::LogContext, native_functions::NativeFunction};
use bytecode_verifier::{
    constants, instantiation_loops::InstantiationLoopChecker, verify_main_signature,
    CodeUnitVerifier, DependencyChecker, DuplicationChecker, InstructionConsistency,
    RecursiveStructDefChecker, ResourceTransitiveChecker, SignatureChecker,
};
use diem_crypto::HashValue;
use diem_infallible::Mutex;
use diem_logger::prelude::*;
use move_core_types::{
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag, TypeTag},
    value::{MoveKind, MoveKindInfo, MoveStructLayout, MoveTypeLayout},
    vm_status::{StatusCode, StatusType},
};
use move_vm_types::{
    data_store::DataStore,
    loaded_data::runtime_types::{StructType, Type},
};
use std::{collections::HashMap, fmt::Debug, hash::Hash, sync::Arc};
use vm::{
    access::{ModuleAccess, ScriptAccess},
    errors::{verification_error, Location, PartialVMError, PartialVMResult, VMError, VMResult},
    file_format::{
        Bytecode, CompiledModule, CompiledScript, Constant, ConstantPoolIndex, FieldHandleIndex,
        FieldInstantiationIndex, FunctionDefinition, FunctionDefinitionIndex, FunctionHandleIndex,
        FunctionInstantiationIndex, Kind, Signature, SignatureToken, StructDefInstantiationIndex,
        StructDefinition, StructDefinitionIndex, StructFieldInformation, TableIndex,
    },
    IndexKind,
};

// A simple cache that offers both a HashMap and a Vector lookup.
// Values are forced into a `Arc` so they can be used from multiple thread.
// Access to this cache is always under a `Mutex`.
struct BinaryCache<K, V> {
    id_map: HashMap<K, usize>,
    binaries: Vec<Arc<V>>,
}

impl<K, V> BinaryCache<K, V>
where
    K: Eq + Hash,
{
    fn new() -> Self {
        Self {
            id_map: HashMap::new(),
            binaries: vec![],
        }
    }

    fn insert(&mut self, key: K, binary: V) -> &Arc<V> {
        self.binaries.push(Arc::new(binary));
        let idx = self.binaries.len() - 1;
        self.id_map.insert(key, idx);
        self.binaries
            .last()
            .expect("BinaryCache: last() after push() impossible failure")
    }

    fn get(&self, key: &K) -> Option<&Arc<V>> {
        self.id_map
            .get(&key)
            .and_then(|idx| self.binaries.get(*idx))
    }
}

// A script cache is a map from the hash value of a script and the `Script` itself.
// Script are added in the cache once verified and so getting a script out the cache
// does not require further verification (except for parameters and type parameters)
struct ScriptCache {
    scripts: BinaryCache<HashValue, Script>,
}

impl ScriptCache {
    fn new() -> Self {
        Self {
            scripts: BinaryCache::new(),
        }
    }

    fn get(&self, hash: &HashValue) -> Option<Arc<Function>> {
        self.scripts.get(hash).map(|script| script.entry_point())
    }

    fn insert(&mut self, hash: HashValue, script: Script) -> PartialVMResult<Arc<Function>> {
        match self.get(&hash) {
            Some(script) => Ok(script),
            None => Ok(self.scripts.insert(hash, script).entry_point()),
        }
    }
}

// A ModuleCache is the core structure in the Loader.
// It holds all Modules, Types and Functions loaded.
// Types and Functions are pushed globally to the ModuleCache.
// All accesses to the ModuleCache are under lock (exclusive).
pub struct ModuleCache {
    modules: BinaryCache<ModuleId, Module>,
    structs: Vec<Arc<StructType>>,
    functions: Vec<Arc<Function>>,
}

impl ModuleCache {
    fn new() -> Self {
        Self {
            modules: BinaryCache::new(),
            structs: vec![],
            functions: vec![],
        }
    }

    //
    // Common "get" operations
    //

    // Retrieve a module by `ModuleId`. The module may have not been loaded yet in which
    // case `None` is returned
    fn module_at(&self, id: &ModuleId) -> Option<Arc<Module>> {
        self.modules.get(id).map(|module| Arc::clone(module))
    }

    // Retrieve a function by index
    fn function_at(&self, idx: usize) -> Arc<Function> {
        Arc::clone(&self.functions[idx])
    }

    // Retrieve a struct by index
    fn struct_at(&self, idx: usize) -> Arc<StructType> {
        Arc::clone(&self.structs[idx])
    }

    //
    // Insertion is under lock and it's a pretty heavy operation.
    // The VM is pretty much stopped waiting for this to finish
    //

    fn insert(
        &mut self,
        id: ModuleId,
        module: CompiledModule,
        log_context: &impl LogContext,
    ) -> VMResult<Arc<Module>> {
        if let Some(module) = self.module_at(&id) {
            return Ok(module);
        }

        // we need this operation to be transactional, if an error occurs we must
        // leave a clean state
        self.add_module(&module, log_context)?;
        match Module::new(module, self) {
            Ok(module) => Ok(Arc::clone(self.modules.insert(id, module))),
            Err((err, module)) => {
                // remove all structs and functions that have been pushed
                let strut_def_count = module.struct_defs().len();
                self.structs.truncate(self.structs.len() - strut_def_count);
                let function_count = module.function_defs().len();
                self.functions
                    .truncate(self.functions.len() - function_count);
                Err(err.finish(Location::Undefined))
            }
        }
    }

    fn add_module(
        &mut self,
        module: &CompiledModule,
        log_context: &impl LogContext,
    ) -> VMResult<()> {
        let starting_idx = self.structs.len();
        for (idx, struct_def) in module.struct_defs().iter().enumerate() {
            let st = self.make_struct_type(module, struct_def, StructDefinitionIndex(idx as u16));
            self.structs.push(Arc::new(st));
        }
        self.load_field_types(module, starting_idx, log_context)
            .map_err(|err| {
                // clean up the structs that were cached
                self.structs.truncate(starting_idx);
                err.finish(Location::Undefined)
            })?;
        for (idx, func) in module.function_defs().iter().enumerate() {
            let findex = FunctionDefinitionIndex(idx as TableIndex);
            let function = Function::new(findex, func, module);
            self.functions.push(Arc::new(function));
        }
        Ok(())
    }

    fn make_struct_type(
        &self,
        module: &CompiledModule,
        struct_def: &StructDefinition,
        idx: StructDefinitionIndex,
    ) -> StructType {
        let struct_handle = module.struct_handle_at(struct_def.struct_handle);
        let is_resource = struct_handle.is_nominal_resource;
        let name = module.identifier_at(struct_handle.name).to_owned();
        let type_parameters = struct_handle.type_parameters.clone();
        let module = module.self_id();
        StructType {
            fields: vec![],
            is_resource,
            type_parameters,
            name,
            module,
            struct_def: idx,
        }
    }

    fn load_field_types(
        &mut self,
        module: &CompiledModule,
        starting_idx: usize,
        log_context: &impl LogContext,
    ) -> PartialVMResult<()> {
        let mut field_types = vec![];
        for struct_def in module.struct_defs() {
            let fields = match &struct_def.field_information {
                StructFieldInformation::Native => unreachable!("native structs have been removed"),
                StructFieldInformation::Declared(fields) => fields,
            };

            let mut field_tys = vec![];
            for field in fields {
                let ty = self.make_type_while_loading(module, &field.signature.0)?;
                assume!(field_tys.len() < usize::max_value());
                field_tys.push(ty);
            }

            field_types.push(field_tys);
        }
        let mut struct_idx = starting_idx;
        for fields in field_types {
            match Arc::get_mut(&mut self.structs[struct_idx]) {
                Some(struct_type) => struct_type.fields = fields,
                None => {
                    // we have pending references to the `Arc` which is impossible,
                    // given the code that adds the `Arc` is above and no reference to
                    // it should exist.
                    // So in the spirit of not crashing we just rewrite the entire `Arc`
                    // over and log the issue.
                    log_context.alert();
                    error!(
                        *log_context,
                        "Arc<StructType> cannot have any live reference while publishing"
                    );
                    let mut struct_type = (*self.structs[struct_idx]).clone();
                    struct_type.fields = fields;
                    self.structs[struct_idx] = Arc::new(struct_type);
                }
            }
            struct_idx += 1;
        }
        Ok(())
    }

    // `make_type` is the entry point to "translate" a `SignatureToken` to a `Type`
    fn make_type(&self, module: &CompiledModule, tok: &SignatureToken) -> PartialVMResult<Type> {
        self.make_type_internal(module, tok, &|struct_name, module_id| {
            Ok(self.resolve_struct_by_name(struct_name, module_id)?.0)
        })
    }

    // While in the process of loading, and before a `Module` is saved into the cache the loader
    // needs to resolve type references to the module itself (self) "manually"; that is,
    // looping through the types of the module itself
    fn make_type_while_loading(
        &self,
        module: &CompiledModule,
        tok: &SignatureToken,
    ) -> PartialVMResult<Type> {
        let self_id = module.self_id();
        self.make_type_internal(module, tok, &|struct_name, module_id| {
            if module_id == &self_id {
                // module has not been published yet, loop through the types
                for (idx, struct_type) in self.structs.iter().enumerate().rev() {
                    if &struct_type.module != module_id {
                        break;
                    }
                    if struct_type.name.as_ident_str() == struct_name {
                        return Ok(idx);
                    }
                }
                Err(
                    PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(format!(
                        "Cannot find {:?}::{:?} in publishing module",
                        module_id, struct_name
                    )),
                )
            } else {
                Ok(self.resolve_struct_by_name(struct_name, module_id)?.0)
            }
        })
    }

    // `make_type_internal` returns a `Type` given a signature and a resolver which
    // is resonsible to map a local struct index to a global one
    fn make_type_internal<F>(
        &self,
        module: &CompiledModule,
        tok: &SignatureToken,
        resolver: &F,
    ) -> PartialVMResult<Type>
    where
        F: Fn(&IdentStr, &ModuleId) -> PartialVMResult<usize>,
    {
        let res = match tok {
            SignatureToken::Bool => Type::Bool,
            SignatureToken::U8 => Type::U8,
            SignatureToken::U64 => Type::U64,
            SignatureToken::U128 => Type::U128,
            SignatureToken::Address => Type::Address,
            SignatureToken::Signer => Type::Signer,
            SignatureToken::TypeParameter(idx) => Type::TyParam(*idx as usize),
            SignatureToken::Vector(inner_tok) => {
                let inner_type = self.make_type_internal(module, inner_tok, resolver)?;
                Type::Vector(Box::new(inner_type))
            }
            SignatureToken::Reference(inner_tok) => {
                let inner_type = self.make_type_internal(module, inner_tok, resolver)?;
                Type::Reference(Box::new(inner_type))
            }
            SignatureToken::MutableReference(inner_tok) => {
                let inner_type = self.make_type_internal(module, inner_tok, resolver)?;
                Type::MutableReference(Box::new(inner_type))
            }
            SignatureToken::Struct(sh_idx) => {
                let struct_handle = module.struct_handle_at(*sh_idx);
                let struct_name = module.identifier_at(struct_handle.name);
                let module_handle = module.module_handle_at(struct_handle.module);
                let module_id = ModuleId::new(
                    *module.address_identifier_at(module_handle.address),
                    module.identifier_at(module_handle.name).to_owned(),
                );
                let def_idx = resolver(struct_name, &module_id)?;
                Type::Struct(def_idx)
            }
            SignatureToken::StructInstantiation(sh_idx, tys) => {
                let type_parameters: Vec<_> = tys
                    .iter()
                    .map(|tok| self.make_type_internal(module, tok, resolver))
                    .collect::<PartialVMResult<_>>()?;
                let struct_handle = module.struct_handle_at(*sh_idx);
                let struct_name = module.identifier_at(struct_handle.name);
                let module_handle = module.module_handle_at(struct_handle.module);
                let module_id = ModuleId::new(
                    *module.address_identifier_at(module_handle.address),
                    module.identifier_at(module_handle.name).to_owned(),
                );
                let def_idx = resolver(struct_name, &module_id)?;
                Type::StructInstantiation(def_idx, type_parameters)
            }
        };
        Ok(res)
    }

    // Given a ModuleId::struct_name, retrieve the `StructType` and the index associated.
    // Return and error if the type has not been loaded
    fn resolve_struct_by_name(
        &self,
        struct_name: &IdentStr,
        module_id: &ModuleId,
    ) -> PartialVMResult<(usize, Arc<StructType>)> {
        match self
            .modules
            .get(module_id)
            .and_then(|module| module.struct_map.get(struct_name))
        {
            Some(struct_idx) => Ok((*struct_idx, Arc::clone(&self.structs[*struct_idx]))),
            None => Err(
                PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(format!(
                    "Cannot find {:?}::{:?} in cache",
                    module_id, struct_name
                )),
            ),
        }
    }

    // Given a ModuleId::func_name, retrieve the `StructType` and the index associated.
    // Return and error if the function has not been loaded
    fn resolve_function_by_name(
        &self,
        func_name: &IdentStr,
        module_id: &ModuleId,
    ) -> PartialVMResult<usize> {
        match self
            .modules
            .get(module_id)
            .and_then(|module| module.function_map.get(func_name))
        {
            Some(func_idx) => Ok(*func_idx),
            None => Err(
                PartialVMError::new(StatusCode::FUNCTION_RESOLUTION_FAILURE).with_message(format!(
                    "Cannot find {:?}::{:?} in cache",
                    module_id, func_name
                )),
            ),
        }
    }
}

//
// Loader
//

// A Loader is responsible to load scripts and modules and holds the cache of all loaded
// entities. Each cache is protected by a `Mutex`. Operation in the Loader must be thread safe
// (operating on values on the stack) and when cache needs updating the mutex must be taken.
// The `pub(crate)` API is what a Loader offers to the runtime.
pub(crate) struct Loader {
    scripts: Mutex<ScriptCache>,
    module_cache: Mutex<ModuleCache>,
    type_cache: Mutex<TypeCache>,
}

impl Loader {
    pub(crate) fn new() -> Self {
        //println!("new loader");
        Self {
            scripts: Mutex::new(ScriptCache::new()),
            module_cache: Mutex::new(ModuleCache::new()),
            type_cache: Mutex::new(TypeCache::new()),
        }
    }

    //
    // Script verification and loading
    //

    // Scripts are verified and dependencies are loaded.
    // Effectively that means modules are cached from leaf to root in the dependency DAG.
    // If a dependency error is found, loading stops and the error is returned.
    // However all modules cached up to that point stay loaded.

    // Entry point for script execution (`MoveVM::execute_script`).
    // Verifies the script if it is not in the cache of scripts loaded.
    // Type parameters are checked as well after every type is loaded.
    pub(crate) fn load_script(
        &self,
        script_blob: &[u8],
        ty_args: &[TypeTag],
        data_store: &mut impl DataStore,
        log_context: &impl LogContext,
    ) -> VMResult<(Arc<Function>, Vec<Type>)> {
        // retrieve or load the script
        let hash_value = HashValue::sha3_256_of(script_blob);
        let opt_main = self.scripts.lock().get(&hash_value);
        let main = match opt_main {
            Some(main) => main,
            None => {
                let ver_script =
                    self.deserialize_and_verify_script(script_blob, data_store, log_context)?;
                let script = Script::new(ver_script, &hash_value, &self.module_cache.lock())?;
                self.scripts
                    .lock()
                    .insert(hash_value, script)
                    .map_err(|e| e.finish(Location::Script))?
            }
        };

        // verify type arguments
        let mut type_params = vec![];
        for ty in ty_args {
            type_params.push(self.load_type(ty, data_store, log_context)?);
        }
        self.verify_ty_args(main.type_parameters(), &type_params)
            .map_err(|e| e.finish(Location::Script))?;

        Ok((main, type_params))
    }

    // The process of deserialization and verification is not and it must not be under lock.
    // So when publishing modules through the dependency DAG it may happen that a different
    // thread had loaded the module after this process fetched it from storage.
    // Caching will take care of that by asking for each dependency module again under lock.
    fn deserialize_and_verify_script(
        &self,
        script: &[u8],
        data_store: &mut impl DataStore,
        log_context: &impl LogContext,
    ) -> VMResult<CompiledScript> {
        let script = match CompiledScript::deserialize(script) {
            Ok(script) => script,
            Err(err) => {
                log_context.alert();
                error!(
                    *log_context,
                    "[VM] deserializer for script returned error: {:?}", err,
                );
                let msg = format!("Deserialization error: {:?}", err);
                return Err(PartialVMError::new(StatusCode::CODE_DESERIALIZATION_ERROR)
                    .with_message(msg)
                    .finish(Location::Script));
            }
        };

        match self.verify_script(&script) {
            Ok(_) => {
                // verify dependencies
                let deps = script_dependencies(&script);
                let loaded_deps = self.load_dependencies_verify_no_missing_dependencies(
                    deps,
                    data_store,
                    log_context,
                )?;
                self.verify_script_dependencies(&script, loaded_deps)?;
                Ok(script)
            }
            Err(err) => {
                log_context.alert();
                error!(
                    *log_context,
                    "[VM] bytecode verifier returned errors for script: {:?}", err
                );
                Err(err)
            }
        }
    }

    // Script verification steps.
    // See `verify_module()` for module verification steps.
    fn verify_script(&self, script: &CompiledScript) -> VMResult<()> {
        DuplicationChecker::verify_script(&script)?;
        SignatureChecker::verify_script(&script)?;
        InstructionConsistency::verify_script(&script)?;
        constants::verify_script(&script)?;
        CodeUnitVerifier::verify_script(&script)?;
        verify_main_signature(&script)
    }

    fn verify_script_dependencies(
        &self,
        script: &CompiledScript,
        dependencies: Vec<Arc<Module>>,
    ) -> VMResult<()> {
        let mut deps = vec![];
        for dep in &dependencies {
            deps.push(dep.module());
        }
        DependencyChecker::verify_script(script, deps)
    }

    //
    // Module verification and loading
    //

    // Entry point for function execution (`MoveVM::execute_function`).
    // Loading verifies the module if it was never loaded.
    // Type parameters are checked as well after every type is loaded.
    pub(crate) fn load_function(
        &self,
        function_name: &IdentStr,
        module_id: &ModuleId,
        ty_args: &[TypeTag],
        data_store: &mut impl DataStore,
        log_context: &impl LogContext,
    ) -> VMResult<(Arc<Function>, Vec<Type>)> {
        self.load_module_expect_no_missing_dependencies(module_id, data_store, log_context)?;
        let idx = self
            .module_cache
            .lock()
            .resolve_function_by_name(function_name, module_id)
            .map_err(|err| {
                expect_no_verification_errors(err.finish(Location::Undefined), log_context)
            })?;
        let func = self.module_cache.lock().function_at(idx);

        // verify type arguments
        let mut type_params = vec![];
        for ty in ty_args {
            type_params.push(self.load_type(ty, data_store, log_context)?);
        }
        self.verify_ty_args(func.type_parameters(), &type_params)
            .map_err(|e| e.finish(Location::Module(module_id.clone())))?;

        Ok((func, type_params))
    }

    // Entry point for module publishing (`MoveVM::publish_module`).
    // A module to be published must be loadable.
    // This step performs all verification steps to load the module without loading it.
    // The module is not added to the code cache. It is simply published to the data cache.
    // See `verify_script()` for script verification steps.
    pub(crate) fn verify_module_verify_no_missing_dependencies(
        &self,
        module: &CompiledModule,
        data_store: &mut impl DataStore,
        log_context: &impl LogContext,
    ) -> VMResult<()> {
        // A standard DFS algorithm that follows the dependency chain from the `source_module_id`
        // to see whether the `dst_module_id` is in its dependencies. This function can only be
        // called when the module is fully verified and the `module_cache` is properly prepared.
        fn is_reachable_via_dependencies(
            dst_module_id: &ModuleId,
            src_module_id: &ModuleId,
            loader: &Loader,
        ) -> bool {
            if dst_module_id == src_module_id {
                return true;
            }
            let src_module = loader.get_module(src_module_id);
            module_dependencies(src_module.module())
                .iter()
                .any(|dep| is_reachable_via_dependencies(dst_module_id, dep, loader))
        }

        // Performs all verification steps to load the module without loading it, i.e., the new
        // module will NOT show up in `module_cache`. In the module republishing case, it means
        // that the old module is still in the `module_cache`, unless a Loader is created (which
        // means a new MoveVm instance needs to be created).
        self.verify_module(module, data_store, true, log_context)?;

        // Check whether publishing/updating this new module introduces cyclic dependencies, i.e.,
        // module A depends on module B while at the same time, B (transitively) depends on A.
        // This can be checked by a DFS walk from module A with detection for back edges.
        let dst_module_id = module.self_id();
        if module_dependencies(module)
            .iter()
            .any(|dep| is_reachable_via_dependencies(&dst_module_id, dep, &self))
        {
            return Err(PartialVMError::new(StatusCode::CYCLIC_MODULE_DEPENDENCY)
                .finish(Location::Module(module.self_id())));
        }
        Ok(())
    }

    fn verify_module_expect_no_missing_dependencies(
        &self,
        module: &CompiledModule,
        data_store: &mut impl DataStore,
        log_context: &impl LogContext,
    ) -> VMResult<()> {
        self.verify_module(module, data_store, false, log_context)
    }

    fn verify_module(
        &self,
        module: &CompiledModule,
        data_store: &mut impl DataStore,
        verify_no_missing_modules: bool,
        log_context: &impl LogContext,
    ) -> VMResult<()> {
        DuplicationChecker::verify_module(&module)?;
        SignatureChecker::verify_module(&module)?;
        InstructionConsistency::verify_module(&module)?;
        ResourceTransitiveChecker::verify_module(&module)?;
        constants::verify_module(&module)?;
        RecursiveStructDefChecker::verify_module(&module)?;
        InstantiationLoopChecker::verify_module(&module)?;
        CodeUnitVerifier::verify_module(&module)?;
        Self::check_natives(&module)?;

        let deps = module_dependencies(&module);
        let loaded_deps = if verify_no_missing_modules {
            self.load_dependencies_verify_no_missing_dependencies(deps, data_store, log_context)?
        } else {
            self.load_dependencies_expect_no_missing_dependencies(deps, data_store, log_context)?
        };

        self.verify_module_dependencies(module, loaded_deps)
    }

    fn verify_module_dependencies(
        &self,
        module: &CompiledModule,
        dependencies: Vec<Arc<Module>>,
    ) -> VMResult<()> {
        let mut deps = vec![];
        for dep in &dependencies {
            deps.push(dep.module());
        }
        DependencyChecker::verify_module(module, deps)
    }

    // All native functions must be known to the loader
    fn check_natives(module: &CompiledModule) -> VMResult<()> {
        fn check_natives_impl(module: &CompiledModule) -> PartialVMResult<()> {
            for (idx, native_function) in module
                .function_defs()
                .iter()
                .filter(|fdv| fdv.is_native())
                .enumerate()
            {
                let fh = module.function_handle_at(native_function.function);
                let mh = module.module_handle_at(fh.module);
                NativeFunction::resolve(
                    module.address_identifier_at(mh.address),
                    module.identifier_at(mh.name).as_str(),
                    module.identifier_at(fh.name).as_str(),
                )
                .ok_or_else(|| {
                    verification_error(
                        StatusCode::MISSING_DEPENDENCY,
                        IndexKind::FunctionHandle,
                        idx as TableIndex,
                    )
                })?;
            }
            // TODO: fix check and error code if we leave something around for native structs.
            // For now this generates the only error test cases care about...
            for (idx, struct_def) in module.struct_defs().iter().enumerate() {
                if struct_def.field_information == StructFieldInformation::Native {
                    return Err(verification_error(
                        StatusCode::MISSING_DEPENDENCY,
                        IndexKind::FunctionHandle,
                        idx as TableIndex,
                    ));
                }
            }
            Ok(())
        }
        check_natives_impl(module).map_err(|e| e.finish(Location::Module(module.self_id())))
    }

    //
    // Helpers for loading and verification
    //

    fn load_type(
        &self,
        type_tag: &TypeTag,
        data_store: &mut impl DataStore,
        log_context: &impl LogContext,
    ) -> VMResult<Type> {
        Ok(match type_tag {
            TypeTag::Bool => Type::Bool,
            TypeTag::U8 => Type::U8,
            TypeTag::U64 => Type::U64,
            TypeTag::U128 => Type::U128,
            TypeTag::Address => Type::Address,
            TypeTag::Signer => Type::Signer,
            TypeTag::Vector(tt) => {
                Type::Vector(Box::new(self.load_type(tt, data_store, log_context)?))
            }
            TypeTag::Struct(struct_tag) => {
                let module_id = ModuleId::new(struct_tag.address, struct_tag.module.clone());
                self.load_module_verify_no_missing_dependencies(
                    &module_id,
                    data_store,
                    log_context,
                )?;
                let (idx, struct_type) = self
                    .module_cache
                    .lock()
                    // GOOD module was loaded above
                    .resolve_struct_by_name(&struct_tag.name, &module_id)
                    .map_err(|e| e.finish(Location::Undefined))?;
                if struct_type.type_parameters.is_empty() && struct_tag.type_params.is_empty() {
                    Type::Struct(idx)
                } else {
                    let mut type_params = vec![];
                    for ty_param in &struct_tag.type_params {
                        type_params.push(self.load_type(ty_param, data_store, log_context)?);
                    }
                    self.verify_ty_args(&struct_type.type_parameters, &type_params)
                        .map_err(|e| e.finish(Location::Undefined))?;
                    Type::StructInstantiation(idx, type_params)
                }
            }
        })
    }

    // The process of loading is recursive, and module are cached by the loader as soon as
    // they are verifiable (including dependencies).
    // Effectively that means modules are cached from leaf to root in the dependency DAG.
    // If a dependency error is found, loading stops and the error is returned.
    // However all modules cached up to that point stay loaded.
    // In principle that is not the safest model but it is justified
    // by the fact that publishing is limited, and complex/tricky dependencies error
    // are difficult or impossible to accomplish in reality.

    // The process of deserialization and verification is not and it must not be under lock.
    // So when publishing modules through the dependency DAG it may happen that a different
    // thread had loaded the module after this process fetched it from storage.
    // Caching will take care of that by asking for each dependency module again under lock.
    fn load_module(
        &self,
        id: &ModuleId,
        data_store: &mut impl DataStore,
        verify_no_missing_modules: bool,
        log_context: &impl LogContext,
    ) -> VMResult<Arc<Module>> {
        // kept private to `load_module` to prevent verification errors from leaking
        // and not being marked as invariant violations
        fn deserialize_and_verify_module(
            loader: &Loader,
            bytes: Vec<u8>,
            data_store: &mut impl DataStore,
            log_context: &impl LogContext,
        ) -> VMResult<CompiledModule> {
            let module = CompiledModule::deserialize(&bytes).map_err(|_| {
                PartialVMError::new(StatusCode::CODE_DESERIALIZATION_ERROR)
                    .finish(Location::Undefined)
            })?;
            loader.verify_module_expect_no_missing_dependencies(
                &module,
                data_store,
                log_context,
            )?;
            Ok(module)
        }

        if let Some(module) = self.module_cache.lock().module_at(id) {
            return Ok(module);
        }

        let bytes = match data_store.load_module(id) {
            Ok(bytes) => bytes,
            Err(err) if verify_no_missing_modules => return Err(err),
            Err(err) => {
                log_context.alert();
                error!(*log_context, "[VM] Error fetching module with id {:?}", id);
                return Err(expect_no_verification_errors(err, log_context));
            }
        };

        let module = deserialize_and_verify_module(self, bytes, data_store, log_context)
            .map_err(|err| expect_no_verification_errors(err, log_context))?;
        self.module_cache
            .lock()
            .insert(id.clone(), module, log_context)
    }

    // Returns a verifier error if the module does not exist
    fn load_module_verify_no_missing_dependencies(
        &self,
        id: &ModuleId,
        data_store: &mut impl DataStore,
        log_context: &impl LogContext,
    ) -> VMResult<Arc<Module>> {
        self.load_module(id, data_store, true, log_context)
    }

    // Expects all modules to be on chain. Gives an invariant violation if it is not found
    fn load_module_expect_no_missing_dependencies(
        &self,
        id: &ModuleId,
        data_store: &mut impl DataStore,
        log_context: &impl LogContext,
    ) -> VMResult<Arc<Module>> {
        self.load_module(id, data_store, false, log_context)
    }

    // Returns a verifier error if the module does not exist
    fn load_dependencies_verify_no_missing_dependencies(
        &self,
        deps: Vec<ModuleId>,
        data_store: &mut impl DataStore,
        log_context: &impl LogContext,
    ) -> VMResult<Vec<Arc<Module>>> {
        deps.into_iter()
            .map(|dep| {
                self.load_module_verify_no_missing_dependencies(&dep, data_store, log_context)
            })
            .collect()
    }

    // Expects all modules to be on chain. Gives an invariant violation if it is not found
    fn load_dependencies_expect_no_missing_dependencies(
        &self,
        deps: Vec<ModuleId>,
        data_store: &mut impl DataStore,
        log_context: &impl LogContext,
    ) -> VMResult<Vec<Arc<Module>>> {
        deps.into_iter()
            .map(|dep| {
                self.load_module_expect_no_missing_dependencies(&dep, data_store, log_context)
            })
            .collect()
    }

    // Verify the kind (constraints) of an instantiation.
    // Both function and script invocation use this function to verify correctness
    // of type arguments provided
    fn verify_ty_args(&self, constraints: &[Kind], ty_args: &[Type]) -> PartialVMResult<()> {
        if constraints.len() != ty_args.len() {
            return Err(PartialVMError::new(
                StatusCode::NUMBER_OF_TYPE_ARGUMENTS_MISMATCH,
            ));
        }
        for (ty, expected_k) in ty_args.iter().zip(constraints) {
            let k = if self.is_resource(ty) {
                Kind::Resource
            } else {
                Kind::Copyable
            };
            if !k.is_sub_kind_of(*expected_k) {
                return Err(PartialVMError::new(StatusCode::CONSTRAINT_KIND_MISMATCH));
            }
        }
        Ok(())
    }

    //
    // Internal helpers
    //

    fn function_at(&self, idx: usize) -> Arc<Function> {
        self.module_cache.lock().function_at(idx)
    }

    fn struct_at(&self, idx: usize) -> Arc<StructType> {
        self.module_cache.lock().struct_at(idx)
    }

    fn get_module(&self, idx: &ModuleId) -> Arc<Module> {
        Arc::clone(
            self.module_cache
                .lock()
                .modules
                .get(idx)
                .expect("ModuleId on Function must exist"),
        )
    }

    fn get_script(&self, hash: &HashValue) -> Arc<Script> {
        Arc::clone(
            self.scripts
                .lock()
                .scripts
                .get(hash)
                .expect("Script hash on Function must exist"),
        )
    }

    fn is_resource(&self, type_: &Type) -> bool {
        match type_ {
            Type::Struct(idx) => self.module_cache.lock().struct_at(*idx).is_resource,
            Type::StructInstantiation(idx, instantiation) => {
                if self.module_cache.lock().struct_at(*idx).is_resource {
                    true
                } else {
                    for ty in instantiation {
                        if self.is_resource(ty) {
                            return true;
                        }
                    }
                    false
                }
            }
            Type::Vector(ty) => self.is_resource(ty),
            _ => false,
        }
    }
}

//
// Resolver
//

// A simple wrapper for a `Module` or a `Script` in the `Resolver`
enum BinaryType {
    Module(Arc<Module>),
    Script(Arc<Script>),
}

// A Resolver is a simple and small structure allocated on the stack and used by the
// interpreter. It's the only API known to the interpreter and it's tailored to the interpreter
// needs.
pub(crate) struct Resolver<'a> {
    loader: &'a Loader,
    binary: BinaryType,
}

impl<'a> Resolver<'a> {
    fn for_module(loader: &'a Loader, module: Arc<Module>) -> Self {
        let binary = BinaryType::Module(module);
        Self { loader, binary }
    }

    fn for_script(loader: &'a Loader, script: Arc<Script>) -> Self {
        let binary = BinaryType::Script(script);
        Self { loader, binary }
    }

    //
    // Constant resolution
    //

    pub(crate) fn constant_at(&self, idx: ConstantPoolIndex) -> &Constant {
        match &self.binary {
            BinaryType::Module(module) => module.module.constant_at(idx),
            BinaryType::Script(script) => script.script.constant_at(idx),
        }
    }

    //
    // Function resolution
    //

    pub(crate) fn function_from_handle(&self, idx: FunctionHandleIndex) -> Arc<Function> {
        let idx = match &self.binary {
            BinaryType::Module(module) => module.function_at(idx.0),
            BinaryType::Script(script) => script.function_at(idx.0),
        };
        self.loader.function_at(idx)
    }

    pub(crate) fn function_from_instantiation(
        &self,
        idx: FunctionInstantiationIndex,
    ) -> Arc<Function> {
        let func_inst = match &self.binary {
            BinaryType::Module(module) => module.function_instantiation_at(idx.0),
            BinaryType::Script(script) => script.function_instantiation_at(idx.0),
        };
        self.loader.function_at(func_inst.handle)
    }

    pub(crate) fn instantiate_generic_function(
        &self,
        idx: FunctionInstantiationIndex,
        type_params: &[Type],
    ) -> PartialVMResult<Vec<Type>> {
        let func_inst = match &self.binary {
            BinaryType::Module(module) => module.function_instantiation_at(idx.0),
            BinaryType::Script(script) => script.function_instantiation_at(idx.0),
        };
        let mut instantiation = vec![];
        for ty in &func_inst.instantiation {
            instantiation.push(ty.subst(type_params)?);
        }
        Ok(instantiation)
    }

    pub(crate) fn type_params_count(
        &self,
        idx: FunctionInstantiationIndex,
    ) -> PartialVMResult<usize> {
        let func_inst = match &self.binary {
            BinaryType::Module(module) => module.function_instantiation_at(idx.0),
            BinaryType::Script(script) => script.function_instantiation_at(idx.0),
        };
        Ok(func_inst.instantiation.len())
    }

    //
    // Type resolution
    //

    pub(crate) fn struct_from_definition(&self, idx: StructDefinitionIndex) -> Arc<StructType> {
        match &self.binary {
            BinaryType::Module(module) => {
                let gidx = module.struct_at(idx);
                self.loader.struct_at(gidx)
            }
            BinaryType::Script(_) => unreachable!("Scripts cannot have type instructions"),
        }
    }

    pub(crate) fn get_struct_type(&self, idx: StructDefinitionIndex) -> Type {
        let struct_def = match &self.binary {
            BinaryType::Module(module) => module.struct_at(idx),
            BinaryType::Script(_) => unreachable!("Scripts cannot have type instructions"),
        };
        Type::Struct(struct_def)
    }

    pub(crate) fn instantiate_generic_type(
        &self,
        idx: StructDefInstantiationIndex,
        ty_args: &[Type],
    ) -> PartialVMResult<Type> {
        let struct_inst = match &self.binary {
            BinaryType::Module(module) => module.struct_instantiation_at(idx.0),
            BinaryType::Script(_) => unreachable!("Scripts cannot have type instructions"),
        };
        Ok(Type::StructInstantiation(
            struct_inst.def,
            struct_inst
                .instantiation
                .iter()
                .map(|ty| ty.subst(ty_args))
                .collect::<PartialVMResult<_>>()?,
        ))
    }

    pub(crate) fn is_resource(&self, ty: &Type) -> bool {
        self.loader.is_resource(ty)
    }

    pub(crate) fn instantiation_is_resource(
        &self,
        idx: StructDefInstantiationIndex,
        instantiation: &[Type],
    ) -> PartialVMResult<bool> {
        let struct_inst = match &self.binary {
            BinaryType::Module(module) => module.struct_instantiation_at(idx.0),
            BinaryType::Script(_) => unreachable!("Scripts cannot have type instructions"),
        };
        let struct_ty = self.loader.struct_at(struct_inst.def);
        let is_nominal_resource = struct_ty.is_resource;
        Ok(if is_nominal_resource {
            true
        } else {
            let mut is_resource = false;
            for ty in &struct_inst.instantiation {
                if self.is_resource(&ty.subst(instantiation)?) {
                    is_resource = true;
                }
            }
            is_resource
        })
    }

    //
    // Fields resolution
    //

    pub(crate) fn field_offset(&self, idx: FieldHandleIndex) -> usize {
        match &self.binary {
            BinaryType::Module(module) => module.field_offset(idx),
            BinaryType::Script(_) => unreachable!("Scripts cannot have field instructions"),
        }
    }

    pub(crate) fn field_instantiation_offset(&self, idx: FieldInstantiationIndex) -> usize {
        match &self.binary {
            BinaryType::Module(module) => module.field_instantiation_offset(idx),
            BinaryType::Script(_) => unreachable!("Scripts cannot have field instructions"),
        }
    }

    pub(crate) fn field_count(&self, idx: StructDefinitionIndex) -> u16 {
        match &self.binary {
            BinaryType::Module(module) => module.field_count(idx.0),
            BinaryType::Script(_) => unreachable!("Scripts cannot have type instructions"),
        }
    }

    pub(crate) fn field_instantiation_count(&self, idx: StructDefInstantiationIndex) -> u16 {
        match &self.binary {
            BinaryType::Module(module) => module.field_instantiation_count(idx.0),
            BinaryType::Script(_) => unreachable!("Scripts cannot have type instructions"),
        }
    }

    pub(crate) fn type_to_type_layout(&self, ty: &Type) -> PartialVMResult<MoveTypeLayout> {
        self.loader.type_to_type_layout(ty)
    }

    pub(crate) fn loader(&self) -> &Loader {
        &self.loader
    }
}

// A Module is very similar to a binary Module but data is "transformed" to a representation
// more appropriate to execution.
// When code executes indexes in instructions are resolved against those runtime structure
// so that any data needed for execution is immediately available
#[derive(Debug)]
pub(crate) struct Module {
    id: ModuleId,
    // primitive pools
    module: CompiledModule,

    //
    // types as indexes into the Loader type list
    //

    // struct references carry the index into the global vector of types.
    // That is effectively an indirection over the ref table:
    // the instruction carries an index into this table which contains the index into the
    // glabal table of types. No instantiation of generic types is saved into the global table.
    struct_refs: Vec<usize>,
    structs: Vec<StructDef>,
    // materialized instantiations, whether partial or not
    struct_instantiations: Vec<StructInstantiation>,

    // functions as indexes into the Loader function list
    // That is effectively an indirection over the ref table:
    // the instruction carries an index into this table which contains the index into the
    // glabal table of functions. No instantiation of generic functions is saved into
    // the global table.
    function_refs: Vec<usize>,
    // materialized instantiations, whether partial or not
    function_instantiations: Vec<FunctionInstantiation>,

    // fields as a pair of index, first to the type, second to the field position in that type
    field_handles: Vec<FieldHandle>,
    // materialized instantiations, whether partial or not
    field_instantiations: Vec<FieldInstantiation>,

    // function name to index into the Loader function list.
    // This allows a direct access from function name to `Function`
    function_map: HashMap<Identifier, usize>,
    // struct name to index into the Loader type list
    // This allows a direct access from struct name to `Struct`
    struct_map: HashMap<Identifier, usize>,
}

impl Module {
    fn new(
        module: CompiledModule,
        cache: &ModuleCache,
    ) -> Result<Self, (PartialVMError, CompiledModule)> {
        let id = module.self_id();

        let mut struct_refs = vec![];
        let mut structs = vec![];
        let mut struct_instantiations = vec![];
        let mut function_refs = vec![];
        let mut function_instantiations = vec![];
        let mut field_handles = vec![];
        let mut field_instantiations: Vec<FieldInstantiation> = vec![];
        let mut function_map = HashMap::new();
        let mut struct_map = HashMap::new();

        let mut create = || {
            for struct_handle in module.struct_handles() {
                let struct_name = module.identifier_at(struct_handle.name);
                let module_handle = module.module_handle_at(struct_handle.module);
                let module_id = module.module_id_for_handle(module_handle);
                if module_id == id {
                    // module has not been published yet, loop through the types in reverse order.
                    // At this point all the types of the module are in the type list but not yet
                    // exposed through the module cache. The implication is that any resolution
                    // to types of the module being loaded is going to fail.
                    // So we manually go through the types and find the proper index
                    for (idx, struct_type) in cache.structs.iter().enumerate().rev() {
                        if struct_type.module != module_id {
                            return Err(PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE)
                                .with_message(format!(
                                    "Cannot find {:?}::{:?} in publishing module",
                                    module_id, struct_name
                                )));
                        }
                        if struct_type.name.as_ident_str() == struct_name {
                            struct_refs.push(idx);
                            break;
                        }
                    }
                } else {
                    struct_refs.push(cache.resolve_struct_by_name(struct_name, &module_id)?.0);
                }
            }

            for struct_def in module.struct_defs() {
                let idx = struct_refs[struct_def.struct_handle.0 as usize];
                let field_count = cache.structs[idx].fields.len() as u16;
                structs.push(StructDef { idx, field_count });
                let name =
                    module.identifier_at(module.struct_handle_at(struct_def.struct_handle).name);
                struct_map.insert(name.to_owned(), idx);
            }

            for struct_inst in module.struct_instantiations() {
                let def = struct_inst.def.0 as usize;
                let struct_def = &structs[def];
                let field_count = struct_def.field_count;
                let mut instantiation = vec![];
                for ty in &module.signature_at(struct_inst.type_parameters).0 {
                    instantiation.push(cache.make_type_while_loading(&module, ty)?);
                }
                struct_instantiations.push(StructInstantiation {
                    field_count,
                    def: struct_def.idx,
                    instantiation,
                });
            }

            for func_handle in module.function_handles() {
                let func_name = module.identifier_at(func_handle.name);
                let module_handle = module.module_handle_at(func_handle.module);
                let module_id = module.module_id_for_handle(module_handle);
                if module_id == id {
                    // module has not been published yet, loop through the functions
                    for (idx, function) in cache.functions.iter().enumerate().rev() {
                        if function.module_id() != Some(&module_id) {
                            return Err(PartialVMError::new(
                                StatusCode::FUNCTION_RESOLUTION_FAILURE,
                            )
                            .with_message(format!(
                                "Cannot find {:?}::{:?} in publishing module",
                                module_id, func_name
                            )));
                        }
                        if function.name.as_ident_str() == func_name {
                            function_refs.push(idx);
                            break;
                        }
                    }
                } else {
                    function_refs.push(cache.resolve_function_by_name(func_name, &module_id)?);
                }
            }

            for func_def in module.function_defs() {
                let idx = function_refs[func_def.function.0 as usize];
                let name = module.identifier_at(module.function_handle_at(func_def.function).name);
                function_map.insert(name.to_owned(), idx);
            }

            for func_inst in module.function_instantiations() {
                let handle = function_refs[func_inst.handle.0 as usize];
                let mut instantiation = vec![];
                for ty in &module.signature_at(func_inst.type_parameters).0 {
                    instantiation.push(cache.make_type_while_loading(&module, ty)?);
                }
                function_instantiations.push(FunctionInstantiation {
                    handle,
                    instantiation,
                });
            }

            for f_handle in module.field_handles() {
                let def_idx = f_handle.owner;
                let owner = structs[def_idx.0 as usize].idx;
                let offset = f_handle.field as usize;
                field_handles.push(FieldHandle { owner, offset });
            }

            for f_inst in module.field_instantiations() {
                let fh_idx = f_inst.handle;
                let owner = field_handles[fh_idx.0 as usize].owner;
                let offset = field_handles[fh_idx.0 as usize].offset;
                field_instantiations.push(FieldInstantiation { owner, offset });
            }

            Ok(())
        };

        match create() {
            Ok(_) => Ok(Self {
                id,
                module,
                struct_refs,
                structs,
                function_refs,
                struct_instantiations,
                function_instantiations,
                field_handles,
                field_instantiations,
                function_map,
                struct_map,
            }),
            Err(err) => Err((err, module)),
        }
    }

    fn struct_at(&self, idx: StructDefinitionIndex) -> usize {
        self.structs[idx.0 as usize].idx
    }

    fn struct_instantiation_at(&self, idx: u16) -> &StructInstantiation {
        &self.struct_instantiations[idx as usize]
    }

    fn function_at(&self, idx: u16) -> usize {
        self.function_refs[idx as usize]
    }

    fn function_instantiation_at(&self, idx: u16) -> &FunctionInstantiation {
        &self.function_instantiations[idx as usize]
    }

    fn field_count(&self, idx: u16) -> u16 {
        self.structs[idx as usize].field_count
    }

    fn field_instantiation_count(&self, idx: u16) -> u16 {
        self.struct_instantiations[idx as usize].field_count
    }

    fn module(&self) -> &CompiledModule {
        &self.module
    }

    fn field_offset(&self, idx: FieldHandleIndex) -> usize {
        self.field_handles[idx.0 as usize].offset
    }

    fn field_instantiation_offset(&self, idx: FieldInstantiationIndex) -> usize {
        self.field_instantiations[idx.0 as usize].offset
    }
}

// A Script is very similar to a `CompiledScript` but data is "transformed" to a representation
// more appropriate to execution.
// When code executes, indexes in instructions are resolved against runtime structures
// (rather then "compiled") to make available data needed for execution
#[derive(Debug)]
struct Script {
    // primitive pools
    script: CompiledScript,

    // types as indexes into the Loader type list
    struct_refs: Vec<usize>,

    // functions as indexes into the Loader function list
    function_refs: Vec<usize>,
    // materialized instantiations, whether partial or not
    function_instantiations: Vec<FunctionInstantiation>,

    // entry point
    main: Arc<Function>,
}

impl Script {
    fn new(script: CompiledScript, script_hash: &HashValue, cache: &ModuleCache) -> VMResult<Self> {
        let mut struct_refs = vec![];
        for struct_handle in script.struct_handles() {
            let struct_name = script.identifier_at(struct_handle.name);
            let module_handle = script.module_handle_at(struct_handle.module);
            let module_id = ModuleId::new(
                *script.address_identifier_at(module_handle.address),
                script.identifier_at(module_handle.name).to_owned(),
            );
            struct_refs.push(
                cache
                    .resolve_struct_by_name(struct_name, &module_id)
                    .map_err(|e| e.finish(Location::Script))?
                    .0,
            );
        }

        let mut function_refs = vec![];
        for func_handle in script.function_handles().iter() {
            let func_name = script.identifier_at(func_handle.name);
            let module_handle = script.module_handle_at(func_handle.module);
            let module_id = ModuleId::new(
                *script.address_identifier_at(module_handle.address),
                script.identifier_at(module_handle.name).to_owned(),
            );
            let ref_idx = cache
                .resolve_function_by_name(func_name, &module_id)
                .map_err(|err| err.finish(Location::Undefined))?;
            function_refs.push(ref_idx);
        }

        let mut function_instantiations = vec![];
        let (_, module) = script.clone().into_module();
        for func_inst in script.function_instantiations() {
            let handle = function_refs[func_inst.handle.0 as usize];
            let mut instantiation = vec![];
            for ty in &script.signature_at(func_inst.type_parameters).0 {
                instantiation.push(
                    cache
                        .make_type(&module, ty)
                        .map_err(|e| e.finish(Location::Script))?,
                );
            }
            function_instantiations.push(FunctionInstantiation {
                handle,
                instantiation,
            });
        }

        let scope = Scope::Script(*script_hash);

        let compiled_script = script.as_inner();
        let code: Vec<Bytecode> = compiled_script.code.code.clone();
        let parameters = script.signature_at(compiled_script.parameters).clone();
        let return_ = Signature(vec![]);
        let locals = Signature(
            parameters
                .0
                .iter()
                .chain(script.signature_at(compiled_script.code.locals).0.iter())
                .cloned()
                .collect(),
        );
        let type_parameters = compiled_script.type_parameters.clone();
        // TODO: main does not have a name. Revisit.
        let name = Identifier::new("main").unwrap();
        let native = None; // Script entries cannot be native
        let main: Arc<Function> = Arc::new(Function {
            index: FunctionDefinitionIndex(0),
            code,
            parameters,
            return_,
            locals,
            type_parameters,
            native,
            scope,
            name,
        });

        Ok(Self {
            script,
            struct_refs,
            function_refs,
            function_instantiations,
            main,
        })
    }

    fn entry_point(&self) -> Arc<Function> {
        self.main.clone()
    }

    fn function_at(&self, idx: u16) -> usize {
        self.function_refs[idx as usize]
    }

    fn function_instantiation_at(&self, idx: u16) -> &FunctionInstantiation {
        &self.function_instantiations[idx as usize]
    }
}

// A simple wrapper for the "owner" of the function (Module or Script)
#[derive(Debug)]
enum Scope {
    Module(ModuleId),
    Script(HashValue),
}

// A runtime function
#[derive(Debug)]
pub(crate) struct Function {
    index: FunctionDefinitionIndex,
    code: Vec<Bytecode>,
    parameters: Signature,
    return_: Signature,
    locals: Signature,
    type_parameters: Vec<Kind>,
    native: Option<NativeFunction>,
    scope: Scope,
    name: Identifier,
}

impl Function {
    fn new(
        index: FunctionDefinitionIndex,
        def: &FunctionDefinition,
        module: &CompiledModule,
    ) -> Self {
        let handle = module.function_handle_at(def.function);
        let name = module.identifier_at(handle.name).to_owned();
        let module_id = module.self_id();
        let native = if def.is_native() {
            NativeFunction::resolve(
                module_id.address(),
                module_id.name().as_str(),
                name.as_str(),
            )
        } else {
            None
        };
        let scope = Scope::Module(module_id);
        let parameters = module.signature_at(handle.parameters).clone();
        // Native functions do not have a code unit
        let (code, locals) = match &def.code {
            Some(code) => (
                code.code.clone(),
                Signature(
                    parameters
                        .0
                        .iter()
                        .chain(module.signature_at(code.locals).0.iter())
                        .cloned()
                        .collect(),
                ),
            ),
            None => (vec![], Signature(vec![])),
        };
        let return_ = module.signature_at(handle.return_).clone();
        let type_parameters = handle.type_parameters.clone();
        Self {
            index,
            code,
            parameters,
            return_,
            locals,
            type_parameters,
            native,
            scope,
            name,
        }
    }

    pub(crate) fn module_id(&self) -> Option<&ModuleId> {
        match &self.scope {
            Scope::Module(module_id) => Some(module_id),
            Scope::Script(_) => None,
        }
    }

    pub(crate) fn index(&self) -> FunctionDefinitionIndex {
        self.index
    }

    pub(crate) fn get_resolver<'a>(&self, loader: &'a Loader) -> Resolver<'a> {
        match &self.scope {
            Scope::Module(module_id) => {
                let module = loader.get_module(module_id);
                Resolver::for_module(loader, module)
            }
            Scope::Script(script_hash) => {
                let script = loader.get_script(script_hash);
                Resolver::for_script(loader, script)
            }
        }
    }

    pub(crate) fn local_count(&self) -> usize {
        self.locals.len()
    }

    pub(crate) fn arg_count(&self) -> usize {
        self.parameters.len()
    }

    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(crate) fn code(&self) -> &[Bytecode] {
        &self.code
    }

    pub(crate) fn type_parameters(&self) -> &[Kind] {
        &self.type_parameters
    }

    pub(crate) fn parameters(&self) -> &Signature {
        &self.parameters
    }

    pub(crate) fn pretty_string(&self) -> String {
        match &self.scope {
            Scope::Script(_) => "Script::main".into(),
            Scope::Module(id) => format!(
                "0x{}::{}::{}",
                id.address(),
                id.name().as_str(),
                self.name.as_str()
            ),
        }
    }

    pub(crate) fn is_native(&self) -> bool {
        self.native.is_some()
    }

    pub(crate) fn get_native(&self) -> PartialVMResult<NativeFunction> {
        self.native.ok_or_else(|| {
            PartialVMError::new(StatusCode::UNREACHABLE)
                .with_message("Missing Native Function".to_string())
        })
    }
}

//
// Internal structures that are saved at the proper index in the proper tables to access
// execution information (interpreter).
// The following structs are internal to the loader and never exposed out.
// The `Loader` will create those struct and the proper table when loading a module.
// The `Resolver` uses those structs to return information to the `Interpreter`.
//

// A function instantiation.
#[derive(Debug)]
struct FunctionInstantiation {
    // index to `ModuleCache::functions` global table
    handle: usize,
    instantiation: Vec<Type>,
}

#[derive(Debug)]
struct StructDef {
    // struct field count
    field_count: u16,
    // `ModuelCache::structs` global table index
    idx: usize,
}

#[derive(Debug)]
struct StructInstantiation {
    // struct field count
    field_count: u16,
    // `ModuelCache::structs` global table index. It is the generic type.
    def: usize,
    instantiation: Vec<Type>,
}

// A field handle. The offset is the only used information when operating on a field
#[derive(Debug)]
struct FieldHandle {
    offset: usize,
    // `ModuelCache::structs` global table index. It is the generic type.
    owner: usize,
}

// A field instantiation. The offset is the only used information when operating on a field
#[derive(Debug)]
struct FieldInstantiation {
    offset: usize,
    // `ModuelCache::structs` global table index. It is the generic type.
    owner: usize,
}

//
// Utility functions
//

fn script_dependencies(script: &CompiledScript) -> Vec<ModuleId> {
    let mut deps = vec![];
    for module in script.module_handles() {
        deps.push(ModuleId::new(
            *script.address_identifier_at(module.address),
            script.identifier_at(module.name).to_owned(),
        ));
    }
    deps
}

fn module_dependencies(module: &CompiledModule) -> Vec<ModuleId> {
    let self_module = module.self_handle();
    let mut deps = vec![];
    for module_handle in module.module_handles() {
        if module_handle == self_module {
            continue;
        }
        deps.push(ModuleId::new(
            *module.address_identifier_at(module_handle.address),
            module.identifier_at(module_handle.name).to_owned(),
        ));
    }
    deps
}

fn expect_no_verification_errors(err: VMError, log_context: &impl LogContext) -> VMError {
    match err.status_type() {
        status_type @ StatusType::Deserialization | status_type @ StatusType::Verification => {
            let message = format!(
                "Unexpected verifier/deserialization error! This likely means there is code \
                stored on chain that is unverifiable!\nError: {:?}",
                &err
            );
            let (_old_status, _old_sub_status, _old_message, location, indices, offsets) =
                err.all_data();
            let major_status = match status_type {
                StatusType::Deserialization => StatusCode::UNEXPECTED_DESERIALIZATION_ERROR,
                StatusType::Verification => StatusCode::UNEXPECTED_VERIFIER_ERROR,
                _ => unreachable!(),
            };

            log_context.alert();
            error!(*log_context, "[VM] {}", message);
            PartialVMError::new(major_status)
                .with_message(message)
                .at_indices(indices)
                .at_code_offsets(offsets)
                .finish(location)
        }
        _ => err,
    }
}

//
// Cache for data associated to a Struct, used for de/serialization and more
//

struct StructInfo {
    struct_tag: Option<StructTag>,
    struct_layout: Option<MoveStructLayout>,
    kind_info: Option<(MoveKind, Vec<MoveKindInfo>)>,
}

impl StructInfo {
    fn new() -> Self {
        Self {
            struct_tag: None,
            struct_layout: None,
            kind_info: None,
        }
    }
}

pub(crate) struct TypeCache {
    structs: HashMap<usize, HashMap<Vec<Type>, StructInfo>>,
}

impl TypeCache {
    fn new() -> Self {
        Self {
            structs: HashMap::new(),
        }
    }
}

const VALUE_DEPTH_MAX: usize = 256;

impl Loader {
    fn struct_gidx_to_type_tag(&self, gidx: usize, ty_args: &[Type]) -> PartialVMResult<StructTag> {
        if let Some(struct_map) = self.type_cache.lock().structs.get(&gidx) {
            if let Some(struct_info) = struct_map.get(ty_args) {
                if let Some(struct_tag) = &struct_info.struct_tag {
                    return Ok(struct_tag.clone());
                }
            }
        }

        let ty_arg_tags = ty_args
            .iter()
            .map(|ty| self.type_to_type_tag(ty))
            .collect::<PartialVMResult<Vec<_>>>()?;
        let struct_type = self.module_cache.lock().struct_at(gidx);
        let struct_tag = StructTag {
            address: *struct_type.module.address(),
            module: struct_type.module.name().to_owned(),
            name: struct_type.name.clone(),
            type_params: ty_arg_tags,
        };

        self.type_cache
            .lock()
            .structs
            .entry(gidx)
            .or_insert_with(HashMap::new)
            .entry(ty_args.to_vec())
            .or_insert_with(StructInfo::new)
            .struct_tag = Some(struct_tag.clone());

        Ok(struct_tag)
    }

    fn type_to_type_tag_impl(&self, ty: &Type) -> PartialVMResult<TypeTag> {
        Ok(match ty {
            Type::Bool => TypeTag::Bool,
            Type::U8 => TypeTag::U8,
            Type::U64 => TypeTag::U64,
            Type::U128 => TypeTag::U128,
            Type::Address => TypeTag::Address,
            Type::Signer => TypeTag::Signer,
            Type::Vector(ty) => TypeTag::Vector(Box::new(self.type_to_type_tag(ty)?)),
            Type::Struct(gidx) => TypeTag::Struct(self.struct_gidx_to_type_tag(*gidx, &[])?),
            Type::StructInstantiation(gidx, ty_args) => {
                TypeTag::Struct(self.struct_gidx_to_type_tag(*gidx, ty_args)?)
            }
            Type::Reference(_) | Type::MutableReference(_) | Type::TyParam(_) => {
                return Err(
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!("no type tag for {:?}", ty)),
                )
            }
        })
    }

    fn struct_gidx_to_type_layout(
        &self,
        gidx: usize,
        ty_args: &[Type],
        depth: usize,
    ) -> PartialVMResult<MoveStructLayout> {
        if let Some(struct_map) = self.type_cache.lock().structs.get(&gidx) {
            if let Some(struct_info) = struct_map.get(ty_args) {
                if let Some(layout) = &struct_info.struct_layout {
                    return Ok(layout.clone());
                }
            }
        }

        let struct_type = self.module_cache.lock().struct_at(gidx);
        let field_tys = struct_type
            .fields
            .iter()
            .map(|ty| ty.subst(ty_args))
            .collect::<PartialVMResult<Vec<_>>>()?;
        let field_layouts = field_tys
            .iter()
            .map(|ty| self.type_to_type_layout_impl(ty, depth + 1))
            .collect::<PartialVMResult<Vec<_>>>()?;
        let struct_layout = MoveStructLayout::new(field_layouts);

        self.type_cache
            .lock()
            .structs
            .entry(gidx)
            .or_insert_with(HashMap::new)
            .entry(ty_args.to_vec())
            .or_insert_with(StructInfo::new)
            .struct_layout = Some(struct_layout.clone());

        Ok(struct_layout)
    }

    fn type_to_type_layout_impl(&self, ty: &Type, depth: usize) -> PartialVMResult<MoveTypeLayout> {
        if depth > VALUE_DEPTH_MAX {
            return Err(PartialVMError::new(StatusCode::VM_MAX_VALUE_DEPTH_REACHED));
        }
        Ok(match ty {
            Type::Bool => MoveTypeLayout::Bool,
            Type::U8 => MoveTypeLayout::U8,
            Type::U64 => MoveTypeLayout::U64,
            Type::U128 => MoveTypeLayout::U128,
            Type::Address => MoveTypeLayout::Address,
            Type::Signer => MoveTypeLayout::Signer,
            Type::Vector(ty) => {
                MoveTypeLayout::Vector(Box::new(self.type_to_type_layout_impl(ty, depth + 1)?))
            }
            Type::Struct(gidx) => {
                MoveTypeLayout::Struct(self.struct_gidx_to_type_layout(*gidx, &[], depth)?)
            }
            Type::StructInstantiation(gidx, ty_args) => {
                MoveTypeLayout::Struct(self.struct_gidx_to_type_layout(*gidx, ty_args, depth)?)
            }
            Type::Reference(_) | Type::MutableReference(_) | Type::TyParam(_) => {
                return Err(
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!("no type layout for {:?}", ty)),
                )
            }
        })
    }

    fn struct_gidx_to_kind_info(
        &self,
        gidx: usize,
        ty_args: &[Type],
        depth: usize,
    ) -> PartialVMResult<(MoveKind, Vec<MoveKindInfo>)> {
        if let Some(struct_map) = self.type_cache.lock().structs.get(&gidx) {
            if let Some(struct_info) = struct_map.get(ty_args) {
                if let Some(kind_info) = &struct_info.kind_info {
                    return Ok(kind_info.clone());
                }
            }
        }

        let struct_type = self.module_cache.lock().struct_at(gidx);

        let mut is_resource = struct_type.is_resource;
        if !is_resource {
            for ty in ty_args {
                if self.is_resource(ty) {
                    is_resource = true;
                }
            }
        }
        let field_tys = struct_type
            .fields
            .iter()
            .map(|ty| ty.subst(ty_args))
            .collect::<PartialVMResult<Vec<_>>>()?;
        let field_kind_info = field_tys
            .iter()
            .map(|ty| self.type_to_kind_info_impl(ty, depth + 1))
            .collect::<PartialVMResult<Vec<_>>>()?;
        let kind_info = (MoveKind::from_bool(is_resource), field_kind_info);

        self.type_cache
            .lock()
            .structs
            .entry(gidx)
            .or_insert_with(HashMap::new)
            .entry(ty_args.to_vec())
            .or_insert_with(StructInfo::new)
            .kind_info = Some(kind_info.clone());

        Ok(kind_info)
    }

    pub(crate) fn type_to_kind_info_impl(
        &self,
        ty: &Type,
        depth: usize,
    ) -> PartialVMResult<MoveKindInfo> {
        if depth > VALUE_DEPTH_MAX {
            return Err(PartialVMError::new(StatusCode::VM_MAX_VALUE_DEPTH_REACHED));
        }
        Ok(match ty {
            Type::Bool | Type::U8 | Type::U64 | Type::U128 | Type::Address => {
                MoveKindInfo::Base(MoveKind::Copyable)
            }
            Type::Signer => MoveKindInfo::Base(MoveKind::Resource),
            Type::Vector(ty) => {
                let kind_info = self.type_to_kind_info_impl(ty, depth + 1)?;
                MoveKindInfo::Vector(kind_info.kind(), Box::new(kind_info))
            }
            Type::Struct(gidx) => {
                let (is_resource, field_kind_info) =
                    self.struct_gidx_to_kind_info(*gidx, &[], depth)?;
                MoveKindInfo::Struct(is_resource, field_kind_info)
            }
            Type::StructInstantiation(gidx, ty_args) => {
                let (is_resource, field_kind_info) =
                    self.struct_gidx_to_kind_info(*gidx, ty_args, depth)?;
                MoveKindInfo::Struct(is_resource, field_kind_info)
            }
            Type::Reference(_) | Type::MutableReference(_) | Type::TyParam(_) => {
                return Err(
                    PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                        .with_message(format!("no kind info for {:?}", ty)),
                )
            }
        })
    }

    pub(crate) fn type_to_type_tag(&self, ty: &Type) -> PartialVMResult<TypeTag> {
        self.type_to_type_tag_impl(ty)
    }
    pub(crate) fn type_to_type_layout(&self, ty: &Type) -> PartialVMResult<MoveTypeLayout> {
        self.type_to_type_layout_impl(ty, 1)
    }
    pub(crate) fn type_to_kind_info(&self, ty: &Type) -> PartialVMResult<MoveKindInfo> {
        self.type_to_kind_info_impl(ty, 1)
    }
}
