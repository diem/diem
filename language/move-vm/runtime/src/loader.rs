// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::native_functions::NativeFunction;
use bytecode_verifier::{
    constants, instantiation_loops::InstantiationLoopChecker, verify_main_signature,
    CodeUnitVerifier, DependencyChecker, DuplicationChecker, InstructionConsistency,
    RecursiveStructDefChecker, ResourceTransitiveChecker, SignatureChecker,
};
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use move_core_types::{
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag, TypeTag},
    value::{MoveKind, MoveKindInfo, MoveStructLayout, MoveTypeLayout},
    vm_status::StatusCode,
};
use move_vm_types::{
    data_store::DataStore,
    loaded_data::{
        runtime_types::{StructType, Type, TypeConverter},
        types::{FatStructType, FatType},
    },
};
use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Mutex},
};
use vm::{
    access::{ModuleAccess, ScriptAccess},
    errors::{verification_error, Location, PartialVMError, PartialVMResult, VMResult},
    file_format::{
        Bytecode, CompiledModule, CompiledScript, Constant, ConstantPoolIndex, FieldHandleIndex,
        FieldInstantiationIndex, FunctionDefinition, FunctionHandleIndex,
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
// A ModuleCache is accessed under lock.
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

    fn get(&self, id: &ModuleId) -> Option<Arc<Module>> {
        self.modules.get(id).map(|module| Arc::clone(module))
    }

    fn insert(&mut self, id: ModuleId, module: CompiledModule) -> VMResult<Arc<Module>> {
        self.add_module(&module)?;
        let module = Module::new(module, self)?;
        match self.get(&id) {
            Some(module) => Ok(module),
            None => Ok(Arc::clone(self.modules.insert(id, module))),
        }
    }

    fn function_at(&self, idx: usize) -> Arc<Function> {
        self.functions[idx].clone()
    }

    fn struct_at(&self, idx: usize) -> Arc<StructType> {
        Arc::clone(&self.structs[idx])
    }

    fn add_module(&mut self, module: &CompiledModule) -> VMResult<()> {
        let starting_idx = self.structs.len();
        for (idx, struct_def) in module.struct_defs().iter().enumerate() {
            let st = self.load_type(module, struct_def, StructDefinitionIndex(idx as u16))?;
            self.structs.push(Arc::new(st));
        }
        self.load_fields(module, starting_idx)
            .map_err(|e| e.finish(Location::Undefined))?;

        for func in module.function_defs() {
            let function = self.load_function(module, func)?;
            self.functions.push(Arc::new(function));
        }
        Ok(())
    }

    fn load_type(
        &self,
        module: &CompiledModule,
        struct_def: &StructDefinition,
        idx: StructDefinitionIndex,
    ) -> VMResult<StructType> {
        let struct_handle = module.struct_handle_at(struct_def.struct_handle);
        let is_resource = struct_handle.is_nominal_resource;
        let name = module.identifier_at(struct_handle.name).to_owned();
        let type_parameters = struct_handle.type_parameters.clone();
        let module = module.self_id();
        Ok(StructType {
            fields: vec![],
            is_resource,
            type_parameters,
            name,
            module,
            struct_def: idx,
        })
    }

    fn load_fields(&mut self, module: &CompiledModule, starting_idx: usize) -> PartialVMResult<()> {
        let mut field_types = vec![];
        for struct_def in module.struct_defs() {
            let fields = match &struct_def.field_information {
                StructFieldInformation::Native => unreachable!("native structs have been removed"),
                StructFieldInformation::Declared(fields) => fields,
            };

            let mut field_tys = vec![];
            for field in fields {
                let ty = self.make_type(module, &field.signature.0)?;
                assume!(field_tys.len() < usize::max_value());
                field_tys.push(ty);
            }

            field_types.push(field_tys);
        }
        for (fields, arc_struct_type) in field_types
            .into_iter()
            .zip(&mut self.structs[starting_idx..])
        {
            match Arc::get_mut(arc_struct_type) {
                None => {
                    return Err(PartialVMError::new(StatusCode::INVALID_CODE_CACHE)
                        .with_message("Arc Type should not have any reference".to_string()))
                }
                Some(struct_type) => struct_type.fields = fields,
            }
        }
        Ok(())
    }

    fn make_type(&self, module: &CompiledModule, tok: &SignatureToken) -> PartialVMResult<Type> {
        let res = match tok {
            SignatureToken::Bool => Type::Bool,
            SignatureToken::U8 => Type::U8,
            SignatureToken::U64 => Type::U64,
            SignatureToken::U128 => Type::U128,
            SignatureToken::Address => Type::Address,
            SignatureToken::Signer => Type::Signer,
            SignatureToken::TypeParameter(idx) => Type::TyParam(*idx as usize),
            SignatureToken::Vector(inner_tok) => {
                let inner_type = self.make_type(module, inner_tok)?;
                Type::Vector(Box::new(inner_type))
            }
            SignatureToken::Reference(inner_tok) => {
                let inner_type = self.make_type(module, inner_tok)?;
                Type::Reference(Box::new(inner_type))
            }
            SignatureToken::MutableReference(inner_tok) => {
                let inner_type = self.make_type(module, inner_tok)?;
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
                let def_idx = self.find_struct_by_name(struct_name, &module_id)?.0;
                Type::Struct(def_idx)
            }
            SignatureToken::StructInstantiation(sh_idx, tys) => {
                let type_parameters: Vec<_> = tys
                    .iter()
                    .map(|tok| self.make_type(module, tok))
                    .collect::<PartialVMResult<_>>()?;
                let struct_handle = module.struct_handle_at(*sh_idx);
                let struct_name = module.identifier_at(struct_handle.name);
                let module_handle = module.module_handle_at(struct_handle.module);
                let module_id = ModuleId::new(
                    *module.address_identifier_at(module_handle.address),
                    module.identifier_at(module_handle.name).to_owned(),
                );
                let def_idx = self.find_struct_by_name(struct_name, &module_id)?.0;
                Type::StructInstantiation(def_idx, type_parameters)
            }
        };
        Ok(res)
    }

    fn find_struct_by_name(
        &self,
        struct_name: &IdentStr,
        module_id: &ModuleId,
    ) -> PartialVMResult<(usize, Arc<StructType>)> {
        for (idx, ty) in self.structs.iter().enumerate() {
            if struct_match(ty, &module_id, struct_name) {
                return Ok((idx, Arc::clone(ty)));
            }
        }
        Err(
            PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(format!(
                "Cannot find {:?}::{:?} in cache",
                module_id, struct_name
            )),
        )
    }

    fn resolve_function_handle(
        &self,
        func_name: &IdentStr,
        module_id: &ModuleId,
    ) -> VMResult<usize> {
        for (idx, f) in self.functions.iter().enumerate() {
            if function_match(&f, module_id, func_name) {
                return Ok(idx);
            }
        }
        Err(PartialVMError::new(StatusCode::FUNCTION_RESOLUTION_FAILURE)
            .with_message(format!(
                "Cannot find {:?}::{:?} in cache",
                module_id, func_name
            ))
            .finish(Location::Undefined))
    }

    fn load_function(
        &self,
        module: &CompiledModule,
        func_def: &FunctionDefinition,
    ) -> VMResult<Function> {
        Ok(Function::new(func_def, module))
    }
}

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
    ) -> VMResult<(Arc<Function>, Vec<Type>)> {
        // retrieve or load the script
        let hash_value = HashValue::sha3_256_of(script_blob);
        let opt_main = self.scripts.lock().unwrap().get(&hash_value);
        let main = match opt_main {
            Some(main) => main,
            None => {
                let ver_script = self.deserialize_and_verify_script(script_blob, data_store)?;
                let script =
                    Script::new(ver_script, &hash_value, &self.module_cache.lock().unwrap())?;
                self.scripts
                    .lock()
                    .unwrap()
                    .insert(hash_value, script)
                    .map_err(|e| e.finish(Location::Script))?
            }
        };

        // verify type arguments
        let mut type_params = vec![];
        for ty in ty_args {
            type_params.push(self.load_type(ty, data_store)?);
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
    ) -> VMResult<CompiledScript> {
        let script = match CompiledScript::deserialize(script) {
            Ok(script) => script,
            Err(err) => {
                error!("[VM] deserializer for script returned error: {:?}", err);
                let msg = format!("Deserialization error: {:?}", err);
                return Err(PartialVMError::new(StatusCode::CODE_DESERIALIZATION_ERROR)
                    .with_message(msg)
                    .finish(Location::Script));
            }
        };

        match self.verify_script(&script) {
            Ok(_) => {
                // verify dependencies
                let deps = load_script_dependencies(&script);
                let mut dependencies = vec![];
                for dep in &deps {
                    dependencies.push(self.load_module(dep, data_store)?);
                }
                self.verify_script_dependencies(script, dependencies)
            }
            Err(err) => {
                error!(
                    "[VM] bytecode verifier returned errors for script: {:?}",
                    err
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
        script: CompiledScript,
        dependencies: Vec<Arc<Module>>,
    ) -> VMResult<CompiledScript> {
        let mut deps = vec![];
        for dep in &dependencies {
            deps.push(dep.module());
        }
        DependencyChecker::verify_script(&script, deps).and_then(|_| Ok(script))
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
    ) -> VMResult<(Arc<Function>, Vec<Type>)> {
        self.load_module(module_id, data_store)?;
        let idx = self
            .module_cache
            .lock()
            .unwrap()
            .resolve_function_handle(function_name, module_id)?;
        let func = self.module_cache.lock().unwrap().function_at(idx);

        // verify type arguments
        let mut type_params = vec![];
        for ty in ty_args {
            type_params.push(self.load_type(ty, data_store)?);
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
    pub(crate) fn verify_module(&self, module: &CompiledModule) -> VMResult<()> {
        DuplicationChecker::verify_module(&module)?;
        SignatureChecker::verify_module(&module)?;
        InstructionConsistency::verify_module(&module)?;
        ResourceTransitiveChecker::verify_module(&module)?;
        constants::verify_module(&module)?;
        RecursiveStructDefChecker::verify_module(&module)?;
        InstantiationLoopChecker::verify_module(&module)?;
        CodeUnitVerifier::verify_module(&module)?;
        Self::check_natives(&module)
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
    fn deserialize_and_verify_module(
        &self,
        id: &ModuleId,
        data_store: &mut impl DataStore,
    ) -> VMResult<CompiledModule> {
        let module = match data_store.load_module(id) {
            Ok(m) => m,
            Err(err) => {
                crit!("[VM] Error fetching module with id {:?}", id);
                return Err(err);
            }
        };
        self.verify_module(&module)?;
        self.check_dependencies(&module, data_store)?;
        Ok(module)
    }

    fn check_dependencies(
        &self,
        module: &CompiledModule,
        data_store: &mut impl DataStore,
    ) -> VMResult<()> {
        let deps = load_module_dependencies(module);
        let mut dependencies = vec![];
        for dep in &deps {
            dependencies.push(self.load_module(dep, data_store)?);
        }
        self.verify_module_dependencies(module, dependencies)
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

    fn load_type(&self, type_tag: &TypeTag, data_store: &mut impl DataStore) -> VMResult<Type> {
        Ok(match type_tag {
            TypeTag::Bool => Type::Bool,
            TypeTag::U8 => Type::U8,
            TypeTag::U64 => Type::U64,
            TypeTag::U128 => Type::U128,
            TypeTag::Address => Type::Address,
            TypeTag::Signer => Type::Signer,
            TypeTag::Vector(tt) => Type::Vector(Box::new(self.load_type(tt, data_store)?)),
            TypeTag::Struct(struct_tag) => {
                let module_id = ModuleId::new(struct_tag.address, struct_tag.module.clone());
                self.load_module(&module_id, data_store)?;
                let (idx, struct_type) = self
                    .module_cache
                    .lock()
                    .unwrap()
                    .find_struct_by_name(&struct_tag.name, &module_id)
                    .map_err(|e| e.finish(Location::Undefined))?;
                if struct_type.type_parameters.is_empty() && struct_tag.type_params.is_empty() {
                    Type::Struct(idx)
                } else {
                    let mut type_params = vec![];
                    for ty_param in &struct_tag.type_params {
                        type_params.push(self.load_type(ty_param, data_store)?);
                    }
                    self.verify_ty_args(&struct_type.type_parameters, &type_params)
                        .map_err(|e| e.finish(Location::Undefined))?;
                    Type::StructInstantiation(idx, type_params)
                }
            }
        })
    }

    fn load_module(&self, id: &ModuleId, data_store: &mut impl DataStore) -> VMResult<Arc<Module>> {
        if let Some(module) = self.module_cache.lock().unwrap().get(id) {
            return Ok(module);
        }
        let module = self.deserialize_and_verify_module(id, data_store)?;
        self.module_cache.lock().unwrap().insert(id.clone(), module)
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
            let k = if self.is_resource(ty)? {
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
        self.module_cache.lock().unwrap().function_at(idx)
    }

    fn get_module(&self, idx: &ModuleId) -> Arc<Module> {
        Arc::clone(
            self.module_cache
                .lock()
                .unwrap()
                .modules
                .get(idx)
                .expect("ModuleId on Function must exist"),
        )
    }

    fn get_script(&self, hash: &HashValue) -> Arc<Script> {
        Arc::clone(
            self.scripts
                .lock()
                .unwrap()
                .scripts
                .get(hash)
                .expect("Script hash on Function must exist"),
        )
    }

    fn struct_at(&self, idx: usize) -> Arc<StructType> {
        self.module_cache.lock().unwrap().struct_at(idx)
    }

    fn is_resource(&self, type_: &Type) -> PartialVMResult<bool> {
        match type_ {
            Type::Struct(idx) => Ok(self
                .module_cache
                .lock()
                .unwrap()
                .struct_at(*idx)
                .is_resource),
            Type::StructInstantiation(idx, instantiation) => {
                if self
                    .module_cache
                    .lock()
                    .unwrap()
                    .struct_at(*idx)
                    .is_resource
                {
                    Ok(true)
                } else {
                    for ty in instantiation {
                        if self.is_resource(ty)? {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }
            }
            Type::Vector(ty) => self.is_resource(ty),
            _ => Ok(false),
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

    pub(crate) fn function_from_generic(&self, idx: FunctionInstantiationIndex) -> Arc<Function> {
        let func_inst = match &self.binary {
            BinaryType::Module(module) => module.function_instantiation_at(idx.0),
            BinaryType::Script(script) => script.function_instantiation_at(idx.0),
        };
        self.loader.function_at(func_inst.handle)
    }

    pub(crate) fn materialize_generic_function(
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

    pub(crate) fn get_struct_instantiation_type(
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
                if self.is_resource(&ty.subst(instantiation)?)? {
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

    pub fn type_to_fat_type(&self, ty: &Type) -> PartialVMResult<FatType> {
        self.loader.type_to_fat_type(ty)
    }

    pub(crate) fn is_resource(&self, ty: &Type) -> PartialVMResult<bool> {
        self.loader.is_resource(ty)
    }

    pub(crate) fn make_fat_type(
        &self,
        token: &SignatureToken,
        type_context: &[Type],
    ) -> PartialVMResult<FatType> {
        match &self.binary {
            BinaryType::Module(module) => {
                let binary = &module.module;
                let ty = self
                    .loader
                    .module_cache
                    .lock()
                    .unwrap()
                    .make_type(binary, token)?
                    .subst(type_context)?;
                self.loader.type_to_fat_type(&ty)
            }
            // TODO: this may not be true at all when it comes to printing (locals for instance)
            BinaryType::Script(_) => unreachable!("Scripts cannot have type operations"),
        }
    }
}

// This is the unfortunate side effect of our type story. It will have to go soon...
impl<'a> TypeConverter for Resolver<'a> {
    fn type_to_fat_type(&self, ty: &Type) -> PartialVMResult<FatType> {
        Resolver::type_to_fat_type(self, ty)
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

    // types as indexes into the Loader type list
    struct_refs: Vec<usize>,
    structs: Vec<StructDef>,
    struct_instantiations: Vec<StructInstantiation>,

    // functions as indexes into the Loader function list
    function_refs: Vec<usize>,
    // materialized instantiations, whether partial or not
    function_instantiations: Vec<FunctionInstantiation>,

    // fields as a pair of index, first to the type, second to the field position in that type
    field_handles: Vec<FieldHandle>,
    // materialized instantiations, whether partial or not
    field_instantiations: Vec<FieldInstantiation>,
}

impl Module {
    fn new(module: CompiledModule, cache: &ModuleCache) -> VMResult<Self> {
        let id = module.self_id();

        let mut struct_refs = vec![];
        for struct_handle in module.struct_handles() {
            let struct_name = module.identifier_at(struct_handle.name);
            let module_handle = module.module_handle_at(struct_handle.module);
            let module_id = module.module_id_for_handle(module_handle);
            struct_refs.push(
                cache
                    .find_struct_by_name(struct_name, &module_id)
                    .map_err(|e| e.finish(Location::Module(module.self_id())))?
                    .0,
            );
        }

        let mut structs = vec![];
        for struct_def in module.struct_defs() {
            let idx = struct_refs[struct_def.struct_handle.0 as usize];
            let field_count = cache.structs[idx].fields.len() as u16;
            structs.push(StructDef { idx, field_count });
        }

        let mut struct_instantiations = vec![];
        for struct_inst in module.struct_instantiations() {
            let def = struct_inst.def.0 as usize;
            let struct_def = &structs[def];
            let field_count = struct_def.field_count;
            let mut instantiation = vec![];
            for ty in &module.signature_at(struct_inst.type_parameters).0 {
                instantiation.push(
                    cache
                        .make_type(&module, ty)
                        .map_err(|e| e.finish(Location::Module(module.self_id())))?,
                );
            }
            struct_instantiations.push(StructInstantiation {
                field_count,
                def: struct_def.idx,
                instantiation,
            });
        }

        let mut function_refs = vec![];
        for func_handle in module.function_handles() {
            let func_name = module.identifier_at(func_handle.name);
            let module_handle = module.module_handle_at(func_handle.module);
            let module_id = module.module_id_for_handle(module_handle);
            let ref_idx = cache.resolve_function_handle(func_name, &module_id)?;
            function_refs.push(ref_idx);
        }

        let mut function_instantiations = vec![];
        for func_inst in module.function_instantiations() {
            let handle = function_refs[func_inst.handle.0 as usize];
            let mut instantiation = vec![];
            for ty in &module.signature_at(func_inst.type_parameters).0 {
                instantiation.push(
                    cache
                        .make_type(&module, ty)
                        .map_err(|e| e.finish(Location::Module(module.self_id())))?,
                );
            }
            function_instantiations.push(FunctionInstantiation {
                handle,
                instantiation,
            });
        }

        let mut field_handles = vec![];
        for f_handle in module.field_handles() {
            let def_idx = f_handle.owner;
            let owner = structs[def_idx.0 as usize].idx;
            let offset = f_handle.field as usize;
            field_handles.push(FieldHandle { owner, offset });
        }

        let mut field_instantiations: Vec<FieldInstantiation> = vec![];
        for f_inst in module.field_instantiations() {
            let fh_idx = f_inst.handle;
            let owner = field_handles[fh_idx.0 as usize].owner;
            let offset = field_handles[fh_idx.0 as usize].offset;
            field_instantiations.push(FieldInstantiation { owner, offset });
        }

        Ok(Self {
            id,
            module,
            struct_refs,
            structs,
            function_refs,
            struct_instantiations,
            function_instantiations,
            field_handles,
            field_instantiations,
        })
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

// A Script is very similar to a binary Script but data is "transformed" to a representation
// more appropriate to execution.
// When code executes indexes in instructions are resolved against those runtime structure
// so that any data needed for execution is immediately available
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
                    .find_struct_by_name(struct_name, &module_id)
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
            let ref_idx = cache.resolve_function_handle(func_name, &module_id)?;
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
    fn new(def: &FunctionDefinition, module: &CompiledModule) -> Self {
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

    pub(crate) fn locals(&self) -> &Signature {
        &self.locals
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
// Internal structures that are saved at the proper index in proper tables to access
// execution information (interpreter).
// The following structs are internal to the loader and never exposed out.
// The `Loader` will create those struct and the proper table when loading a module.
// THe `Resolver` uses those structs to return information to the `Interpreter`.
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

fn load_script_dependencies(script: &CompiledScript) -> Vec<ModuleId> {
    let mut deps = vec![];
    for module in script.module_handles() {
        deps.push(ModuleId::new(
            *script.address_identifier_at(module.address),
            script.identifier_at(module.name).to_owned(),
        ));
    }
    deps
}

fn load_module_dependencies(module: &CompiledModule) -> Vec<ModuleId> {
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

fn struct_match(struct_: &StructType, module: &ModuleId, name: &IdentStr) -> bool {
    struct_.name.as_ident_str() == name && &struct_.module == module
}

fn function_match(function: &Function, module: &ModuleId, name: &IdentStr) -> bool {
    function.name.as_ident_str() == name && function.module_id() == Some(module)
}

impl Loader {
    fn type_to_fat_type(&self, ty: &Type) -> PartialVMResult<FatType> {
        use Type::*;

        Ok(match ty {
            Bool => FatType::Bool,
            U8 => FatType::U8,
            U64 => FatType::U64,
            U128 => FatType::U128,
            Address => FatType::Address,
            Signer => FatType::Signer,
            Vector(ty) => FatType::Vector(Box::new(self.type_to_fat_type(ty)?)),
            Struct(idx) => FatType::Struct(Box::new(self.struct_to_fat_struct(*idx, vec![])?)),
            StructInstantiation(idx, instantiation) => {
                let mut ty_args = vec![];
                for inst in instantiation {
                    ty_args.push(self.type_to_fat_type(inst)?);
                }
                FatType::Struct(Box::new(self.struct_to_fat_struct(*idx, ty_args)?))
            }
            Reference(ty) => FatType::Reference(Box::new(self.type_to_fat_type(ty)?)),
            MutableReference(ty) => FatType::MutableReference(Box::new(self.type_to_fat_type(ty)?)),
            TyParam(idx) => FatType::TyParam(*idx),
        })
    }

    fn struct_gidx_to_type_tag(&self, gidx: usize, ty_args: &[Type]) -> PartialVMResult<StructTag> {
        if let Some(struct_map) = self.type_cache.lock().unwrap().structs.get(&gidx) {
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
        let struct_type = self.module_cache.lock().unwrap().struct_at(gidx);
        let struct_tag = StructTag {
            address: *struct_type.module.address(),
            module: struct_type.module.name().to_owned(),
            name: struct_type.name.clone(),
            type_params: ty_arg_tags,
        };

        self.type_cache
            .lock()
            .unwrap()
            .structs
            .entry(gidx)
            .or_insert_with(HashMap::new)
            .entry(ty_args.to_vec())
            .or_insert_with(StructInfo::new)
            .struct_tag = Some(struct_tag.clone());

        Ok(struct_tag)
    }

    pub(crate) fn type_to_type_tag(&self, ty: &Type) -> PartialVMResult<TypeTag> {
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
    ) -> PartialVMResult<MoveStructLayout> {
        if let Some(struct_map) = self.type_cache.lock().unwrap().structs.get(&gidx) {
            if let Some(struct_info) = struct_map.get(ty_args) {
                if let Some(layout) = &struct_info.struct_layout {
                    return Ok(layout.clone());
                }
            }
        }

        let struct_type = self.module_cache.lock().unwrap().struct_at(gidx);
        let field_tys = struct_type
            .fields
            .iter()
            .map(|ty| ty.subst(ty_args))
            .collect::<PartialVMResult<Vec<_>>>()?;
        let field_layouts = field_tys
            .iter()
            .map(|ty| self.type_to_type_layout(ty))
            .collect::<PartialVMResult<Vec<_>>>()?;
        let struct_layout = MoveStructLayout::new(field_layouts);

        self.type_cache
            .lock()
            .unwrap()
            .structs
            .entry(gidx)
            .or_insert_with(HashMap::new)
            .entry(ty_args.to_vec())
            .or_insert_with(StructInfo::new)
            .struct_layout = Some(struct_layout.clone());

        Ok(struct_layout)
    }

    pub(crate) fn type_to_type_layout(&self, ty: &Type) -> PartialVMResult<MoveTypeLayout> {
        Ok(match ty {
            Type::Bool => MoveTypeLayout::Bool,
            Type::U8 => MoveTypeLayout::U8,
            Type::U64 => MoveTypeLayout::U64,
            Type::U128 => MoveTypeLayout::U128,
            Type::Address => MoveTypeLayout::Address,
            Type::Signer => MoveTypeLayout::Signer,
            Type::Vector(ty) => MoveTypeLayout::Vector(Box::new(self.type_to_type_layout(ty)?)),
            Type::Struct(gidx) => {
                MoveTypeLayout::Struct(self.struct_gidx_to_type_layout(*gidx, &[])?)
            }
            Type::StructInstantiation(gidx, ty_args) => {
                MoveTypeLayout::Struct(self.struct_gidx_to_type_layout(*gidx, ty_args)?)
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
    ) -> PartialVMResult<(MoveKind, Vec<MoveKindInfo>)> {
        if let Some(struct_map) = self.type_cache.lock().unwrap().structs.get(&gidx) {
            if let Some(struct_info) = struct_map.get(ty_args) {
                if let Some(kind_info) = &struct_info.kind_info {
                    return Ok(kind_info.clone());
                }
            }
        }

        let struct_type = self.module_cache.lock().unwrap().struct_at(gidx);

        let mut is_resource = struct_type.is_resource;
        if !is_resource {
            for ty in ty_args {
                if self.is_resource(ty)? {
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
            .map(|ty| self.type_to_kind_info(ty))
            .collect::<PartialVMResult<Vec<_>>>()?;
        let kind_info = (MoveKind::from_bool(is_resource), field_kind_info);

        self.type_cache
            .lock()
            .unwrap()
            .structs
            .entry(gidx)
            .or_insert_with(HashMap::new)
            .entry(ty_args.to_vec())
            .or_insert_with(StructInfo::new)
            .kind_info = Some(kind_info.clone());

        Ok(kind_info)
    }

    pub(crate) fn type_to_kind_info(&self, ty: &Type) -> PartialVMResult<MoveKindInfo> {
        Ok(match ty {
            Type::Bool | Type::U8 | Type::U64 | Type::U128 | Type::Address => {
                MoveKindInfo::Base(MoveKind::Copyable)
            }
            Type::Signer => MoveKindInfo::Base(MoveKind::Resource),
            Type::Vector(ty) => {
                let kind_info = self.type_to_kind_info(ty)?;
                MoveKindInfo::Vector(kind_info.kind(), Box::new(kind_info))
            }
            Type::Struct(gidx) => {
                let (is_resource, field_kind_info) = self.struct_gidx_to_kind_info(*gidx, &[])?;
                MoveKindInfo::Struct(is_resource, field_kind_info)
            }
            Type::StructInstantiation(gidx, ty_args) => {
                let (is_resource, field_kind_info) =
                    self.struct_gidx_to_kind_info(*gidx, ty_args)?;
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

    fn struct_to_fat_struct(
        &self,
        idx: usize,
        ty_args: Vec<FatType>,
    ) -> PartialVMResult<FatStructType> {
        let struct_type = self.module_cache.lock().unwrap().struct_at(idx);
        let address = *struct_type.module.address();
        let module = struct_type.module.name().to_owned();
        let name = struct_type.name.clone();
        let is_resource = struct_type.is_resource;
        let mut fields = vec![];
        for field_type in &struct_type.fields {
            fields.push(self.type_to_fat_type(field_type)?);
        }
        let mut layout = vec![];
        for field in &fields {
            layout.push(field.subst(&ty_args)?);
        }
        Ok(FatStructType {
            address,
            module,
            name,
            is_resource,
            ty_args,
            layout,
        })
    }
}
