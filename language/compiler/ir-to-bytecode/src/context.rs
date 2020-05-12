// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use bytecode_source_map::source_map::SourceMap;
use libra_types::account_address::AccountAddress;
use move_core_types::identifier::{IdentStr, Identifier};
use move_ir_types::{
    ast::{
        BlockLabel, Field_, FunctionName, ModuleName, QualifiedModuleIdent, QualifiedStructIdent,
        StructName,
    },
    location::*,
};
use std::{clone::Clone, collections::HashMap, hash::Hash};
use vm::{
    access::ModuleAccess,
    file_format::{
        AddressIdentifierIndex, CodeOffset, Constant, ConstantPoolIndex, FieldHandle,
        FieldHandleIndex, FieldInstantiation, FieldInstantiationIndex, FunctionDefinitionIndex,
        FunctionHandle, FunctionHandleIndex, FunctionInstantiation, FunctionInstantiationIndex,
        FunctionSignature, IdentifierIndex, Kind, ModuleHandle, ModuleHandleIndex, Signature,
        SignatureIndex, SignatureToken, StructDefInstantiation, StructDefInstantiationIndex,
        StructDefinitionIndex, StructHandle, StructHandleIndex, TableIndex,
    },
};

macro_rules! get_or_add_item_macro {
    ($m:ident, $k_get:expr, $k_insert:expr) => {{
        let k_key = $k_get;
        Ok(if $m.contains_key(k_key) {
            *$m.get(k_key).unwrap()
        } else {
            let len = $m.len();
            if len >= TABLE_MAX_SIZE {
                bail!("Max table size reached!")
            }
            let index = len as TableIndex;
            $m.insert($k_insert, index);
            index
        })
    }};
}

pub const TABLE_MAX_SIZE: usize = u16::max_value() as usize;
fn get_or_add_item_ref<K: Clone + Eq + Hash>(
    m: &mut HashMap<K, TableIndex>,
    k: &K,
) -> Result<TableIndex> {
    get_or_add_item_macro!(m, k, k.clone())
}

fn get_or_add_item<K: Eq + Hash>(m: &mut HashMap<K, TableIndex>, k: K) -> Result<TableIndex> {
    get_or_add_item_macro!(m, &k, k)
}

pub fn ident_str(s: &str) -> Result<&IdentStr> {
    IdentStr::new(s)
}

struct CompiledDependency<'a> {
    structs: HashMap<(&'a IdentStr, &'a IdentStr), TableIndex>,
    functions: HashMap<&'a IdentStr, TableIndex>,

    module_pool: &'a [ModuleHandle],
    struct_pool: &'a [StructHandle],
    function_pool: &'a [FunctionHandle],
    identifiers: &'a [Identifier],
    address_identifiers: &'a [AccountAddress],
    signature_pool: &'a [Signature],
}

impl<'a> CompiledDependency<'a> {
    fn new<T: 'a + ModuleAccess>(dep: &'a T) -> Result<Self> {
        let mut structs = HashMap::new();
        let mut functions = HashMap::new();

        for shandle in dep.struct_handles() {
            let mhandle = dep.module_handle_at(shandle.module);
            let mname = dep.identifier_at(mhandle.name);
            let sname = dep.identifier_at(shandle.name);
            // get_or_add_item gets the proper struct handle index, as `dep.struct_handles()` is
            // properly ordered
            get_or_add_item(&mut structs, (mname, sname))?;
        }

        // keep only functions defined in the current module
        // with module handle 0
        let defined_function_handles = dep
            .function_handles()
            .iter()
            .enumerate()
            .filter(|(_idx, fhandle)| fhandle.module.0 == 0);
        for (idx, fhandle) in defined_function_handles {
            let fname = dep.identifier_at(fhandle.name);
            functions.insert(fname, idx as u16);
        }

        Ok(Self {
            structs,
            functions,
            module_pool: dep.module_handles(),
            struct_pool: dep.struct_handles(),
            function_pool: dep.function_handles(),
            identifiers: dep.identifiers(),
            address_identifiers: dep.address_identifiers(),
            signature_pool: dep.signatures(),
        })
    }

    fn source_struct_info(
        &self,
        idx: StructHandleIndex,
    ) -> Option<(QualifiedModuleIdent, StructName)> {
        let handle = self.struct_pool.get(idx.0 as usize)?;
        let module_handle = self.module_pool.get(handle.module.0 as usize)?;
        let address = *self
            .address_identifiers
            .get(module_handle.address.0 as usize)?;
        let module = ModuleName::new(
            self.identifiers
                .get(module_handle.name.0 as usize)?
                .to_string(),
        );
        assert!(module.as_inner() != ModuleName::self_name());
        let ident = QualifiedModuleIdent {
            address,
            name: module,
        };
        let name = StructName::new(self.identifiers.get(handle.name.0 as usize)?.to_string());
        Some((ident, name))
    }

    fn struct_handle(&self, name: &QualifiedStructIdent) -> Option<&'a StructHandle> {
        self.structs
            .get(&(
                ident_str(name.module.as_inner()).ok()?,
                ident_str(name.name.as_inner()).ok()?,
            ))
            .and_then(|idx| self.struct_pool.get(*idx as usize))
    }

    fn function_signature(&self, name: &FunctionName) -> Option<FunctionSignature> {
        self.functions
            .get(ident_str(name.as_inner()).ok()?)
            .and_then(|idx| {
                let fh = self.function_pool.get(*idx as usize)?;
                Some(FunctionSignature {
                    parameters: self.signature_pool[fh.parameters.0 as usize].0.clone(),
                    return_: self.signature_pool[fh.return_.0 as usize].0.clone(),
                    type_parameters: fh.type_parameters.clone(),
                })
            })
    }
}

/// Represents all of the pools to be used in the file format, both by CompiledModule
/// and CompiledScript.
pub struct MaterializedPools {
    /// Module handle pool
    pub module_handles: Vec<ModuleHandle>,
    /// Struct handle pool
    pub struct_handles: Vec<StructHandle>,
    /// Function handle pool
    pub function_handles: Vec<FunctionHandle>,
    /// Field handle pool
    pub field_handles: Vec<FieldHandle>,
    /// Struct instantiation pool
    pub struct_def_instantiations: Vec<StructDefInstantiation>,
    /// Function instantiation pool
    pub function_instantiations: Vec<FunctionInstantiation>,
    /// Field instantiation pool
    pub field_instantiations: Vec<FieldInstantiation>,
    /// Locals signatures pool
    pub signatures: Vec<Signature>,
    /// Identifier pool
    pub identifiers: Vec<Identifier>,
    /// Address identifier pool
    pub address_identifiers: Vec<AccountAddress>,
    /// Constnat pool
    pub constant_pool: Vec<Constant>,
}

/// Compilation context for a single compilation unit (module or script).
/// Contains all of the pools as they are built up.
/// Specific definitions to CompiledModule or CompiledScript are not stored.
/// However, some fields, like struct_defs and fields, are not used in CompiledScript.
pub struct Context<'a> {
    dependencies: HashMap<QualifiedModuleIdent, CompiledDependency<'a>>,

    // helpers
    aliases: HashMap<QualifiedModuleIdent, ModuleName>,
    modules: HashMap<ModuleName, (QualifiedModuleIdent, ModuleHandle)>,
    structs: HashMap<QualifiedStructIdent, StructHandle>,
    struct_defs: HashMap<StructName, TableIndex>,
    labels: HashMap<BlockLabel, u16>,

    // queryable pools
    // TODO: lookup for Fields is not that seemless after binary format changes
    // We need multiple lookups or a better representation for fields
    fields: HashMap<(StructHandleIndex, Field_), (StructDefinitionIndex, SignatureToken, usize)>,
    function_handles: HashMap<(ModuleName, FunctionName), (FunctionHandle, FunctionHandleIndex)>,
    function_signatures: HashMap<(ModuleName, FunctionName), FunctionSignature>,

    // Simple pools
    module_handles: HashMap<ModuleHandle, TableIndex>,
    struct_handles: HashMap<StructHandle, TableIndex>,
    signatures: HashMap<Signature, TableIndex>,
    identifiers: HashMap<Identifier, TableIndex>,
    address_identifiers: HashMap<AccountAddress, TableIndex>,
    constant_pool: HashMap<Constant, TableIndex>,
    field_handles: HashMap<FieldHandle, TableIndex>,
    struct_instantiations: HashMap<StructDefInstantiation, TableIndex>,
    function_instantiations: HashMap<FunctionInstantiation, TableIndex>,
    field_instantiations: HashMap<FieldInstantiation, TableIndex>,

    // The current function index that we are on
    current_function_index: FunctionDefinitionIndex,

    // Source location mapping for this module
    pub source_map: SourceMap<Loc>,
}

impl<'a> Context<'a> {
    /// Given the dependencies and the current module, creates an empty context.
    /// The current module is a dummy `Self` for CompiledScript.
    /// It initializes an "import" of `Self` as the alias for the current_module.
    pub fn new<T: 'a + ModuleAccess>(
        dependencies_iter: impl IntoIterator<Item = &'a T>,
        current_module_opt: Option<QualifiedModuleIdent>,
    ) -> Result<Self> {
        let dependencies = dependencies_iter
            .into_iter()
            .map(|dep| {
                let ident = QualifiedModuleIdent {
                    address: *dep.address(),
                    name: ModuleName::new(dep.name().to_string()),
                };
                Ok((ident, CompiledDependency::new(dep)?))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        let context = Self {
            dependencies,
            aliases: HashMap::new(),
            modules: HashMap::new(),
            structs: HashMap::new(),
            struct_defs: HashMap::new(),
            labels: HashMap::new(),
            fields: HashMap::new(),
            function_handles: HashMap::new(),
            function_signatures: HashMap::new(),
            module_handles: HashMap::new(),
            struct_handles: HashMap::new(),
            field_handles: HashMap::new(),
            struct_instantiations: HashMap::new(),
            function_instantiations: HashMap::new(),
            field_instantiations: HashMap::new(),
            signatures: HashMap::new(),
            identifiers: HashMap::new(),
            address_identifiers: HashMap::new(),
            constant_pool: HashMap::new(),
            current_function_index: FunctionDefinitionIndex::new(0),
            source_map: SourceMap::new(current_module_opt),
        };

        Ok(context)
    }

    fn materialize_pool<T: Clone>(
        size: usize,
        items: impl IntoIterator<Item = (T, TableIndex)>,
    ) -> Vec<T> {
        let mut options = vec![None; size];
        for (item, idx) in items {
            assert!(options[idx as usize].is_none());
            options[idx as usize] = Some(item);
        }
        options.into_iter().map(|opt| opt.unwrap()).collect()
    }

    fn materialize_map<T: Clone>(m: HashMap<T, TableIndex>) -> Vec<T> {
        Self::materialize_pool(m.len(), m.into_iter())
    }

    /// Finish compilation, and materialize the pools for file format.
    pub fn materialize_pools(self) -> (MaterializedPools, SourceMap<Loc>) {
        let num_functions = self.function_handles.len();
        assert!(num_functions == self.function_signatures.len());
        let function_handles = Self::materialize_pool(
            num_functions,
            self.function_handles
                .into_iter()
                .map(|(_, (t, idx))| (t, idx.0)),
        );
        let materialized_pools = MaterializedPools {
            function_handles,
            module_handles: Self::materialize_map(self.module_handles),
            struct_handles: Self::materialize_map(self.struct_handles),
            field_handles: Self::materialize_map(self.field_handles),
            signatures: Self::materialize_map(self.signatures),
            identifiers: Self::materialize_map(self.identifiers),
            address_identifiers: Self::materialize_map(self.address_identifiers),
            constant_pool: Self::materialize_map(self.constant_pool),
            function_instantiations: Self::materialize_map(self.function_instantiations),
            struct_def_instantiations: Self::materialize_map(self.struct_instantiations),
            field_instantiations: Self::materialize_map(self.field_instantiations),
        };
        (materialized_pools, self.source_map)
    }

    pub fn build_index_remapping(
        &mut self,
        label_to_index: HashMap<BlockLabel, u16>,
    ) -> HashMap<u16, u16> {
        let labels = std::mem::replace(&mut self.labels, HashMap::new());
        label_to_index
            .into_iter()
            .map(|(lbl, actual_idx)| (labels[&lbl], actual_idx))
            .collect()
    }

    //**********************************************************************************************
    // Pools
    //**********************************************************************************************

    /// Get the alias for the identifier, fails if it is not bound.
    fn module_alias(&self, ident: &QualifiedModuleIdent) -> Result<&ModuleName> {
        self.aliases
            .get(ident)
            .ok_or_else(|| format_err!("Missing import for module {}", ident))
    }

    /// Get the handle for the alias, fails if it is not bound.
    fn module_handle(&self, module_name: &ModuleName) -> Result<&ModuleHandle> {
        match self.modules.get(module_name) {
            None => bail!("Unbound module alias {}", module_name),
            Some((_, mh)) => Ok(mh),
        }
    }

    /// Get the identifier for the alias, fails if it is not bound.
    fn module_ident(&self, module_name: &ModuleName) -> Result<&QualifiedModuleIdent> {
        match self.modules.get(module_name) {
            None => bail!("Unbound module alias {}", module_name),
            Some((id, _)) => Ok(id),
        }
    }

    /// Get the module handle index for the alias, fails if it is not bound.
    fn module_handle_index(&self, module_name: &ModuleName) -> Result<ModuleHandleIndex> {
        Ok(ModuleHandleIndex(
            *self
                .module_handles
                .get(self.module_handle(module_name)?)
                .unwrap(),
        ))
    }

    /// Get the field handle index for the alias, adds it if missing.
    pub fn field_handle_index(
        &mut self,
        owner: StructDefinitionIndex,
        field: u16,
    ) -> Result<FieldHandleIndex> {
        let field_handle = FieldHandle { owner, field };
        Ok(FieldHandleIndex(get_or_add_item(
            &mut self.field_handles,
            field_handle,
        )?))
    }

    /// Get the struct instantiation index for the alias, adds it if missing.
    pub fn struct_instantiation_index(
        &mut self,
        def: StructDefinitionIndex,
        type_parameters: SignatureIndex,
    ) -> Result<StructDefInstantiationIndex> {
        let struct_inst = StructDefInstantiation {
            def,
            type_parameters,
        };
        Ok(StructDefInstantiationIndex(get_or_add_item(
            &mut self.struct_instantiations,
            struct_inst,
        )?))
    }

    /// Get the function instantiation index for the alias, adds it if missing.
    pub fn function_instantiation_index(
        &mut self,
        handle: FunctionHandleIndex,
        type_parameters: SignatureIndex,
    ) -> Result<FunctionInstantiationIndex> {
        let func_inst = FunctionInstantiation {
            handle,
            type_parameters,
        };
        Ok(FunctionInstantiationIndex(get_or_add_item(
            &mut self.function_instantiations,
            func_inst,
        )?))
    }

    /// Get the field instantiation index for the alias, adds it if missing.
    pub fn field_instantiation_index(
        &mut self,
        handle: FieldHandleIndex,
        type_parameters: SignatureIndex,
    ) -> Result<FieldInstantiationIndex> {
        let field_inst = FieldInstantiation {
            handle,
            type_parameters,
        };
        Ok(FieldInstantiationIndex(get_or_add_item(
            &mut self.field_instantiations,
            field_inst,
        )?))
    }

    /// Get the fake offset for the label. Labels will be fixed to real offsets after compilation
    pub fn label_index(&mut self, label: BlockLabel) -> Result<CodeOffset> {
        Ok(get_or_add_item(&mut self.labels, label)?)
    }

    /// Get the identifier pool index, adds it if missing.
    pub fn identifier_index(&mut self, s: &str) -> Result<IdentifierIndex> {
        let ident = ident_str(s)?;
        let m = &mut self.identifiers;
        let idx: Result<TableIndex> = get_or_add_item_macro!(m, ident, ident.to_owned());
        Ok(IdentifierIndex(idx?))
    }

    /// Get the address pool index, adds it if missing.
    pub fn address_index(&mut self, addr: AccountAddress) -> Result<AddressIdentifierIndex> {
        Ok(AddressIdentifierIndex(get_or_add_item(
            &mut self.address_identifiers,
            addr,
        )?))
    }

    /// Get the byte array pool index, adds it if missing.
    #[allow(clippy::ptr_arg)]
    pub fn constant_index(&mut self, constant: Constant) -> Result<ConstantPoolIndex> {
        Ok(ConstantPoolIndex(get_or_add_item(
            &mut self.constant_pool,
            constant,
        )?))
    }

    /// Get the field index, fails if it is not bound.
    pub fn field(
        &self,
        s: StructHandleIndex,
        f: Field_,
    ) -> Result<(StructDefinitionIndex, SignatureToken, usize)> {
        match self.fields.get(&(s, f.clone())) {
            None => bail!("Unbound field {}", f),
            Some((sd_idx, token, decl_order)) => Ok((*sd_idx, token.clone(), *decl_order)),
        }
    }

    /// Get the struct definition index, fails if it is not bound.
    pub fn struct_definition_index(&self, s: &StructName) -> Result<StructDefinitionIndex> {
        match self.struct_defs.get(&s) {
            None => bail!("Missing struct definition for {}", s),
            Some(idx) => Ok(StructDefinitionIndex(*idx)),
        }
    }

    /// Get the signature pool index, adds it if missing.
    pub fn signature_index(&mut self, sig: Signature) -> Result<SignatureIndex> {
        Ok(SignatureIndex(get_or_add_item(&mut self.signatures, sig)?))
    }

    pub fn set_function_index(&mut self, index: TableIndex) {
        self.current_function_index = FunctionDefinitionIndex(index);
    }

    pub fn current_function_definition_index(&self) -> FunctionDefinitionIndex {
        self.current_function_index
    }

    pub fn current_struct_definition_index(&self) -> StructDefinitionIndex {
        let idx = self.struct_defs.len();
        StructDefinitionIndex(idx as TableIndex)
    }

    //**********************************************************************************************
    // Declarations
    //**********************************************************************************************

    /// Add an import. This creates a module handle index for the imported module.
    pub fn declare_import(
        &mut self,
        id: QualifiedModuleIdent,
        alias: ModuleName,
    ) -> Result<ModuleHandleIndex> {
        // We don't care about duplicate aliases, if they exist
        self.aliases.insert(id.clone(), alias.clone());
        let address = self.address_index(id.address)?;
        let name = self.identifier_index(id.name.as_inner())?;
        self.modules
            .insert(alias.clone(), (id, ModuleHandle { address, name }));
        Ok(ModuleHandleIndex(get_or_add_item_ref(
            &mut self.module_handles,
            &self.modules.get(&alias).unwrap().1,
        )?))
    }

    /// Given an identifier and basic "signature" information, creates a struct handle
    /// and adds it to the pool.
    pub fn declare_struct_handle_index(
        &mut self,
        sname: QualifiedStructIdent,
        is_nominal_resource: bool,
        type_parameters: Vec<Kind>,
    ) -> Result<StructHandleIndex> {
        let module = self.module_handle_index(&sname.module)?;
        let name = self.identifier_index(sname.name.as_inner())?;
        self.structs.insert(
            sname.clone(),
            StructHandle {
                module,
                name,
                is_nominal_resource,
                type_parameters,
            },
        );
        Ok(StructHandleIndex(get_or_add_item_ref(
            &mut self.struct_handles,
            self.structs.get(&sname).unwrap(),
        )?))
    }

    /// Given an identifier, declare the struct definition index.
    pub fn declare_struct_definition_index(
        &mut self,
        s: StructName,
    ) -> Result<StructDefinitionIndex> {
        let idx = self.struct_defs.len();
        if idx > TABLE_MAX_SIZE {
            bail!("too many struct definitions {}", s)
        }
        // TODO: Add the decl of the struct definition name here
        // need to handle duplicates
        Ok(StructDefinitionIndex(
            *self.struct_defs.entry(s).or_insert(idx as TableIndex),
        ))
    }

    /// Given an identifier and a signature, creates a function handle and adds it to the pool.
    /// Finds the index for the signature, or adds it to the pool if an identical one has not yet
    /// been used.
    pub fn declare_function(
        &mut self,
        mname: ModuleName,
        fname: FunctionName,
        signature: FunctionSignature,
    ) -> Result<()> {
        let m_f = (mname.clone(), fname.clone());
        let module = self.module_handle_index(&mname)?;
        let name = self.identifier_index(fname.as_inner())?;

        self.function_signatures
            .insert(m_f.clone(), signature.clone());

        let FunctionSignature {
            return_,
            parameters,
            type_parameters,
        } = signature;

        let params_idx = get_or_add_item(&mut self.signatures, Signature(parameters))?;
        let return_idx = get_or_add_item(&mut self.signatures, Signature(return_))?;

        let handle = FunctionHandle {
            module,
            name,
            parameters: SignatureIndex(params_idx as TableIndex),
            return_: SignatureIndex(return_idx as TableIndex),
            type_parameters,
        };
        // handle duplicate declarations
        // erroring on duplicates needs to be done by the bytecode verifier
        let hidx = match self.function_handles.get(&m_f) {
            None => self.function_handles.len(),
            Some((_, idx)) => idx.0 as usize,
        };
        if hidx > TABLE_MAX_SIZE {
            bail!("too many functions: {}.{}", mname, fname)
        }
        let handle_index = FunctionHandleIndex(hidx as TableIndex);
        self.function_handles.insert(m_f, (handle, handle_index));

        Ok(())
    }

    /// Given a struct handle and a field, adds it to the pool.
    pub fn declare_field(
        &mut self,
        s: StructHandleIndex,
        sd_idx: StructDefinitionIndex,
        f: Field_,
        token: SignatureToken,
        decl_order: usize,
    ) {
        // need to handle duplicates
        self.fields
            .entry((s, f))
            .or_insert((sd_idx, token, decl_order));
    }

    //**********************************************************************************************
    // Dependency Resolution
    //**********************************************************************************************

    fn dependency(&self, m: &QualifiedModuleIdent) -> Result<&CompiledDependency> {
        self.dependencies
            .get(m)
            .ok_or_else(|| format_err!("Dependency not provided for {}", m))
    }

    fn dep_struct_handle(&mut self, s: &QualifiedStructIdent) -> Result<(bool, Vec<Kind>)> {
        if s.module.as_inner() == ModuleName::self_name() {
            bail!("Unbound struct {}", s)
        }
        let mident = self.module_ident(&s.module)?.clone();
        let dep = self.dependency(&mident)?;
        match dep.struct_handle(s) {
            None => bail!("Unbound struct {}", s),
            Some(shandle) => Ok((shandle.is_nominal_resource, shandle.type_parameters.clone())),
        }
    }

    /// Given an identifier, find the struct handle index.
    /// Creates the handle and adds it to the pool if it it is the *first* time it looks
    /// up the struct in a dependency.
    pub fn struct_handle_index(&mut self, s: QualifiedStructIdent) -> Result<StructHandleIndex> {
        match self.structs.get(&s) {
            Some(sh) => Ok(StructHandleIndex(*self.struct_handles.get(sh).unwrap())),
            None => {
                let (is_nominal_resource, type_parameters) = self.dep_struct_handle(&s)?;
                self.declare_struct_handle_index(s, is_nominal_resource, type_parameters)
            }
        }
    }

    fn reindex_signature_token(
        &mut self,
        dep: &QualifiedModuleIdent,
        orig: SignatureToken,
    ) -> Result<SignatureToken> {
        Ok(match orig {
            x @ SignatureToken::Bool
            | x @ SignatureToken::U8
            | x @ SignatureToken::U64
            | x @ SignatureToken::U128
            | x @ SignatureToken::Address
            | x @ SignatureToken::Signer
            | x @ SignatureToken::TypeParameter(_) => x,
            SignatureToken::Vector(inner) => {
                let correct_inner = self.reindex_signature_token(dep, *inner)?;
                SignatureToken::Vector(Box::new(correct_inner))
            }
            SignatureToken::Reference(inner) => {
                let correct_inner = self.reindex_signature_token(dep, *inner)?;
                SignatureToken::Reference(Box::new(correct_inner))
            }
            SignatureToken::MutableReference(inner) => {
                let correct_inner = self.reindex_signature_token(dep, *inner)?;
                SignatureToken::MutableReference(Box::new(correct_inner))
            }
            SignatureToken::Struct(orig_sh_idx) => {
                let dep_info = self.dependency(&dep)?;
                let (mident, sname) = dep_info
                    .source_struct_info(orig_sh_idx)
                    .ok_or_else(|| format_err!("Malformed dependency"))?;
                let module_name = self.module_alias(&mident)?.clone();
                let sident = QualifiedStructIdent {
                    module: module_name,
                    name: sname,
                };
                let correct_sh_idx = self.struct_handle_index(sident)?;
                SignatureToken::Struct(correct_sh_idx)
            }
            SignatureToken::StructInstantiation(orig_sh_idx, inners) => {
                let dep_info = self.dependency(&dep)?;
                let (mident, sname) = dep_info
                    .source_struct_info(orig_sh_idx)
                    .ok_or_else(|| format_err!("Malformed dependency"))?;
                let module_name = self.module_alias(&mident)?.clone();
                let sident = QualifiedStructIdent {
                    module: module_name,
                    name: sname,
                };
                let correct_sh_idx = self.struct_handle_index(sident)?;
                let correct_inners = inners
                    .into_iter()
                    .map(|t| self.reindex_signature_token(dep, t))
                    .collect::<Result<_>>()?;
                SignatureToken::StructInstantiation(correct_sh_idx, correct_inners)
            }
        })
    }

    fn reindex_function_signature(
        &mut self,
        dep: &QualifiedModuleIdent,
        orig: FunctionSignature,
    ) -> Result<FunctionSignature> {
        let return_ = orig
            .return_
            .into_iter()
            .map(|t| self.reindex_signature_token(dep, t))
            .collect::<Result<_>>()?;
        let parameters = orig
            .parameters
            .into_iter()
            .map(|t| self.reindex_signature_token(dep, t))
            .collect::<Result<_>>()?;
        let type_parameters = orig.type_parameters;
        Ok(FunctionSignature {
            return_,
            parameters,
            type_parameters,
        })
    }

    fn dep_function_signature(
        &mut self,
        m: &ModuleName,
        f: &FunctionName,
    ) -> Result<FunctionSignature> {
        if m.as_inner() == ModuleName::self_name() {
            bail!("Unbound function {}.{}", m, f)
        }
        let mident = self.module_ident(m)?.clone();
        let dep = self.dependency(&mident)?;
        match dep.function_signature(f) {
            None => bail!("Unbound function {}.{}", m, f),
            Some(sig) => self.reindex_function_signature(&mident, sig),
        }
    }

    fn ensure_function_declared(&mut self, m: ModuleName, f: FunctionName) -> Result<()> {
        let m_f = (m.clone(), f.clone());
        if !self.function_handles.contains_key(&m_f) {
            assert!(!self.function_signatures.contains_key(&m_f));
            let sig = self.dep_function_signature(&m, &f)?;
            self.declare_function(m, f, sig)?;
        }

        assert!(self.function_handles.contains_key(&m_f));
        assert!(self.function_signatures.contains_key(&m_f));
        Ok(())
    }

    /// Given an identifier, find the function handle and its index.
    /// Creates the handle+signature and adds it to the pool if it it is the *first* time it looks
    /// up the function in a dependency.
    pub fn function_handle(
        &mut self,
        m: ModuleName,
        f: FunctionName,
    ) -> Result<&(FunctionHandle, FunctionHandleIndex)> {
        self.ensure_function_declared(m.clone(), f.clone())?;
        Ok(self.function_handles.get(&(m, f)).unwrap())
    }

    /// Given an identifier, find the function signature and its index.
    /// Creates the handle+signature and adds it to the pool if it it is the *first* time it looks
    /// up the function in a dependency.
    pub fn function_signature(
        &mut self,
        m: ModuleName,
        f: FunctionName,
    ) -> Result<&FunctionSignature> {
        self.ensure_function_declared(m.clone(), f.clone())?;
        Ok(self.function_signatures.get(&(m, f)).unwrap())
    }
}
