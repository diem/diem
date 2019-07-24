// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cfgir::ast as G,
    errors::*,
    naming::ast::TParamID,
    parser::ast::{Field, FunctionName, ModuleIdent, StructName},
    shared::*,
};
use libra_types::{
    account_address::AccountAddress as LibraAddress, byte_array::ByteArray as LibraByteArray,
    identifier::Identifier as LibraIdentifier,
};
use move_vm::{
    file_format::{
        self as F, AddressPoolIndex, ByteArrayPoolIndex, FieldDefinitionIndex, FunctionHandle,
        FunctionHandleIndex, FunctionSignature, FunctionSignatureIndex, IdentifierIndex,
        LocalsSignature, LocalsSignatureIndex, ModuleHandle, ModuleHandleIndex, SignatureToken,
        StructDefinitionIndex, StructHandle, StructHandleIndex, TableIndex, TypeSignature,
        TypeSignatureIndex,
    },
    vm_string::VMString,
};
use std::{clone::Clone, collections::HashMap, hash::Hash};

#[macro_export]
macro_rules! bail {
    ($loc:expr, $msg:expr) => {{
        return Err(vec![($loc, $msg.into())]);
    }};
}

pub type Result<T> = std::result::Result<T, Error>;

macro_rules! get_or_add_item_macro {
    ($loc:expr, $m:ident, $k_get:expr, $k_insert:expr) => {{
        let k_key = $k_get;
        Ok(if $m.contains_key(k_key) {
            *$m.get(k_key).unwrap()
        } else {
            let len = $m.len();
            if len >= TABLE_MAX_SIZE {
                bail!($loc, "Max table size reached!")
            }
            let index = len as TableIndex;
            $m.insert($k_insert, index);
            index
        })
    }};
}

const TABLE_MAX_SIZE: usize = TableIndex::max_value() as usize;
fn get_or_add_item_ref<K: Clone + Eq + Hash>(
    loc: Loc,
    m: &mut HashMap<K, TableIndex>,
    k: &K,
) -> Result<TableIndex> {
    get_or_add_item_macro!(loc, m, k, k.clone())
}

fn get_or_add_item<K: Eq + Hash>(
    loc: Loc,
    m: &mut HashMap<K, TableIndex>,
    k: K,
) -> Result<TableIndex> {
    get_or_add_item_macro!(loc, m, &k, k)
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
    /// Type signature pool
    pub type_signatures: Vec<TypeSignature>,
    /// Function signature pool
    pub function_signatures: Vec<FunctionSignature>,
    /// Locals signatures pool
    pub locals_signatures: Vec<LocalsSignature>,
    /// Identifier pool
    pub identifiers: Vec<LibraIdentifier>,
    /// User string pool
    pub user_strings: Vec<VMString>,
    /// Byte array pool
    pub byte_array_pool: Vec<LibraByteArray>,
    /// Address pool
    pub address_pool: Vec<LibraAddress>,
}

/// Compilation context for a single compilation unit (module or script).
/// Contains all of the pools as they are built up.
/// Specific definitions to CompiledModule or CompiledScript are not stored.
/// However, some fields, like struct_defs and fields, are not used in CompiledScript.
pub struct Context<'a> {
    struct_declarations: &'a HashMap<(ModuleIdent, StructName), (bool, Vec<F::Kind>)>,
    function_declarations: &'a HashMap<(ModuleIdent, FunctionName), G::FunctionSignature>,

    // helpers
    modules: HashMap<ModuleIdent, ModuleHandle>,
    functions: HashMap<(ModuleIdent, FunctionName), FunctionHandle>,
    structs: HashMap<(ModuleIdent, StructName), StructHandle>,
    struct_defs: HashMap<StructName, TableIndex>,

    // queryable pools
    fields: HashMap<(StructDefinitionIndex, Field), TableIndex>,

    // Simple pools
    function_signatures: HashMap<FunctionSignature, TableIndex>,
    module_handles: HashMap<ModuleHandle, TableIndex>,
    struct_handles: HashMap<StructHandle, TableIndex>,
    function_handles: HashMap<FunctionHandle, TableIndex>,
    type_signatures: HashMap<TypeSignature, TableIndex>,
    locals_signatures: HashMap<LocalsSignature, TableIndex>,
    identifiers: HashMap<String, TableIndex>,
    byte_array_pool: HashMap<Vec<u8>, TableIndex>,
    address_pool: HashMap<Address, TableIndex>,

    // Current type parameter context
    type_parameters: HashMap<TParamID, TableIndex>,
}

impl<'a> Context<'a> {
    /// Given the dependencies and the current module, creates an empty context.
    /// The current module is a dummy `Self` for CompiledScript.
    /// It initializes an "import" of `Self` as the alias for the current_module.
    pub fn new(
        current_module: &ModuleIdent,
        struct_declarations: &'a HashMap<(ModuleIdent, StructName), (bool, Vec<F::Kind>)>,
        function_declarations: &'a HashMap<(ModuleIdent, FunctionName), G::FunctionSignature>,
    ) -> Result<Self> {
        let mut context = Self {
            struct_declarations,
            function_declarations,
            modules: HashMap::new(),
            functions: HashMap::new(),
            structs: HashMap::new(),
            struct_defs: HashMap::new(),
            fields: HashMap::new(),
            function_handles: HashMap::new(),
            function_signatures: HashMap::new(),
            module_handles: HashMap::new(),
            struct_handles: HashMap::new(),
            type_signatures: HashMap::new(),
            locals_signatures: HashMap::new(),
            identifiers: HashMap::new(),
            byte_array_pool: HashMap::new(),
            address_pool: HashMap::new(),
            type_parameters: HashMap::new(),
        };
        assert!(context.module_handle_index(current_module)?.0 == 0);
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
    pub fn materialize_pools(self) -> MaterializedPools {
        let identifiers = self
            .identifiers
            .into_iter()
            .map(|(s, idx)| (LibraIdentifier::new(s).unwrap(), idx))
            .collect();
        let addresses = self
            .address_pool
            .into_iter()
            .map(|(addr, idx)| (LibraAddress::new(addr.to_u8()), idx))
            .collect();
        let byte_arrays = self
            .byte_array_pool
            .into_iter()
            .map(|(bytes, idx)| (LibraByteArray::new(bytes), idx))
            .collect();
        MaterializedPools {
            function_signatures: Self::materialize_map(self.function_signatures),
            module_handles: Self::materialize_map(self.module_handles),
            struct_handles: Self::materialize_map(self.struct_handles),
            function_handles: Self::materialize_map(self.function_handles),
            type_signatures: Self::materialize_map(self.type_signatures),
            locals_signatures: Self::materialize_map(self.locals_signatures),
            identifiers: Self::materialize_map(identifiers),
            // TODO: implement support for user strings (string literals)
            user_strings: vec![],
            byte_array_pool: Self::materialize_map(byte_arrays),
            address_pool: Self::materialize_map(addresses),
        }
    }

    /// Bind the type parameters into a "pool" for the current context.
    pub fn bind_type_parameters(
        &mut self,
        loc: Loc,
        m: HashMap<TParamID, usize>,
    ) -> Result<HashMap<TParamID, TableIndex>> {
        let new_tmap = m
            .into_iter()
            .map(|(k, idx)| {
                if idx >= TABLE_MAX_SIZE {
                    bail!(loc, "Too many type parameters")
                }
                Ok((k, idx as TableIndex))
            })
            .collect::<Result<_>>()?;
        Ok(std::mem::replace(&mut self.type_parameters, new_tmap))
    }

    //**********************************************************************************************
    // Pools
    //**********************************************************************************************

    /// Get the module handle index for the alias
    pub fn module_handle_index(&mut self, module_ident: &ModuleIdent) -> Result<ModuleHandleIndex> {
        if !self.modules.contains_key(module_ident) {
            let address = self.address_index(module_ident.loc(), module_ident.0.value.address)?;
            let name = self.identifier_index(&module_ident.0.value.name)?;
            let handle = ModuleHandle { address, name };
            get_or_add_item(module_ident.loc(), &mut self.module_handles, handle.clone())?;
            self.modules.insert(module_ident.clone(), handle);
        }
        let handle = self.modules.get(module_ident).unwrap();
        Ok(ModuleHandleIndex(*self.module_handles.get(handle).unwrap()))
    }

    /// Get the type formal index, fails if it is not bound.
    pub fn type_formal_index(&mut self, t: TParamID) -> TableIndex {
        match self.type_parameters.get(&t) {
            None => panic!("Unbound type parameter"),
            Some(idx) => *idx,
        }
    }

    /// Get the address pool index, adds it if missing.
    pub fn address_index(&mut self, loc: Loc, addr: Address) -> Result<AddressPoolIndex> {
        Ok(AddressPoolIndex(get_or_add_item(
            loc,
            &mut self.address_pool,
            addr,
        )?))
    }

    /// Get the identifier pool index, adds it if missing.
    pub fn identifier_index<T: Identifier>(&mut self, ident: &T) -> Result<IdentifierIndex> {
        let m = &mut self.identifiers;
        let idx: Result<TableIndex> =
            get_or_add_item_macro!(ident.loc(), m, ident.value(), ident.value().to_owned());
        Ok(IdentifierIndex(idx?))
    }

    /// Get the byte array pool index, adds it if missing.
    pub fn byte_array_index(
        &mut self,
        loc: Loc,
        byte_array: Vec<u8>,
    ) -> Result<ByteArrayPoolIndex> {
        Ok(ByteArrayPoolIndex(get_or_add_item(
            loc,
            &mut self.byte_array_pool,
            byte_array,
        )?))
    }

    /// Get the field index, fails if it is not bound.
    pub fn field(&self, s: StructDefinitionIndex, f: Field) -> FieldDefinitionIndex {
        match self.fields.get(&(s, f.clone())) {
            None => panic!("Unbound field {}", f),
            Some(idx) => FieldDefinitionIndex(*idx),
        }
    }

    /// Get the type signature index, adds it if it is not bound.
    pub fn type_signature_index(
        &mut self,
        loc: Loc,
        token: SignatureToken,
    ) -> Result<TypeSignatureIndex> {
        Ok(TypeSignatureIndex(get_or_add_item(
            loc,
            &mut self.type_signatures,
            TypeSignature(token),
        )?))
    }

    /// Get the struct definition index, fails if it is not bound.
    pub fn struct_definition_index(&self, s: &StructName) -> StructDefinitionIndex {
        match self.struct_defs.get(&s) {
            None => panic!("Missing struct definition for {}", s),
            Some(idx) => StructDefinitionIndex(*idx),
        }
    }

    /// Get the locals signature pool index, adds it if missing.
    pub fn locals_signature_index(
        &mut self,
        loc: Loc,
        locals: LocalsSignature,
    ) -> Result<LocalsSignatureIndex> {
        Ok(LocalsSignatureIndex(get_or_add_item(
            loc,
            &mut self.locals_signatures,
            locals,
        )?))
    }

    //**********************************************************************************************
    // Declarations
    //**********************************************************************************************

    /// Given an identifier, declare the struct definition index.
    pub fn declare_struct_definition_index(
        &mut self,
        s: StructName,
    ) -> Result<StructDefinitionIndex> {
        let idx = self.struct_defs.len();
        if idx >= TABLE_MAX_SIZE {
            bail!(s.loc(), format!("too many struct definitions {}", s))
        }
        assert!(self.struct_defs.insert(s, idx as TableIndex).is_none());
        Ok(StructDefinitionIndex(idx as TableIndex))
    }

    /// Given a struct handle and a field, adds it to the pool.
    pub fn declare_field(
        &mut self,
        s: StructDefinitionIndex,
        f: Field,
    ) -> Result<FieldDefinitionIndex> {
        let idx = self.fields.len();
        if idx >= TABLE_MAX_SIZE {
            bail!(f.loc(), format!("too many fields: {}.{}", s, f))
        }
        assert!(self.fields.insert((s, f), idx as TableIndex).is_none());
        Ok(FieldDefinitionIndex(idx as TableIndex))
    }

    //**********************************************************************************************
    // Dependency Resolution
    //**********************************************************************************************

    /// Given an identifier and a signature, creates a function handle and adds it to the pool.
    /// Finds the index for the signature, or adds it to the pool if an identical one has not yet
    /// been used.
    pub fn declare_function(
        &mut self,
        mname: ModuleIdent,
        fname: FunctionName,
        signature: FunctionSignature,
    ) -> Result<FunctionHandleIndex> {
        let module = self.module_handle_index(&mname)?;
        let name = self.identifier_index(&fname)?;

        let sidx = get_or_add_item(fname.loc(), &mut self.function_signatures, signature)?;
        let signature_index = FunctionSignatureIndex(sidx);

        let m_f = (mname.clone(), fname.clone());
        let handle = FunctionHandle {
            module,
            name,
            signature: signature_index,
        };
        assert!(self.functions.insert(m_f, handle.clone()).is_none());

        let hidx = self.function_handles.len();
        if hidx >= TABLE_MAX_SIZE {
            bail!(
                fname.loc(),
                format!("too many functions: {}.{}", module, fname)
            )
        }
        assert!(self
            .function_handles
            .insert(handle, hidx as TableIndex)
            .is_none());
        Ok(FunctionHandleIndex(hidx as TableIndex))
    }

    pub fn declaration_function_signature(
        &self,
        mname: &ModuleIdent,
        fname: &FunctionName,
    ) -> G::FunctionSignature {
        self.function_declarations
            .get(&(mname.clone(), fname.clone()))
            .unwrap()
            .clone()
    }

    pub fn function_handle_index(
        &mut self,
        module_ident: &ModuleIdent,
        fname: &FunctionName,
    ) -> Option<FunctionHandleIndex> {
        let handle = self.functions.get(&(module_ident.clone(), fname.clone()))?;
        Some(FunctionHandleIndex(
            *self.function_handles.get(handle).unwrap(),
        ))
    }

    /// Given an identifier and basic "signature" information, creates a struct handle
    /// and adds it to the pool.
    fn declare_struct_handle_index(
        &mut self,
        mname: &ModuleIdent,
        sname: &StructName,
        is_nominal_resource: bool,
        type_formals: Vec<F::Kind>,
    ) -> Result<StructHandleIndex> {
        let module = self.module_handle_index(mname)?;
        let name = self.identifier_index(sname)?;
        let m_s = (mname.clone(), sname.clone());
        assert!(self
            .structs
            .insert(
                m_s.clone(),
                StructHandle {
                    module,
                    name,
                    is_nominal_resource,
                    type_formals,
                },
            )
            .is_none());
        Ok(StructHandleIndex(get_or_add_item_ref(
            sname.loc(),
            &mut self.struct_handles,
            self.structs.get(&m_s).unwrap(),
        )?))
    }

    fn struct_handle_opt(
        &self,
        module_ident: &ModuleIdent,
        sname: &StructName,
    ) -> Option<StructHandleIndex> {
        let handle = self.structs.get(&(module_ident.clone(), sname.clone()))?;
        Some(StructHandleIndex(*self.struct_handles.get(handle).unwrap()))
    }

    pub fn struct_handle_index(
        &mut self,
        module_ident: &ModuleIdent,
        sname: &StructName,
    ) -> Result<StructHandleIndex> {
        match self.struct_handle_opt(module_ident, sname) {
            Some(idx) => Ok(idx),
            None => {
                let (is_nominal_resource, kinds) = self
                    .struct_declarations
                    .get(&(module_ident.clone(), sname.clone()))
                    .unwrap()
                    .clone();
                self.declare_struct_handle_index(module_ident, sname, is_nominal_resource, kinds)
            }
        }
    }
}
