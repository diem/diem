// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::native_functions::NativeFunction;
use bytecode_verifier::{
    verifier::{verify_dependencies, verify_script_dependency_map},
    VerifiedModule, VerifiedScript,
};
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::{
    access_path::AccessPath,
    language_storage::{ModuleId, StructTag, TypeTag},
    vm_error::{StatusCode, VMStatus},
};
use move_core_types::identifier::{IdentStr, Identifier};
use move_vm_types::{
    interpreter_context::InterpreterContext,
    loaded_data::{
        runtime_types::{StructType, Type, TypeConverter},
        types::{FatStructType, FatType},
    },
};
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Mutex},
};
use vm::{
    access::{ModuleAccess, ScriptAccess},
    errors::{verification_error, vm_error, Location, VMResult},
    file_format::{
        Bytecode, CompiledScript, Constant, ConstantPoolIndex, FieldHandleIndex,
        FieldInstantiationIndex, FunctionDefinition, FunctionHandleIndex,
        FunctionInstantiationIndex, Kind, Signature, SignatureToken, StructDefInstantiationIndex,
        StructDefinition, StructDefinitionIndex, StructFieldInformation,
    },
    CompiledModule, IndexKind,
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

    fn insert(&mut self, hash: HashValue, script: Script) -> VMResult<Arc<Function>> {
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
struct ModuleCache {
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

    fn insert(&mut self, id: ModuleId, ver_module: VerifiedModule) -> VMResult<Arc<Module>> {
        self.add_module(&ver_module)?;
        let module = Module::new(ver_module, self)?;
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

    fn add_module(&mut self, module: &VerifiedModule) -> VMResult<()> {
        let starting_idx = self.structs.len();
        for (idx, struct_def) in module.struct_defs().iter().enumerate() {
            let st = self.load_type(module, struct_def, StructDefinitionIndex(idx as u16))?;
            self.structs.push(Arc::new(st));
        }
        self.load_fields(module, starting_idx)?;

        for func in module.function_defs() {
            let function = self.load_function(module, func)?;
            self.functions.push(Arc::new(function));
        }
        Ok(())
    }

    fn load_type(
        &self,
        module: &VerifiedModule,
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

    fn load_fields(&mut self, module: &VerifiedModule, starting_idx: usize) -> VMResult<()> {
        let mut field_types = vec![];
        for struct_def in module.struct_defs() {
            let fields = match &struct_def.field_information {
                StructFieldInformation::Native => unreachable!("native structs have been removed"),
                StructFieldInformation::Declared(fields) => fields,
            };

            let mut field_tys = vec![];
            for field in fields {
                let ty = self.make_type(module.as_inner(), &field.signature.0)?;
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
                    return Err(VMStatus::new(StatusCode::INVALID_CODE_CACHE)
                        .with_message("Arc Type should not have any reference".to_string()))
                }
                Some(struct_type) => struct_type.fields = fields,
            }
        }
        Ok(())
    }

    fn make_type(&self, binary: &dyn ModuleAccess, tok: &SignatureToken) -> VMResult<Type> {
        let res = match tok {
            SignatureToken::Bool => Type::Bool,
            SignatureToken::U8 => Type::U8,
            SignatureToken::U64 => Type::U64,
            SignatureToken::U128 => Type::U128,
            SignatureToken::Address => Type::Address,
            SignatureToken::TypeParameter(idx) => Type::TyParam(*idx as usize),
            SignatureToken::Vector(inner_tok) => {
                let inner_type = self.make_type(binary, inner_tok)?;
                Type::Vector(Box::new(inner_type))
            }
            SignatureToken::Reference(inner_tok) => {
                let inner_type = self.make_type(binary, inner_tok)?;
                Type::Reference(Box::new(inner_type))
            }
            SignatureToken::MutableReference(inner_tok) => {
                let inner_type = self.make_type(binary, inner_tok)?;
                Type::MutableReference(Box::new(inner_type))
            }
            SignatureToken::Struct(sh_idx) => {
                let struct_handle = binary.struct_handle_at(*sh_idx);
                let struct_name = binary.identifier_at(struct_handle.name);
                let module_handle = binary.module_handle_at(struct_handle.module);
                let module_id = ModuleId::new(
                    *binary.address_identifier_at(module_handle.address),
                    binary.identifier_at(module_handle.name).to_owned(),
                );
                let def_idx = self.find_struct_by_name(struct_name, &module_id)?.0;
                Type::Struct(def_idx)
            }
            SignatureToken::StructInstantiation(sh_idx, tys) => {
                let type_parameters: Vec<_> = tys
                    .iter()
                    .map(|tok| self.make_type(binary, tok))
                    .collect::<VMResult<_>>()?;
                let struct_handle = binary.struct_handle_at(*sh_idx);
                let struct_name = binary.identifier_at(struct_handle.name);
                let module_handle = binary.module_handle_at(struct_handle.module);
                let module_id = ModuleId::new(
                    *binary.address_identifier_at(module_handle.address),
                    binary.identifier_at(module_handle.name).to_owned(),
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
    ) -> VMResult<(usize, Arc<StructType>)> {
        for (idx, ty) in self.structs.iter().enumerate() {
            if struct_match(ty, &module_id, struct_name) {
                return Ok((idx, Arc::clone(ty)));
            }
        }
        Err(VMStatus::new(StatusCode::UNREACHABLE).with_message(format!(
            "Cannot find {:?}::{:?} in cache",
            module_id, struct_name
        )))
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
        Err(VMStatus::new(StatusCode::UNREACHABLE)
            .with_message("Missing function in cache".to_string()))
    }

    fn load_function(
        &self,
        module: &VerifiedModule,
        func_def: &FunctionDefinition,
    ) -> VMResult<Function> {
        Ok(Function::new(func_def, module))
    }
}

// A Loader is responsible to load scripts and modules and holds the cache of all loaded
// entities. Each cache is protected by a `Mutex`. Operation in the Loader must be thread safe
// (operating on values on the stack) and when cache needs updating the mutex must be taken.
// The `pub(crate)` API is what a Loader offers to the runtime.
pub struct Loader {
    scripts: Mutex<ScriptCache>,
    module_cache: Mutex<ModuleCache>,
    libra_cache: Mutex<HashMap<ModuleId, LibraCache>>,
}

impl Loader {
    pub(crate) fn new() -> Self {
        Self {
            scripts: Mutex::new(ScriptCache::new()),
            module_cache: Mutex::new(ModuleCache::new()),
            libra_cache: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn function_at(&self, idx: usize) -> Arc<Function> {
        self.module_cache.lock().unwrap().function_at(idx)
    }

    pub(crate) fn load_function(
        &self,
        function_name: &IdentStr,
        module_id: &ModuleId,
        context: &mut dyn InterpreterContext,
    ) -> VMResult<Arc<Function>> {
        self.load_module(module_id, context)?;
        let idx = self
            .module_cache
            .lock()
            .unwrap()
            .resolve_function_handle(function_name, module_id)?;
        Ok(self.module_cache.lock().unwrap().function_at(idx))
    }

    pub(crate) fn load_script(
        &self,
        script_blob: &[u8],
        context: &mut dyn InterpreterContext,
    ) -> VMResult<Arc<Function>> {
        let hash_value = HashValue::from_sha3_256(script_blob);
        if let Some(main) = self.scripts.lock().unwrap().get(&hash_value) {
            return Ok(main);
        }

        let ver_script = self.deserialize_and_verify_script(script_blob, context)?;
        let script = Script::new(ver_script, &hash_value, &self.module_cache.lock().unwrap())?;
        self.scripts.lock().unwrap().insert(hash_value, script)
    }

    pub(crate) fn load_type(
        &self,
        type_tag: &TypeTag,
        context: &dyn InterpreterContext,
    ) -> VMResult<Type> {
        Ok(match type_tag {
            TypeTag::Bool => Type::Bool,
            TypeTag::U8 => Type::U8,
            TypeTag::U64 => Type::U64,
            TypeTag::U128 => Type::U128,
            TypeTag::Address => Type::Address,
            TypeTag::Vector(tt) => Type::Vector(Box::new(self.load_type(tt, context)?)),
            TypeTag::Struct(struct_tag) => {
                let module_id = ModuleId::new(struct_tag.address, struct_tag.module.clone());
                self.load_module(&module_id, context)?;
                let (idx, struct_type) = self
                    .module_cache
                    .lock()
                    .unwrap()
                    .find_struct_by_name(&struct_tag.name, &module_id)?;
                if struct_type.type_parameters.is_empty() && struct_tag.type_params.is_empty() {
                    Type::Struct(idx)
                } else {
                    let mut type_params = vec![];
                    for ty_param in &struct_tag.type_params {
                        type_params.push(self.load_type(ty_param, context)?);
                    }
                    self.verify_ty_args(&struct_type.type_parameters, &type_params)?;
                    Type::StructInstantiation(idx, type_params)
                }
            }
        })
    }

    pub(crate) fn cache_module(
        &self,
        module: VerifiedModule,
        context: &mut dyn InterpreterContext,
    ) -> VMResult<()> {
        self.check_dependencies(&module, context)?;
        Self::check_natives(&module)?;
        let module_id = module.self_id();
        self.module_cache
            .lock()
            .unwrap()
            .insert(module_id, module)
            .and_then(|_| Ok(()))
    }

    fn load_module(
        &self,
        id: &ModuleId,
        context: &dyn InterpreterContext,
    ) -> VMResult<Arc<Module>> {
        if let Some(module) = self.module_cache.lock().unwrap().get(id) {
            return Ok(module);
        }
        let module = self.deserialize_and_verify_module(id, context)?;
        Self::check_natives(&module)?;
        self.module_cache.lock().unwrap().insert(id.clone(), module)
    }

    pub(crate) fn verify_ty_args(&self, constraints: &[Kind], ty_args: &[Type]) -> VMResult<()> {
        if constraints.len() != ty_args.len() {
            return Err(VMStatus::new(StatusCode::NUMBER_OF_TYPE_ARGUMENTS_MISMATCH));
        }
        for (ty, expected_k) in ty_args.iter().zip(constraints) {
            let k = if self.is_resource(ty)? {
                Kind::Resource
            } else {
                Kind::Copyable
            };
            if !k.is_sub_kind_of(*expected_k) {
                return Err(VMStatus::new(StatusCode::CONTRAINT_KIND_MISMATCH));
            }
        }
        Ok(())
    }

    pub(crate) fn check_natives(module: &VerifiedModule) -> VMResult<()> {
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
                    IndexKind::FunctionHandle,
                    idx,
                    StatusCode::MISSING_DEPENDENCY,
                )
            })?;
        }
        // TODO: fix check and error code if we leave something around for native structs.
        // For now this generates the only error test cases care about...
        for (idx, struct_def) in module.struct_defs().iter().enumerate() {
            if struct_def.field_information == StructFieldInformation::Native {
                return Err(verification_error(
                    IndexKind::FunctionHandle,
                    idx,
                    StatusCode::MISSING_DEPENDENCY,
                ));
            }
        }
        Ok(())
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

    fn is_resource(&self, type_: &Type) -> VMResult<bool> {
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

    fn deserialize_and_verify_script(
        &self,
        script: &[u8],
        context: &mut dyn InterpreterContext,
    ) -> VMResult<VerifiedScript> {
        let script = match CompiledScript::deserialize(script) {
            Ok(script) => script,
            Err(err) => {
                error!("[VM] deserializer for script returned error: {:?}", err);
                let error = vm_error(Location::default(), StatusCode::CODE_DESERIALIZATION_ERROR)
                    .append(err);
                return Err(error);
            }
        };

        let err = match VerifiedScript::new(script) {
            Ok(script) => {
                // verify dependencies
                let deps = load_script_dependencies(&script);
                let mut dependencies = vec![];
                for dep in &deps {
                    dependencies.push(self.load_module(dep, context)?);
                }
                let mut dependency_map = BTreeMap::new();
                for dependency in &dependencies {
                    dependency_map.insert(dependency.module_id().clone(), dependency.module());
                }
                match verify_script_dependency_map(&script, &dependency_map) {
                    Ok(()) => return Ok(script),
                    Err(e) => e,
                }
            }
            Err((_, errs)) => errs,
        };
        error!(
            "[VM] bytecode verifier returned errors for script: {:?}",
            err
        );
        // If there are errors there should be at least one otherwise there's an internal
        // error in the verifier. We only give back the first error. If the user wants to
        // debug things, they can do that offline.
        Err(err)
    }

    fn deserialize_and_verify_module(
        &self,
        id: &ModuleId,
        context: &dyn InterpreterContext,
    ) -> VMResult<VerifiedModule> {
        let comp_module = match context.load_module(id) {
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
            Ok(module) => {
                self.check_dependencies(&module, context)?;
                Ok(module)
            }
            Err((_, err)) => Err(err),
        }
    }

    fn check_dependencies(
        &self,
        module: &VerifiedModule,
        context: &dyn InterpreterContext,
    ) -> VMResult<()> {
        let deps = load_module_dependencies(module);
        let mut dependencies = vec![];
        for dep in &deps {
            dependencies.push(self.load_module(dep, context)?);
        }
        let mut dependency_map = BTreeMap::new();
        for dependency in &dependencies {
            dependency_map.insert(dependency.module_id().clone(), dependency.module());
        }
        verify_dependencies(module, &dependency_map)
    }
}

// A simple wrapper for a `Module` or a `Script` in the `Resolver`
enum BinaryType {
    Module(Arc<Module>),
    Script(Arc<Script>),
}

// A Resolver is a simple and small structure allocated on the stack and used by the
// interpreter. It's the only API known to the interpreter and it's tailored to the interpreter
// needs.
pub struct Resolver<'a> {
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

    pub(crate) fn constant_at(&self, idx: ConstantPoolIndex) -> &Constant {
        match &self.binary {
            BinaryType::Module(module) => module.module.constant_at(idx),
            BinaryType::Script(script) => script.script.constant_at(idx),
        }
    }

    pub(crate) fn function_at(&self, idx: FunctionHandleIndex) -> Arc<Function> {
        let idx = match &self.binary {
            BinaryType::Module(module) => module.function_at(idx.0),
            BinaryType::Script(script) => script.function_at(idx.0),
        };
        self.loader.function_at(idx)
    }

    pub(crate) fn function_instantiation_at(
        &self,
        idx: FunctionInstantiationIndex,
    ) -> &FunctionInstantiation {
        match &self.binary {
            BinaryType::Module(module) => module.function_instantiation_at(idx.0),
            BinaryType::Script(script) => script.function_instantiation_at(idx.0),
        }
    }

    pub(crate) fn struct_at(&self, idx: StructDefinitionIndex) -> Arc<StructType> {
        match &self.binary {
            BinaryType::Module(module) => {
                let gidx = module.struct_at(idx);
                self.loader.struct_at(gidx)
            }
            BinaryType::Script(_) => unreachable!("Scripts cannot have type instructions"),
        }
    }

    pub(crate) fn struct_type_at(&self, idx: usize) -> Arc<StructType> {
        self.loader.struct_at(idx)
    }

    pub(crate) fn struct_instantiation_at(
        &self,
        idx: StructDefInstantiationIndex,
    ) -> &StructInstantiation {
        match &self.binary {
            BinaryType::Module(module) => module.struct_instantiation_at(idx.0),
            BinaryType::Script(_) => unreachable!("Scripts cannot have type instructions"),
        }
    }

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

    pub(crate) fn get_libra_type_info(
        &self,
        module_id: &ModuleId,
        name: &IdentStr,
        ty_args: &[Type],
        context: &mut dyn InterpreterContext,
    ) -> VMResult<Arc<LibraType>> {
        self.loader.load_module(module_id, context)?;
        self.loader.get_libra_type_info(module_id, name, ty_args)
    }

    pub fn type_to_fat_type(&self, ty: &Type) -> VMResult<FatType> {
        self.loader.type_to_fat_type(ty)
    }

    pub(crate) fn make_fat_type(
        &self,
        token: &SignatureToken,
        type_context: &[Type],
    ) -> VMResult<FatType> {
        match &self.binary {
            BinaryType::Module(module) => {
                let binary = &module.module;
                let ty = self
                    .loader
                    .module_cache
                    .lock()
                    .unwrap()
                    .make_type(binary.as_inner(), token)?
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
    fn type_to_fat_type(&self, ty: &Type) -> VMResult<FatType> {
        Resolver::type_to_fat_type(self, ty)
    }
}

// A Module is very similar to a binary Module but data is "transformed" to a representation
// more appropriate to execution.
// When code executes indexes in instructions are resolved against those runtime structure
// so that any data needed for execution is immediately available
#[derive(Debug)]
pub struct Module {
    id: ModuleId,
    // primitive pools
    module: VerifiedModule,

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
    fn new(module: VerifiedModule, cache: &ModuleCache) -> VMResult<Self> {
        let id = module.self_id();

        let mut struct_refs = vec![];
        for struct_handle in module.struct_handles() {
            let struct_name = module.identifier_at(struct_handle.name);
            let module_handle = module.module_handle_at(struct_handle.module);
            let module_id = module.as_inner().module_id_for_handle(module_handle);
            struct_refs.push(cache.find_struct_by_name(struct_name, &module_id)?.0);
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
                instantiation.push(cache.make_type(module.as_inner(), ty)?);
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
            let module_id = module.as_inner().module_id_for_handle(module_handle);
            let ref_idx = cache.resolve_function_handle(func_name, &module_id)?;
            function_refs.push(ref_idx);
        }

        let mut function_instantiations = vec![];
        for func_inst in module.function_instantiations() {
            let handle = function_refs[func_inst.handle.0 as usize];
            let mut instantiation = vec![];
            for ty in &module.signature_at(func_inst.type_parameters).0 {
                instantiation.push(cache.make_type(module.as_inner(), ty)?);
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

    fn module_id(&self) -> &ModuleId {
        &self.id
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

    fn module(&self) -> &VerifiedModule {
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
    script: VerifiedScript,

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
    fn new(script: VerifiedScript, script_hash: &HashValue, cache: &ModuleCache) -> VMResult<Self> {
        let mut struct_refs = vec![];
        for struct_handle in script.struct_handles() {
            let struct_name = script.identifier_at(struct_handle.name);
            let module_handle = script.module_handle_at(struct_handle.module);
            let module_id = ModuleId::new(
                *script.address_identifier_at(module_handle.address),
                script.identifier_at(module_handle.name).to_owned(),
            );
            struct_refs.push(cache.find_struct_by_name(struct_name, &module_id)?.0);
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
                instantiation.push(cache.make_type(&module, ty)?);
            }
            function_instantiations.push(FunctionInstantiation {
                handle,
                instantiation,
            });
        }

        let scope = Scope::Script(*script_hash);

        let compiled_script = script.as_inner().as_inner();
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
pub struct Function {
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
    fn new(def: &FunctionDefinition, module: &VerifiedModule) -> Self {
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

    pub(crate) fn get_native(&self) -> VMResult<NativeFunction> {
        self.native.ok_or_else(|| {
            VMStatus::new(StatusCode::UNREACHABLE)
                .with_message("Missing Native Function".to_string())
        })
    }
}

// A function instantiation.
#[derive(Debug)]
pub struct FunctionInstantiation {
    handle: usize,
    instantiation: Vec<Type>,
}

impl FunctionInstantiation {
    pub(crate) fn materialize(&self, type_params: &[Type]) -> VMResult<Vec<Type>> {
        let mut instantiation = vec![];
        for ty in &self.instantiation {
            instantiation.push(ty.subst(type_params)?);
        }
        Ok(instantiation)
    }

    pub(crate) fn handle(&self) -> usize {
        self.handle
    }

    pub(crate) fn instantiation_size(&self) -> usize {
        self.instantiation.len()
    }
}

// A struct definition carries an index to the type in the ModuleCache and the field count
// which is the most common information used at runtime
#[derive(Debug)]
struct StructDef {
    field_count: u16,
    idx: usize,
}

// A struct insantiation.
#[derive(Debug)]
pub(crate) struct StructInstantiation {
    field_count: u16,
    def: usize,
    instantiation: Vec<Type>,
}

impl StructInstantiation {
    pub(crate) fn get_def_idx(&self) -> usize {
        self.def
    }

    pub(crate) fn get_instantiation(&self) -> &[Type] {
        &self.instantiation
    }
}

// A field handle. The offset is the only used information when operating on a field
#[derive(Debug)]
struct FieldHandle {
    offset: usize,
    owner: usize,
}

// A field instantiation. The offset is the only used information when operating on a field
#[derive(Debug)]
pub struct FieldInstantiation {
    offset: usize,
    owner: usize,
}

//
// Utility functions
//

fn load_script_dependencies(binary: &dyn ScriptAccess) -> Vec<ModuleId> {
    let mut deps = vec![];
    for module in binary.module_handles() {
        deps.push(ModuleId::new(
            *binary.address_identifier_at(module.address),
            binary.identifier_at(module.name).to_owned(),
        ));
    }
    deps
}

fn load_module_dependencies(binary: &dyn ModuleAccess) -> Vec<ModuleId> {
    let self_module = binary.self_handle();
    let mut deps = vec![];
    for module in binary.module_handles() {
        if module == self_module {
            continue;
        }
        deps.push(ModuleId::new(
            *binary.address_identifier_at(module.address),
            binary.identifier_at(module.name).to_owned(),
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

//
// What follows is the API to get serde info for Move values (FatStructType) and
// resource keys for AccessPath.
// All of that should go into the data cache once the data cache is refactored.
// There is no reason to put this knowledge into the Move VM and it would be better
// handled by the data cache and the Move <-> Libra layer.
//

#[derive(Debug)]
pub(crate) struct LibraType {
    fat_type: FatStructType,
    resource_key: Vec<u8>,
}

impl LibraType {
    fn make(fat_type: FatStructType) -> VMResult<Self> {
        let mut type_params = vec![];
        for ty in &fat_type.ty_args {
            type_params.push(ty.type_tag()?);
        }
        let tag = StructTag {
            address: fat_type.address,
            module: fat_type.module.clone(),
            name: fat_type.name.clone(),
            type_params,
        };
        let resource_key = AccessPath::resource_access_vec(&tag);
        Ok(Self {
            fat_type,
            resource_key,
        })
    }

    pub(crate) fn fat_type(&self) -> &FatStructType {
        &self.fat_type
    }

    pub(crate) fn resource_key(&self) -> &[u8] {
        &self.resource_key
    }
}

#[derive(Debug)]
struct LibraTypeInfo {
    instantiations: HashMap<Vec<Type>, Arc<LibraType>>,
}

impl LibraTypeInfo {
    fn new() -> Self {
        Self {
            instantiations: HashMap::new(),
        }
    }

    fn get(&self, instantiation: &[Type]) -> Option<&Arc<LibraType>> {
        self.instantiations.get(instantiation)
    }

    fn add(&mut self, instantiation: Vec<Type>, libra_type: LibraType) -> Arc<LibraType> {
        Arc::clone(
            self.instantiations
                .entry(instantiation)
                .or_insert_with(|| Arc::new(libra_type)),
        )
    }
}

#[derive(Debug)]
struct LibraCache {
    cache: HashMap<Identifier, LibraTypeInfo>,
}

impl LibraCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn get_libra_type_info(
        &self,
        name: &IdentStr,
        instantiation: &[Type],
    ) -> Option<&Arc<LibraType>> {
        self.cache
            .get(name)
            .and_then(|type_info| type_info.get(instantiation))
    }

    fn set_libra_type_info(
        &mut self,
        name: Identifier,
        instantiation: Vec<Type>,
        libra_type: LibraType,
    ) -> Arc<LibraType> {
        let type_info = self.cache.entry(name).or_insert_with(LibraTypeInfo::new);
        type_info.add(instantiation, libra_type)
    }
}

impl Loader {
    fn type_to_fat_type(&self, ty: &Type) -> VMResult<FatType> {
        use Type::*;

        Ok(match ty {
            Bool => FatType::Bool,
            U8 => FatType::U8,
            U64 => FatType::U64,
            U128 => FatType::U128,
            Address => FatType::Address,
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

    fn struct_to_fat_struct(&self, idx: usize, ty_args: Vec<FatType>) -> VMResult<FatStructType> {
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

    fn get_libra_type_info(
        &self,
        module_id: &ModuleId,
        name: &IdentStr,
        type_params: &[Type],
    ) -> VMResult<Arc<LibraType>> {
        if let Some(libra_cache) = self.libra_cache.lock().unwrap().get(module_id) {
            if let Some(libra_type) = libra_cache.get_libra_type_info(name, type_params) {
                return Ok(Arc::clone(libra_type));
            }
        }

        let (idx, _) = self
            .module_cache
            .lock()
            .unwrap()
            .find_struct_by_name(name, module_id)?;
        let mut ty_args = vec![];
        for inst in type_params {
            ty_args.push(self.type_to_fat_type(inst)?);
        }
        let fat_struct = self.struct_to_fat_struct(idx, ty_args)?;
        let libra_type = LibraType::make(fat_struct)?;
        let mut libra_module_cache = self.libra_cache.lock().unwrap();
        let libra_cache = libra_module_cache
            .entry(module_id.clone())
            .or_insert_with(LibraCache::new);
        Ok(libra_cache.set_libra_type_info(name.to_owned(), type_params.to_vec(), libra_type))
    }
}
