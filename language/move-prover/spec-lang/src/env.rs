// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides an environment -- global state -- for translation, including helper functions
//! to interpret metadata about the translation target.

use std::cell::RefCell;

use codespan::{FileId, Files, Location, Span};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label, Severity},
    term::{emit, termcolor::WriteColor, Config},
};
use itertools::Itertools;
use num::{BigUint, Num};

use bytecode_source_map::source_map::ModuleSourceMap;
use libra_types::language_storage;
use vm::{
    access::ModuleAccess,
    file_format::{
        AddressPoolIndex, FieldDefinitionIndex, FunctionDefinitionIndex, FunctionHandleIndex, Kind,
        LocalsSignatureIndex, SignatureToken, StructDefinitionIndex, StructFieldInformation,
        StructHandleIndex,
    },
    views::{
        FieldDefinitionView, FunctionDefinitionView, FunctionHandleView, SignatureTokenView,
        StructDefinitionView, StructHandleView, ViewInternals,
    },
};

use crate::{
    ast::{Condition, Invariant, InvariantKind, ModuleName, SpecFunDecl, SpecVarDecl},
    symbol::{Symbol, SymbolPool},
    ty::{PrimitiveType, Type},
};
use std::collections::BTreeMap;
use vm::CompiledModule;

// =================================================================================================
/// # Locations

/// A location, consisting of a FileId and a span in this file.
#[derive(Debug, PartialEq, Clone)]
pub struct Loc {
    pub file_id: FileId,
    pub span: Span,
}

// =================================================================================================
/// # Identifiers
///
/// Identifiers are opaque values used to reference entities in the environment.
///
/// We have two kinds of ids: those based on an index, and those based on a symbol. We use
/// the symbol based ids where we do not have control of the definition index order in bytecode
/// (i.e. we do not know in which order move-lang enters functions and structs into file format),
/// and index based ids where we do have control (for modules, SpecFun and SpecVar).
///
/// In any case, ids are opaque in the sense that if someone has a StructId or similar in hand,
/// it is known to be defined in the environment, as it has been obtained also from the environment.

/// Raw index type used in ids. 16 bits are sufficient currently.
type RawIndex = u16;

/// Identifier for a module.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct ModuleId(RawIndex);

/// Identifier for a structure/resource, relative to module.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct StructId(Symbol);

/// Identifier for a field of a structure, relative to struct.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct FieldId(Symbol);

/// Identifier for a Move function, relative to module.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct FunId(Symbol);

/// Identifier for a specification function, relative to module.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SpecFunId(RawIndex);

/// Identifier for a specification variable, relative to module.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SpecVarId(RawIndex);

/// Identifier for a node in the AST, relative to a function. This is used to associate attributes
/// with the node, like source location and type.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct NodeId(RawIndex);

impl FunId {
    pub fn new(sym: Symbol) -> Self {
        Self(sym)
    }

    pub fn symbol(self) -> Symbol {
        self.0
    }
}

impl StructId {
    pub fn new(sym: Symbol) -> Self {
        Self(sym)
    }

    pub fn symbol(self) -> Symbol {
        self.0
    }
}

impl FieldId {
    pub fn new(sym: Symbol) -> Self {
        Self(sym)
    }

    pub fn symbol(self) -> Symbol {
        self.0
    }
}

impl SpecFunId {
    pub fn new(idx: usize) -> Self {
        Self(idx as RawIndex)
    }
}

impl SpecVarId {
    pub fn new(idx: usize) -> Self {
        Self(idx as RawIndex)
    }
}

impl NodeId {
    pub fn new(idx: usize) -> Self {
        Self(idx as RawIndex)
    }
}

impl ModuleId {
    pub fn new(idx: usize) -> Self {
        Self(idx as RawIndex)
    }
}

// =================================================================================================
/// # Global Environment

/// Global environment for a set of modules.
#[derive(Debug)]
pub struct GlobalEnv {
    /// A Files database for the codespan crate which supports diagnostics.
    source_files: Files<String>,
    /// A mapping from file names to associated FileId. Though this information is
    /// already in `source_files`, we can't get it out of there so need to book keep here.
    file_name_map: BTreeMap<String, FileId>,
    /// Accumulated diagnosis. In a RefCell so we can add to it without needing a mutable GlobalEnv.
    diags: RefCell<Vec<Diagnostic>>,
    /// Pool of symbols -- internalized strings.
    symbol_pool: SymbolPool,
    /// List of loaded modules, in order they have been provided using `add`.
    module_data: Vec<ModuleData>,
}

impl GlobalEnv {
    /// Creates a new environment.
    pub fn new() -> Self {
        GlobalEnv {
            source_files: Files::new(),
            file_name_map: BTreeMap::new(),
            diags: RefCell::new(vec![]),
            symbol_pool: SymbolPool::new(),
            module_data: vec![],
        }
    }

    /// Returns a reference to the symbol pool owned by this environment.
    pub fn symbol_pool(&self) -> &SymbolPool {
        &self.symbol_pool
    }

    /// Adds a source to this environment, returning a FileId for it.
    pub fn add_source(&mut self, file_name: &str, source: &str) -> FileId {
        let file_id = self.source_files.add(file_name, source.to_string());
        self.file_name_map.insert(file_name.to_string(), file_id);
        file_id
    }

    /// Adds diagnostic to the environment.
    pub fn add_diag(&self, diag: Diagnostic) {
        self.diags.borrow_mut().push(diag);
    }

    /// Adds an error to this environment, with notes.
    pub fn error_with_notes(&self, loc: &Loc, msg: &str, notes: Vec<String>) {
        let diag = Diagnostic::new_error(msg, Label::new(loc.file_id, loc.span, ""));
        let diag = diag.with_notes(notes);
        self.add_diag(diag);
    }

    /// Adds an error to this environment, without notes.
    pub fn error(&self, loc: &Loc, msg: &str) {
        self.error_with_notes(loc, msg, vec![]);
    }

    /// Returns a default location. This is used as location of intrinsics which do not have
    /// a real source location. This returns a location pointing to a source which has been added
    /// to the environment with `add_source`. It crashes if no source exists.
    pub fn default_loc(&self) -> Loc {
        let (_, file_id) = self
            .file_name_map
            .iter()
            .nth(0)
            .expect("no source added to the environment");
        Loc {
            file_id: *file_id,
            span: Span::default(),
        }
    }

    /// Converts a Loc as used by the move-lang compiler to the one we are using here.
    /// TODO: move-lang should use FileId as well so we don't need this here. There is already
    /// a todo in their code to remove the current use of `&'static str` for file names in Loc.
    pub fn to_loc(&self, loc: &move_ir_types::location::Loc) -> Loc {
        let file_id = self
            .file_name_map
            .get(loc.file())
            .expect("file name undefined");
        Loc {
            file_id: *file_id,
            span: loc.span(),
        }
    }

    /// Returns true if diagnostics have error severity or worse.
    pub fn has_errors(&self) -> bool {
        self.diags
            .borrow()
            .iter()
            .any(|d| d.severity >= Severity::Error)
    }

    /// Writes accumulated diagnostics to writer.
    pub fn report_errors<W: WriteColor>(&self, writer: &mut W) {
        for diag in self.diags.borrow().iter() {
            emit(writer, &Config::default(), &self.source_files, diag).expect("emit must not fail");
        }
    }

    /// Adds a new module to the environment. StructData and FunctionData need to be provided
    /// in definition index order. See `create_function_data` and `create_struct_data` for how
    /// to create them.
    pub fn add(
        &mut self,
        source_file_id: FileId,
        module: CompiledModule,
        source_map: ModuleSourceMap<Span>,
        struct_data: BTreeMap<StructId, StructData>,
        function_data: BTreeMap<FunId, FunctionData>,
        spec_vars: Vec<SpecVarDecl>,
        spec_funs: Vec<SpecFunDecl>,
        loc_map: BTreeMap<NodeId, Loc>,
        type_map: BTreeMap<NodeId, Type>,
    ) {
        let idx = self.module_data.len();
        let name = ModuleName::from_str(
            &module.self_id().address().to_string(),
            self.symbol_pool.make(module.self_id().name().as_str()),
        );
        self.module_data.push(ModuleData {
            name,
            id: ModuleId(idx as RawIndex),
            module,
            struct_data,
            function_data,
            spec_vars,
            spec_funs,
            source_map,
            source_file_id,
            loc_map,
            type_map,
        });
    }

    /// Creates data for a function, adding any information not contained in bytecode. This is
    /// a helper for adding a new module to the environment.
    pub fn create_function_data(
        &self,
        module: &CompiledModule,
        def_idx: FunctionDefinitionIndex,
        name: Symbol,
        arg_names: Vec<Symbol>,
        type_arg_names: Vec<Symbol>,
        spec: Vec<Condition>,
    ) -> FunctionData {
        let handle_idx = module.function_def_at(def_idx).function;
        FunctionData {
            name,
            def_idx,
            handle_idx,
            arg_names,
            type_arg_names,
            spec,
        }
    }

    /// Creates data for a struct. Currently all information is contained in the byte code. This is
    /// a helper for adding a new module to the environment.
    pub fn create_struct_data(
        &self,
        module: &CompiledModule,
        def_idx: StructDefinitionIndex,
        name: Symbol,
        invariants: Vec<Invariant>,
    ) -> StructData {
        let handle_idx = module.struct_def_at(def_idx).struct_handle;
        let field_data = if let StructFieldInformation::Declared {
            field_count,
            fields,
        } = module.struct_def_at(def_idx).field_information
        {
            let mut map = BTreeMap::new();
            for idx in fields.0..fields.0 + field_count {
                let def_idx = FieldDefinitionIndex(idx);
                let def = module.field_def_at(def_idx);
                let name = self
                    .symbol_pool
                    .make(module.identifier_at(def.name).as_str());
                map.insert(FieldId(name), FieldData { name, def_idx });
            }
            map
        } else {
            BTreeMap::new()
        };
        let mut data_invariants = vec![];
        let mut update_invariants = vec![];
        let mut pack_invariants = vec![];
        let mut unpack_invariants = vec![];

        for inv in invariants {
            match inv.kind {
                InvariantKind::Data => data_invariants.push(inv),
                InvariantKind::Update => update_invariants.push(inv),
                InvariantKind::Pack => pack_invariants.push(inv),
                InvariantKind::Unpack => unpack_invariants.push(inv),
            }
        }
        StructData {
            name,
            def_idx,
            handle_idx,
            field_data,
            data_invariants,
            update_invariants,
            pack_invariants,
            unpack_invariants,
        }
    }

    /// Finds a module by name and returns an environment for it.
    pub fn find_module(&self, name: &ModuleName) -> Option<ModuleEnv<'_>> {
        for module_data in &self.module_data {
            let module_env = ModuleEnv {
                env: self,
                data: module_data,
            };
            if module_env.get_name() == name {
                return Some(module_env);
            }
        }
        None
    }

    /// Finds a module by simple name and returns an environment for it.
    /// TODO: we may need to disallow this to support modules of the same simple name but with
    ///    different addresses in one verification session.
    pub fn find_module_by_name(&self, simple_name: Symbol) -> Option<ModuleEnv<'_>> {
        self.get_modules()
            .find(|m| m.get_name().name() == simple_name)
    }

    // Gets the number of modules in this environment.
    pub fn get_module_count(&self) -> usize {
        self.module_data.len()
    }

    /// Gets a module by id.
    pub fn get_module(&self, id: ModuleId) -> ModuleEnv<'_> {
        let module_data = &self.module_data[id.0 as usize];
        ModuleEnv {
            env: self,
            data: module_data,
        }
    }

    /// Returns an iterator for all modules in the environment.
    pub fn get_modules(&self) -> impl Iterator<Item = ModuleEnv<'_>> {
        self.module_data.iter().map(move |module_data| ModuleEnv {
            env: self,
            data: module_data,
        })
    }

    /// Returns an iterator for all bytecode modules in the environment.
    pub fn get_bytecode_modules(&self) -> impl Iterator<Item = &CompiledModule> {
        self.module_data
            .iter()
            .map(|module_data| &module_data.module)
    }

    /// Returns all structs in all modules which carry invariants.
    pub fn get_all_structs_with_invariants(&self) -> Vec<Type> {
        let mut res = vec![];
        for module_env in self.get_modules() {
            for struct_env in module_env.get_structs() {
                if struct_env.has_invariants() {
                    let formals = struct_env
                        .get_type_parameters()
                        .iter()
                        .enumerate()
                        .map(|(idx, _)| Type::TypeParameter(idx as u16))
                        .collect_vec();
                    res.push(Type::Struct(
                        module_env.get_id(),
                        struct_env.get_id(),
                        formals,
                    ));
                }
            }
        }
        res
    }

    /// Converts a storage module id into an AST module name.
    fn to_module_name(&self, storage_id: &language_storage::ModuleId) -> ModuleName {
        ModuleName::from_str(
            &storage_id.address().to_string(),
            self.symbol_pool.make(storage_id.name().as_str()),
        )
    }
}

impl Default for GlobalEnv {
    fn default() -> Self {
        Self::new()
    }
}

// =================================================================================================
/// # Module Environment

/// Represents data for a module.
#[derive(Debug)]
pub struct ModuleData {
    /// Module name.
    pub name: ModuleName,

    /// Id of this module in the global env.
    pub id: ModuleId,

    /// Module byte code.
    pub module: CompiledModule,

    /// Struct data.
    pub struct_data: BTreeMap<StructId, StructData>,

    /// Function data.
    pub function_data: BTreeMap<FunId, FunctionData>,

    /// Specification variables, in SpecVarId order.
    pub spec_vars: Vec<SpecVarDecl>,

    /// Specification functions, in SpecFunId order.
    pub spec_funs: Vec<SpecFunDecl>,

    /// Module source location information.
    pub source_map: ModuleSourceMap<Span>,

    /// A FileId for the `self.source_files` database.
    pub source_file_id: FileId,

    /// A map from node id to associated location.
    pub loc_map: BTreeMap<NodeId, Loc>,

    /// A map from node id to associated type.
    pub type_map: BTreeMap<NodeId, Type>,
}

/// Represents a module environment.
#[derive(Debug, Clone)]
pub struct ModuleEnv<'env> {
    /// Reference to the outer env.
    pub env: &'env GlobalEnv,

    /// Reference to the data of the module.
    data: &'env ModuleData,
}

impl<'env> ModuleEnv<'env> {
    /// Returns the id of this module in the global env.
    pub fn get_id(&self) -> ModuleId {
        self.data.id
    }

    /// Returns the name of this module.
    pub fn get_name(&'env self) -> &'env ModuleName {
        &self.data.name
    }

    /// Returns file name and line/column position for location in this module.
    pub fn get_position(&self, loc: Span) -> Option<(String, Location)> {
        self.env
            .source_files
            .location(self.data.source_file_id, loc.start())
            .ok()
            .map(|location| {
                (
                    self.env
                        .source_files
                        .name(self.data.source_file_id)
                        .to_string_lossy()
                        .to_string(),
                    location,
                )
            })
    }

    /// Gets the underlying bytecode module.
    pub fn get_verified_module(&'env self) -> &'env CompiledModule {
        &self.data.module
    }

    /// Gets a FunctionEnv in this module by name.
    pub fn find_function(&self, name: Symbol) -> Option<FunctionEnv<'env>> {
        let id = FunId(name);
        if let Some(data) = self.data.function_data.get(&id) {
            Some(FunctionEnv {
                module_env: self.clone(),
                data,
            })
        } else {
            None
        }
    }

    /// Gets a FunctionEnv by id.
    pub fn get_function(&'env self, id: FunId) -> FunctionEnv<'env> {
        let data = self.data.function_data.get(&id).expect("FunId undefined");
        FunctionEnv {
            module_env: self.clone(),
            data,
        }
    }

    /// Gets the number of functions in this module.
    pub fn get_function_count(&self) -> usize {
        self.data.function_data.len()
    }

    /// Returns iterator over FunctionEnvs in this module.
    pub fn get_functions(&'env self) -> impl Iterator<Item = FunctionEnv<'env>> {
        self.data
            .function_data
            .iter()
            .map(move |(_, data)| FunctionEnv {
                module_env: self.clone(),
                data,
            })
    }

    /// Returns the FunctionEnv which contains the location. This returns any function
    /// which location encloses the given one.
    pub fn get_enclosing_function(&'env self, loc: Span) -> Option<FunctionEnv<'env>> {
        // Currently we do a brute-force linear search, may need to speed this up if it appears
        // to be a bottleneck.
        for func_env in self.get_functions() {
            let fun_loc = func_env.get_loc();
            if loc.start() >= fun_loc.start() && loc.end() <= fun_loc.end() {
                return Some(func_env);
            }
        }
        None
    }

    /// Get FunctionEnv for a function used in this module, via the FunctionHandleIndex. The
    /// returned function might be from this or another module.
    pub fn get_called_function(&self, idx: FunctionHandleIndex) -> FunctionEnv<'_> {
        let view =
            FunctionHandleView::new(&self.data.module, self.data.module.function_handle_at(idx));
        let module_name = self.env.to_module_name(&view.module_id());
        let module_env = self
            .env
            .find_module(&module_name)
            .expect("unexpected reference to module not found in global env");
        module_env
            .find_function(self.env.symbol_pool.make(view.name().as_str()))
            .expect("unexpected reference to function not found in associated module")
    }

    /// Gets a StructEnv in this module by name.
    pub fn find_struct(&self, name: Symbol) -> Option<StructEnv<'_>> {
        let id = StructId(name);
        if let Some(data) = self.data.struct_data.get(&id) {
            Some(StructEnv {
                module_env: self.clone(),
                data,
            })
        } else {
            None
        }
    }

    /// Gets a StructEnv by id.
    pub fn get_struct(&self, id: StructId) -> StructEnv<'_> {
        let data = self.data.struct_data.get(&id).expect("StructId undefined");
        StructEnv {
            module_env: self.clone(),
            data,
        }
    }

    /// Gets a StructEnv by id, consuming this module env.
    pub fn into_get_struct(self, id: StructId) -> StructEnv<'env> {
        let data = self.data.struct_data.get(&id).expect("StructId undefined");
        StructEnv {
            module_env: self,
            data,
        }
    }

    /// Gets the number of structs in this module.
    pub fn get_struct_count(&self) -> usize {
        self.data.struct_data.len()
    }

    /// Gets the struct declaring a field specified by FieldDefinitionIndex,
    /// as it is globally unique for this module.
    pub fn get_struct_of_field(&self, idx: &FieldDefinitionIndex) -> StructEnv<'_> {
        let field_view =
            FieldDefinitionView::new(&self.data.module, self.data.module.field_def_at(*idx));
        let struct_name = self
            .env
            .symbol_pool
            .make(field_view.member_of().name().as_str());
        self.find_struct(struct_name).expect("struct undefined")
    }

    /// Returns iterator over structs in this module.
    pub fn get_structs(&'env self) -> impl Iterator<Item = StructEnv<'env>> {
        self.data
            .struct_data
            .iter()
            .map(move |(_, data)| StructEnv {
                module_env: self.clone(),
                data,
            })
    }

    /// Globalizes a signature local to this module.
    pub fn globalize_signature(&self, sig: &SignatureToken) -> Type {
        match sig {
            SignatureToken::Bool => Type::Primitive(PrimitiveType::Bool),
            SignatureToken::U8 => Type::Primitive(PrimitiveType::U8),
            SignatureToken::U64 => Type::Primitive(PrimitiveType::U64),
            SignatureToken::U128 => Type::Primitive(PrimitiveType::U128),
            SignatureToken::ByteArray => Type::Primitive(PrimitiveType::ByteArray),
            SignatureToken::Address => Type::Primitive(PrimitiveType::Address),
            SignatureToken::Reference(t) => {
                Type::Reference(false, Box::new(self.globalize_signature(&*t)))
            }
            SignatureToken::MutableReference(t) => {
                Type::Reference(true, Box::new(self.globalize_signature(&*t)))
            }
            SignatureToken::TypeParameter(index) => Type::TypeParameter(*index),
            SignatureToken::Vector(bt) => Type::Vector(Box::new(self.globalize_signature(&*bt))),
            SignatureToken::Struct(handle_idx, args) => {
                let struct_view = StructHandleView::new(
                    &self.data.module,
                    self.data.module.struct_handle_at(*handle_idx),
                );
                let declaring_module_env = self
                    .env
                    .find_module(&self.env.to_module_name(&struct_view.module_id()))
                    .expect("undefined module");
                let struct_env = declaring_module_env
                    .find_struct(self.env.symbol_pool.make(struct_view.name().as_str()))
                    .expect("undefined struct");
                Type::Struct(
                    declaring_module_env.data.id,
                    struct_env.get_id(),
                    self.globalize_signatures(args),
                )
            }
        }
    }

    /// Globalizes a list of signatures.
    fn globalize_signatures(&self, sigs: &[SignatureToken]) -> Vec<Type> {
        sigs.iter()
            .map(|s| self.globalize_signature(s))
            .collect_vec()
    }

    /// Gets a list of type actuals associated with the index in the bytecode.
    pub fn get_type_actuals(&self, idx: LocalsSignatureIndex) -> Vec<Type> {
        let actuals = &self.data.module.locals_signature_at(idx).0;
        self.globalize_signatures(actuals)
    }

    /// Converts an address pool index for this module into a number representing the address.
    pub fn get_address(&self, idx: AddressPoolIndex) -> BigUint {
        let addr = &self.data.module.address_pool()[idx.0 as usize];
        BigUint::from_str_radix(&addr.to_string(), 16).unwrap()
    }

    /// Returns specification variables of this module.
    pub fn get_spec_vars(&'env self) -> &'env [SpecVarDecl] {
        &self.data.spec_vars
    }

    /// Returns specification functions of this module.
    pub fn get_spec_funs(&'env self) -> &'env [SpecFunDecl] {
        &self.data.spec_funs
    }
}

// =================================================================================================
/// # Struct Environment

#[derive(Debug)]
pub struct StructData {
    /// The name of this struct.
    name: Symbol,

    /// The definition index of this struct in its module.
    def_idx: StructDefinitionIndex,

    /// The handle index of this struct in its module.
    handle_idx: StructHandleIndex,

    /// Field definitions.
    field_data: BTreeMap<FieldId, FieldData>,

    // Invariants
    data_invariants: Vec<Invariant>,
    update_invariants: Vec<Invariant>,
    pack_invariants: Vec<Invariant>,
    unpack_invariants: Vec<Invariant>,
}

#[derive(Debug, Clone)]
pub struct StructEnv<'env> {
    /// Reference to enclosing module.
    pub module_env: ModuleEnv<'env>,

    /// Reference to the struct data.
    data: &'env StructData,
}

impl<'env> StructEnv<'env> {
    /// Returns the name of this struct.
    pub fn get_name(&self) -> Symbol {
        self.data.name
    }

    /// Returns the location of this struct.
    pub fn get_loc(&self) -> Span {
        if let Ok(source_map) = self
            .module_env
            .data
            .source_map
            .get_struct_source_map(self.data.def_idx)
        {
            source_map.decl_location
        } else {
            Span::default()
        }
    }

    /// Gets the definition index associated with this struct.
    pub fn get_id(&self) -> StructId {
        StructId(self.data.name)
    }

    /// Determines whether this struct is native.
    pub fn is_native(&self) -> bool {
        let def = self.module_env.data.module.struct_def_at(self.data.def_idx);
        def.field_information == StructFieldInformation::Native
    }

    /// Determines whether this struct is the well-known vector type.
    pub fn is_vector(&self) -> bool {
        let name = self
            .module_env
            .env
            .symbol_pool
            .string(self.module_env.get_name().name());
        let addr = self.module_env.get_name().addr();
        name.as_ref() == "Vector" && addr == &BigUint::from(0 as u64)
    }

    /// Determines whether this struct is a resource type.
    pub fn is_resource(&self) -> bool {
        let def = self.module_env.data.module.struct_def_at(self.data.def_idx);
        let handle = self
            .module_env
            .data
            .module
            .struct_handle_at(def.struct_handle);
        handle.is_nominal_resource
    }

    /// Get an iterator for the fields.
    pub fn get_fields(&'env self) -> impl Iterator<Item = FieldEnv<'env>> {
        self.data.field_data.iter().map(move |(_, data)| FieldEnv {
            struct_env: self.clone(),
            data,
        })
    }

    /// Gets a field by its id.
    pub fn get_field(&'env self, id: FieldId) -> FieldEnv<'env> {
        let data = self.data.field_data.get(&id).expect("FieldId undefined");
        FieldEnv {
            struct_env: self.clone(),
            data,
        }
    }

    /// Find a field by its name.
    pub fn find_field(&'env self, name: Symbol) -> Option<FieldEnv<'env>> {
        let id = FieldId(name);
        if let Some(data) = self.data.field_data.get(&id) {
            Some(FieldEnv {
                struct_env: self.clone(),
                data,
            })
        } else {
            None
        }
    }

    /// Returns the type parameters associated with this struct.
    pub fn get_type_parameters(&self) -> Vec<TypeParameter> {
        // TODO: we currently do not know the original names of those formals, so we generate them.
        let view = StructDefinitionView::new(
            &self.module_env.data.module,
            self.module_env.data.module.struct_def_at(self.data.def_idx),
        );
        view.type_formals()
            .iter()
            .enumerate()
            .map(|(i, k)| {
                TypeParameter(
                    self.module_env.env.symbol_pool.make(&format!("tv{}", i)),
                    *k,
                )
            })
            .collect_vec()
    }

    /// Returns true if this struct has invariants.
    pub fn has_invariants(&self) -> bool {
        !self.data.data_invariants.is_empty()
            || !self.data.update_invariants.is_empty()
            || !self.data.pack_invariants.is_empty()
            || !self.data.unpack_invariants.is_empty()
    }

    /// Returns the data invariants associated with this struct.
    pub fn get_data_invariants(&'env self) -> &'env [Invariant] {
        &self.data.data_invariants
    }

    /// Returns the update invariants associated with this struct.
    pub fn get_update_invariants(&'env self) -> &'env [Invariant] {
        &self.data.update_invariants
    }

    /// Returns the pack invariants associated with this struct.
    pub fn get_pack_invariants(&'env self) -> &'env [Invariant] {
        &self.data.pack_invariants
    }

    /// Returns the unpack invariants associated with this struct.
    pub fn get_unpack_invariants(&'env self) -> &'env [Invariant] {
        &self.data.unpack_invariants
    }
}

// =================================================================================================
/// # Field Environment
#[derive(Debug)]
pub struct FieldData {
    /// The name of this field.
    name: Symbol,

    /// The definition index of this field in its module.
    def_idx: FieldDefinitionIndex,
}

#[derive(Debug)]
pub struct FieldEnv<'env> {
    /// Reference to enclosing struct.
    pub struct_env: StructEnv<'env>,

    /// Reference to the field data.
    data: &'env FieldData,
}

impl<'env> FieldEnv<'env> {
    /// Gets the name of this field.
    pub fn get_name(&self) -> Symbol {
        self.data.name
    }

    /// Gets the id of this field.
    pub fn get_id(&self) -> FieldId {
        FieldId(self.data.name)
    }

    /// Gets the type of this field.
    pub fn get_type(&self) -> Type {
        let view = FieldDefinitionView::new(
            &self.struct_env.module_env.data.module,
            self.struct_env
                .module_env
                .data
                .module
                .field_def_at(self.data.def_idx),
        );
        self.struct_env
            .module_env
            .globalize_signature(view.type_signature().token().as_inner())
    }
}

// =================================================================================================
/// # Function Environment

/// Represents a type parameter.
#[derive(Debug, Clone)]
pub struct TypeParameter(pub Symbol, pub Kind);

/// Represents a parameter.
#[derive(Debug, Clone)]
pub struct Parameter(pub Symbol, pub Type);

#[derive(Debug)]
pub struct FunctionData {
    /// Name of this function.
    name: Symbol,

    /// The definition index of this function in its module.
    def_idx: FunctionDefinitionIndex,

    /// The handle index of this function in its module.
    handle_idx: FunctionHandleIndex,

    /// List of function argument names. Not in bytecode but obtained from AST.
    arg_names: Vec<Symbol>,

    /// List of type argument names. Not in bytecode but obtained from AST.
    type_arg_names: Vec<Symbol>,

    /// List of specification conditions. Not in bytecode but obtained from AST.
    spec: Vec<Condition>,
}

#[derive(Debug)]
pub struct FunctionEnv<'env> {
    /// Reference to enclosing module.
    pub module_env: ModuleEnv<'env>,

    /// Reference to the function data.
    data: &'env FunctionData,
}

impl<'env> FunctionEnv<'env> {
    /// Returns the name of this function.
    pub fn get_name(&self) -> Symbol {
        self.data.name
    }

    /// Gets the id of this function.
    pub fn get_id(&self) -> FunId {
        FunId(self.data.name)
    }

    /// Returns the location of this function.
    pub fn get_loc(&self) -> Span {
        if let Ok(source_map) = self
            .module_env
            .data
            .source_map
            .get_function_source_map(self.data.def_idx)
        {
            source_map.decl_location
        } else {
            Span::default()
        }
    }

    /// Returns the location of the bytecode at the given offset.
    pub fn get_bytecode_loc(&self, offset: u16) -> Span {
        if let Ok(fmap) = self
            .module_env
            .data
            .source_map
            .get_function_source_map(self.data.def_idx)
        {
            if let Some(loc) = fmap.get_code_location(offset) {
                return loc;
            }
        }
        self.get_loc()
    }

    /// Returns true if this function is native.
    pub fn is_native(&self) -> bool {
        let view = self.definition_view();
        view.is_native()
    }

    /// Returns true if this function mutates any references (i.e. has &mut parameters).
    pub fn is_mutating(&self) -> bool {
        self.get_parameters()
            .iter()
            .any(|Parameter(_, ty)| ty.is_mutual_reference())
    }

    /// Returns the type parameters associated with this function.
    pub fn get_type_parameters(&self) -> Vec<TypeParameter> {
        // TODO: currently the translation scheme isn't working with using real type
        //   parameter names, so use indices instead.
        let view = self.definition_view();
        view.signature()
            .type_formals()
            .iter()
            .enumerate()
            .map(|(i, k)| {
                TypeParameter(
                    self.module_env.env.symbol_pool.make(&format!("tv{}", i)),
                    *k,
                )
            })
            .collect_vec()
    }

    /// Returns the regular parameters associated with this function.
    pub fn get_parameters(&self) -> Vec<Parameter> {
        let view = self.definition_view();
        view.signature()
            .arg_tokens()
            .map(|tv: SignatureTokenView<CompiledModule>| {
                self.module_env.globalize_signature(tv.signature_token())
            })
            .zip(self.data.arg_names.iter())
            .map(|(s, i)| Parameter(*i, s))
            .collect_vec()
    }

    /// Returns return types of this function.
    pub fn get_return_types(&self) -> Vec<Type> {
        let view = self.definition_view();
        view.signature()
            .return_tokens()
            .map(|tv: SignatureTokenView<CompiledModule>| {
                self.module_env.globalize_signature(tv.signature_token())
            })
            .collect_vec()
    }

    /// Returns the number of return values of this function.
    pub fn get_return_count(&self) -> usize {
        let view = self.definition_view();
        view.signature().return_count()
    }

    /// Get the name to be used for a local. If the local is an argument, use that for naming,
    /// otherwise generate a unique name.
    pub fn get_local_name(&self, idx: usize) -> Symbol {
        if idx < self.data.arg_names.len() {
            return self.data.arg_names[idx as usize];
        }
        // Try to obtain name from source map.
        if let Ok(fmap) = self
            .module_env
            .data
            .source_map
            .get_function_source_map(self.data.def_idx)
        {
            if let Some((ident, _)) = fmap.get_local_name(idx as u64) {
                return self.module_env.env.symbol_pool.make(ident.as_str());
            }
        }
        self.module_env.env.symbol_pool.make(&format!("__t{}", idx))
    }

    /// Gets the number of proper locals of this function. Those are locals which are declared
    /// by the user and also have a user assigned name which can be discovered via `get_local_name`.
    /// Note we may have more anonymous locals generated e.g by the 'stackless' transformation.
    pub fn get_local_count(&self) -> usize {
        let view = self.definition_view();
        view.locals_signature().len()
    }

    /// Gets the type of the local at index. This must use an index in the range as determined by
    /// `get_local_count`.
    /// TODO: we currently do not have a way to determine type of anonymous locals like those
    ///   generated by the stackless transformation via the environment. We may want to add a
    ///   feature to register such types with the environment to better support program
    ///   transformations.
    pub fn get_local_type(&self, idx: usize) -> Type {
        let view = self.definition_view();
        self.module_env.globalize_signature(
            view.locals_signature()
                .token_at(idx as u8)
                .signature_token(),
        )
    }

    /// Returns specification conditions associated with this function.
    pub fn get_specification(&'env self) -> &'env [Condition] {
        &self.data.spec
    }

    fn definition_view(&'env self) -> FunctionDefinitionView<'env, CompiledModule> {
        FunctionDefinitionView::new(
            &self.module_env.data.module,
            self.module_env
                .data
                .module
                .function_def_at(self.data.def_idx),
        )
    }
}
