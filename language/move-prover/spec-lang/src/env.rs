// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides an environment -- global state -- for translation, including helper functions
//! to interpret metadata about the translation target.

#[allow(unused_imports)]
use log::info;

use std::cell::RefCell;

use codespan::{ByteIndex, ByteOffset, FileId, Files, Location, Span};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label, Severity},
    term::{emit, termcolor::WriteColor, Config},
};
use itertools::Itertools;
use num::{BigUint, Num};

use bytecode_source_map::source_map::SourceMap;
use libra_types::language_storage;
use vm::{
    access::ModuleAccess,
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, CodeOffset, FunctionDefinitionIndex,
        FunctionHandleIndex, Kind, SignatureIndex, SignatureToken, StructDefinitionIndex,
        StructFieldInformation, StructHandleIndex,
    },
    views::{
        FunctionDefinitionView, FunctionHandleView, SignatureTokenView, StructDefinitionView,
        StructHandleView,
    },
};

use crate::{
    ast::{Condition, FunSpec, Invariant, InvariantKind, ModuleName, SpecFunDecl, SpecVarDecl},
    symbol::{Symbol, SymbolPool},
    ty::{PrimitiveType, Type},
};
use std::collections::BTreeMap;
use vm::CompiledModule;

// =================================================================================================
/// # Locations

/// A location, consisting of a FileId and a span in this file.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Loc {
    file_id: FileId,
    span: Span,
}

impl Loc {
    pub fn new(file_id: FileId, span: Span) -> Loc {
        Loc { file_id, span }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn file_id(&self) -> FileId {
        self.file_id
    }

    // Delivers a location pointing to the end of this one.
    pub fn at_end(&self) -> Loc {
        if self.span.end() > ByteIndex(0) {
            Loc::new(
                self.file_id,
                Span::new(self.span.end() - ByteOffset(1), self.span.end()),
            )
        } else {
            self.clone()
        }
    }

    // Delivers a location pointing to the start of this one.
    pub fn at_start(&self) -> Loc {
        Loc::new(
            self.file_id,
            Span::new(self.span.start(), self.span.start() + ByteOffset(1)),
        )
    }
}

/// Alias for the Loc variant of MoveIR. This uses a `&static str` instead of `FileId` for the
/// file name.
pub type MoveIrLoc = move_ir_types::location::Loc;

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

/// Identifier for a node in the AST, relative to a module. This is used to associate attributes
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

    pub fn to_usize(self) -> usize {
        self.0 as usize
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
    /// Bijective mapping between FileId and a plain int. FileId's are themselves wrappers around
    /// ints, but the inner representation is opaque and cannot be accessed. This is used so we
    /// can emit FileId's to generated code and read them back.
    file_id_to_idx: BTreeMap<FileId, u16>,
    file_idx_to_id: BTreeMap<u16, FileId>,
    /// A special constant location representing an unknown location.
    /// This uses a pseudo entry in `source_files` to be safely represented.
    unknown_loc: Loc,
    /// An equivalent of the MoveIrLoc to the above location. Used to map back and force between
    /// them.
    unknown_move_ir_loc: MoveIrLoc,
    /// A special constant location representing an opaque location.
    /// In difference to an `unknown_loc`, this is a well-known but undisclosed location.
    internal_loc: Loc,
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
        let mut source_files = Files::new();
        let mut file_name_map = BTreeMap::new();
        let mut file_id_to_idx = BTreeMap::new();
        let mut file_idx_to_id = BTreeMap::new();
        let mut fake_loc = |content: &str| {
            let file_id = source_files.add(content, content.to_string());
            file_name_map.insert(content.to_string(), file_id);
            let file_idx = file_id_to_idx.len() as u16;
            file_id_to_idx.insert(file_id, file_idx);
            file_idx_to_id.insert(file_idx, file_id);
            Loc::new(
                file_id,
                Span::from(ByteIndex(0 as u32)..ByteIndex(content.len() as u32)),
            )
        };
        let unknown_loc = fake_loc("<unknown>");
        let unknown_move_ir_loc = MoveIrLoc::new("<unknown>", Span::default());
        let internal_loc = fake_loc("<internal>");
        GlobalEnv {
            source_files,
            unknown_loc,
            unknown_move_ir_loc,
            internal_loc,
            file_name_map,
            file_id_to_idx,
            file_idx_to_id,
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
        let file_idx = self.file_id_to_idx.len() as u16;
        self.file_id_to_idx.insert(file_id, file_idx);
        self.file_idx_to_id.insert(file_idx, file_id);
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

    /// Returns the unknown location.
    pub fn unknown_loc(&self) -> Loc {
        self.unknown_loc.clone()
    }

    /// Returns a Move IR version of the unknown location which is guaranteed to map to the
    /// regular unknown location via `to_loc`.
    pub fn unknown_move_ir_loc(&self) -> MoveIrLoc {
        self.unknown_move_ir_loc
    }

    /// Returns the internal location.
    pub fn internal_loc(&self) -> Loc {
        self.internal_loc.clone()
    }

    /// Converts a Loc as used by the move-lang compiler to the one we are using here.
    /// TODO: move-lang should use FileId as well so we don't need this here. There is already
    /// a todo in their code to remove the current use of `&'static str` for file names in Loc.
    pub fn to_loc(&self, loc: &MoveIrLoc) -> Loc {
        let file_id = self
            .file_name_map
            .get(loc.file())
            .expect("file name undefined");
        Loc {
            file_id: *file_id,
            span: loc.span(),
        }
    }

    /// Maps a FileId to an index which can be mapped back to a FileId.
    pub fn file_id_to_idx(&self, file_id: FileId) -> u16 {
        *self
            .file_id_to_idx
            .get(&file_id)
            .expect("file_id undefined")
    }

    /// Maps a an index which was obtained by `file_id_to_idx` back to a FileId.
    pub fn file_idx_to_id(&self, file_idx: u16) -> FileId {
        *self
            .file_idx_to_id
            .get(&file_idx)
            .expect("file_idx undefined")
    }

    /// Returns file name and line/column position for a location, if available.
    pub fn get_position(&self, loc: Loc) -> Option<(String, Location)> {
        self.source_files
            .location(loc.file_id(), loc.span().start())
            .ok()
            .map(|line_column| {
                (
                    self.source_files
                        .name(loc.file_id())
                        .to_string_lossy()
                        .to_string(),
                    line_column,
                )
            })
    }

    // Gets the number of source files in this environment.
    pub fn get_file_count(&self) -> usize {
        self.file_name_map.len()
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
        loc: Loc,
        module: CompiledModule,
        source_map: SourceMap<MoveIrLoc>,
        struct_data: BTreeMap<StructId, StructData>,
        function_data: BTreeMap<FunId, FunctionData>,
        spec_vars: Vec<SpecVarDecl>,
        spec_funs: Vec<SpecFunDecl>,
        module_invariants: Vec<Invariant>,
        loc_map: BTreeMap<NodeId, Loc>,
        type_map: BTreeMap<NodeId, Type>,
        instantiation_map: BTreeMap<NodeId, Vec<Type>>,
    ) {
        let idx = self.module_data.len();
        let name = ModuleName::from_str(
            &module.self_id().address().to_string(),
            self.symbol_pool.make(module.self_id().name().as_str()),
        );
        let struct_idx_to_id: BTreeMap<StructDefinitionIndex, StructId> = struct_data
            .iter()
            .map(|(id, data)| (data.def_idx, *id))
            .collect();
        let function_idx_to_id: BTreeMap<FunctionDefinitionIndex, FunId> = function_data
            .iter()
            .map(|(id, data)| (data.def_idx, *id))
            .collect();
        let spec_vars: BTreeMap<SpecVarId, SpecVarDecl> = spec_vars
            .into_iter()
            .enumerate()
            .map(|(i, v)| (SpecVarId::new(i), v))
            .collect();
        let spec_funs: BTreeMap<SpecFunId, SpecFunDecl> = spec_funs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (SpecFunId::new(i), v))
            .collect();

        self.module_data.push(ModuleData {
            name,
            id: ModuleId(idx as RawIndex),
            module,
            struct_data,
            struct_idx_to_id,
            function_data,
            function_idx_to_id,
            spec_vars,
            spec_funs,
            module_invariants,
            source_map,
            loc,
            loc_map,
            type_map,
            instantiation_map,
        });
    }

    /// Creates data for a function, adding any information not contained in bytecode. This is
    /// a helper for adding a new module to the environment.
    pub fn create_function_data(
        &self,
        module: &CompiledModule,
        def_idx: FunctionDefinitionIndex,
        name: Symbol,
        loc: Loc,
        arg_names: Vec<Symbol>,
        type_arg_names: Vec<Symbol>,
        spec: FunSpec,
    ) -> FunctionData {
        let handle_idx = module.function_def_at(def_idx).function;
        FunctionData {
            name,
            loc,
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
        loc: Loc,
        invariants: Vec<Invariant>,
    ) -> StructData {
        let handle_idx = module.struct_def_at(def_idx).struct_handle;
        let field_data = if let StructFieldInformation::Declared(fields) =
            &module.struct_def_at(def_idx).field_information
        {
            let mut map = BTreeMap::new();
            for (offset, field) in fields.iter().enumerate() {
                let name = self
                    .symbol_pool
                    .make(module.identifier_at(field.name).as_str());
                map.insert(
                    FieldId(name),
                    FieldData {
                        name,
                        def_idx,
                        offset,
                    },
                );
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
                _ => panic!("unexpected invariant kind"),
            }
        }
        StructData {
            name,
            loc,
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

    /// Return the module enclosing this location.
    pub fn get_enclosing_module(&self, loc: Loc) -> Option<ModuleEnv<'_>> {
        for data in &self.module_data {
            if data.loc.file_id() == loc.file_id()
                && Self::enclosing_span(data.loc.span(), loc.span())
            {
                return Some(ModuleEnv { env: self, data });
            }
        }
        None
    }

    /// Returns the function enclosing this location.
    pub fn get_enclosing_function(&self, loc: Loc) -> Option<FunctionEnv<'_>> {
        // Currently we do a brute-force linear search, may need to speed this up if it appears
        // to be a bottleneck.
        let module_env = self.get_enclosing_module(loc.clone())?;
        for func_env in module_env.into_functions() {
            if Self::enclosing_span(func_env.get_loc().span(), loc.span()) {
                return Some(func_env.clone());
            }
        }
        None
    }

    /// Returns the struct enclosing this location.
    pub fn get_enclosing_struct(&self, loc: Loc) -> Option<StructEnv<'_>> {
        let module_env = self.get_enclosing_module(loc.clone())?;
        for struct_env in module_env.into_structs() {
            if Self::enclosing_span(struct_env.get_loc().span(), loc.span()) {
                return Some(struct_env);
            }
        }
        None
    }

    fn enclosing_span(outer: Span, inner: Span) -> bool {
        inner.start() >= outer.start() && inner.end() <= outer.end()
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

    /// Mapping from struct definition index to id in above map.
    pub struct_idx_to_id: BTreeMap<StructDefinitionIndex, StructId>,

    /// Function data.
    pub function_data: BTreeMap<FunId, FunctionData>,

    /// Mapping from function definition index to id in above map.
    pub function_idx_to_id: BTreeMap<FunctionDefinitionIndex, FunId>,

    /// Specification variables, in SpecVarId order.
    pub spec_vars: BTreeMap<SpecVarId, SpecVarDecl>,

    /// Specification functions, in SpecFunId order.
    pub spec_funs: BTreeMap<SpecFunId, SpecFunDecl>,

    /// Module level invariants.
    pub module_invariants: Vec<Invariant>,

    /// Module source location information.
    pub source_map: SourceMap<MoveIrLoc>,

    /// The location of this module.
    pub loc: Loc,

    /// A map from node id to associated location.
    pub loc_map: BTreeMap<NodeId, Loc>,

    /// A map from node id to associated type.
    pub type_map: BTreeMap<NodeId, Type>,

    /// A map from node id to associated instantiation of type parameters.
    pub instantiation_map: BTreeMap<NodeId, Vec<Type>>,
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

    /// Shortcut for accessing the symbol pool.
    pub fn symbol_pool(&self) -> &SymbolPool {
        &self.env.symbol_pool
    }

    /// Gets the underlying bytecode module.
    pub fn get_verified_module(&'env self) -> &'env CompiledModule {
        &self.data.module
    }

    /// Gets a FunctionEnv in this module by name.
    pub fn find_function(&self, name: Symbol) -> Option<FunctionEnv<'env>> {
        let id = FunId(name);
        self.data.function_data.get(&id).map(|data| FunctionEnv {
            module_env: self.clone(),
            data,
        })
    }

    /// Gets a FunctionEnv by id.
    pub fn get_function(&'env self, id: FunId) -> FunctionEnv<'env> {
        self.clone().into_function(id)
    }

    /// Gets a FunctionEnv by id.
    pub fn into_function(self, id: FunId) -> FunctionEnv<'env> {
        let data = self.data.function_data.get(&id).expect("FunId undefined");
        FunctionEnv {
            module_env: self,
            data,
        }
    }

    /// Gets the number of functions in this module.
    pub fn get_function_count(&self) -> usize {
        self.data.function_data.len()
    }

    /// Returns iterator over FunctionEnvs in this module.
    pub fn get_functions(&'env self) -> impl Iterator<Item = FunctionEnv<'env>> {
        self.clone().into_functions()
    }

    /// Returns iterator over FunctionEnvs in this module.
    pub fn into_functions(self) -> impl Iterator<Item = FunctionEnv<'env>> {
        self.data
            .function_data
            .iter()
            .map(move |(_, data)| FunctionEnv {
                module_env: self.clone(),
                data,
            })
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
        module_env.into_function(FunId::new(self.env.symbol_pool.make(view.name().as_str())))
    }

    /// Gets a StructEnv in this module by name.
    pub fn find_struct(&self, name: Symbol) -> Option<StructEnv<'_>> {
        let id = StructId(name);
        self.data.struct_data.get(&id).map(|data| StructEnv {
            module_env: self.clone(),
            data,
        })
    }

    /// Gets the struct id from a definition index which must be valid for this environment.
    pub fn get_struct_id(&self, idx: StructDefinitionIndex) -> StructId {
        *self
            .data
            .struct_idx_to_id
            .get(&idx)
            .expect("undefined struct definition index")
    }

    /// Gets a StructEnv by id.
    pub fn get_struct(&self, id: StructId) -> StructEnv<'_> {
        let data = self.data.struct_data.get(&id).expect("StructId undefined");
        StructEnv {
            module_env: self.clone(),
            data,
        }
    }

    pub fn get_struct_by_def_idx(&self, idx: StructDefinitionIndex) -> StructEnv<'_> {
        self.get_struct(self.get_struct_id(idx))
    }

    /// Gets the function id from a definition index which must be valid for this environment.
    pub fn get_function_id(&self, idx: FunctionDefinitionIndex) -> FunId {
        *self
            .data
            .function_idx_to_id
            .get(&idx)
            .expect("undefined function definition index")
    }

    /// Gets a StructEnv by id, consuming this module env.
    pub fn into_struct(self, id: StructId) -> StructEnv<'env> {
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

    /// Returns iterator over structs in this module.
    pub fn get_structs(&'env self) -> impl Iterator<Item = StructEnv<'env>> {
        self.clone().into_structs()
    }

    /// Returns iterator over structs in this module.
    pub fn into_structs(self) -> impl Iterator<Item = StructEnv<'env>> {
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
            SignatureToken::Address => Type::Primitive(PrimitiveType::Address),
            SignatureToken::Reference(t) => {
                Type::Reference(false, Box::new(self.globalize_signature(&*t)))
            }
            SignatureToken::MutableReference(t) => {
                Type::Reference(true, Box::new(self.globalize_signature(&*t)))
            }
            SignatureToken::TypeParameter(index) => Type::TypeParameter(*index),
            SignatureToken::Vector(bt) => Type::Vector(Box::new(self.globalize_signature(&*bt))),
            SignatureToken::Struct(handle_idx) => {
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
                Type::Struct(declaring_module_env.data.id, struct_env.get_id(), vec![])
            }
            SignatureToken::StructInstantiation(handle_idx, args) => {
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
    pub fn globalize_signatures(&self, sigs: &[SignatureToken]) -> Vec<Type> {
        sigs.iter()
            .map(|s| self.globalize_signature(s))
            .collect_vec()
    }

    /// Gets a list of type actuals associated with the index in the bytecode.
    pub fn get_type_actuals(&self, idx: Option<SignatureIndex>) -> Vec<Type> {
        match idx {
            Some(idx) => {
                let actuals = &self.data.module.signature_at(idx).0;
                self.globalize_signatures(actuals)
            }
            None => vec![],
        }
    }

    /// Converts an address pool index for this module into a number representing the address.
    pub fn get_address(&self, idx: AddressPoolIndex) -> BigUint {
        let addr = &self.data.module.address_pool()[idx.0 as usize];
        BigUint::from_str_radix(&addr.to_string(), 16).unwrap()
    }

    /// Gets a byte blob based on a pool index.
    pub fn get_byte_blob(&self, idx: ByteArrayPoolIndex) -> &[u8] {
        &self.data.module.byte_array_pool()[idx.0 as usize]
    }

    /// Returns specification variables of this module.
    pub fn get_spec_vars(&'env self) -> impl Iterator<Item = (&'env SpecVarId, &'env SpecVarDecl)> {
        self.data.spec_vars.iter()
    }

    /// Gets spec var by id.
    pub fn get_spec_var(&self, id: SpecVarId) -> &SpecVarDecl {
        self.data.spec_vars.get(&id).expect("spec var id defined")
    }

    /// Returns specification functions of this module.
    pub fn get_spec_funs(&'env self) -> impl Iterator<Item = (&'env SpecFunId, &'env SpecFunDecl)> {
        self.data.spec_funs.iter()
    }

    /// Gets spec fun by id.
    pub fn get_spec_fun(&self, id: SpecFunId) -> &SpecFunDecl {
        self.data.spec_funs.get(&id).expect("spec fun id defined")
    }

    /// Gets module invariants.
    pub fn get_module_invariants(&self) -> &[Invariant] {
        &self.data.module_invariants
    }

    /// Get all spec fun overloads with the given name.
    pub fn get_spec_funs_of_name(
        &self,
        name: Symbol,
    ) -> impl Iterator<Item = (&'env SpecFunId, &'env SpecFunDecl)> {
        self.data
            .spec_funs
            .iter()
            .filter(move |(_, decl)| decl.name == name)
    }

    /// Gets the location of the given node.
    pub fn get_node_loc(&self, node_id: NodeId) -> Loc {
        self.data
            .loc_map
            .get(&node_id)
            .cloned()
            .unwrap_or_else(|| self.env.unknown_loc())
    }

    /// Gets the type of the given node.
    pub fn get_node_type(&self, node_id: NodeId) -> Type {
        self.data
            .type_map
            .get(&node_id)
            .cloned()
            .unwrap_or_else(|| Type::Error)
    }

    /// Gets the type parameter instantiation associated with the given node.
    pub fn get_node_instantiation(&self, node_id: NodeId) -> Vec<Type> {
        self.data
            .instantiation_map
            .get(&node_id)
            .cloned()
            .unwrap_or_else(|| vec![])
    }
}

// =================================================================================================
/// # Struct Environment

#[derive(Debug)]
pub struct StructData {
    /// The name of this struct.
    name: Symbol,

    /// The location of this struct.
    loc: Loc,

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

    /// Shortcut for accessing the symbol pool.
    pub fn symbol_pool(&self) -> &SymbolPool {
        self.module_env.symbol_pool()
    }

    /// Returns the location of this struct.
    pub fn get_loc(&self) -> Loc {
        self.data.loc.clone()
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

    /// Get an iterator for the fields, ordered by offset.
    pub fn get_fields(&'env self) -> impl Iterator<Item = FieldEnv<'env>> {
        self.data
            .field_data
            .values()
            .sorted_by_key(|data| data.offset)
            .map(move |data| FieldEnv {
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
        self.data.field_data.get(&id).map(|data| FieldEnv {
            struct_env: self.clone(),
            data,
        })
    }

    /// Gets a field by its offset.
    pub fn get_field_by_offset(&'env self, offset: usize) -> FieldEnv<'env> {
        for data in self.data.field_data.values() {
            if data.offset == offset {
                return FieldEnv {
                    struct_env: self.clone(),
                    data,
                };
            }
        }
        unreachable!("invalid field lookup")
    }

    /// Returns the type parameters associated with this struct.
    pub fn get_type_parameters(&self) -> Vec<TypeParameter> {
        // TODO: we currently do not know the original names of those formals, so we generate them.
        let view = StructDefinitionView::new(
            &self.module_env.data.module,
            self.module_env.data.module.struct_def_at(self.data.def_idx),
        );
        view.type_parameters()
            .iter()
            .enumerate()
            .map(|(i, k)| {
                TypeParameter(
                    self.module_env.env.symbol_pool.make(&format!("$tv{}", i)),
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

    /// The struct definition index of this field in its module.
    def_idx: StructDefinitionIndex,

    /// The offset of this field.
    offset: usize,
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
        let struct_def = self
            .struct_env
            .module_env
            .data
            .module
            .struct_def_at(self.data.def_idx);
        let field = match &struct_def.field_information {
            StructFieldInformation::Declared(fields) => &fields[self.data.offset],
            StructFieldInformation::Native => unreachable!(),
        };
        self.struct_env
            .module_env
            .globalize_signature(&field.signature.0)
    }

    /// Get field offset.
    pub fn get_offset(&self) -> usize {
        self.data.offset
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

    /// Location of this function.
    loc: Loc,

    /// The definition index of this function in its module.
    def_idx: FunctionDefinitionIndex,

    /// The handle index of this function in its module.
    handle_idx: FunctionHandleIndex,

    /// List of function argument names. Not in bytecode but obtained from AST.
    arg_names: Vec<Symbol>,

    /// List of type argument names. Not in bytecode but obtained from AST.
    type_arg_names: Vec<Symbol>,

    /// List of specification conditions. Not in bytecode but obtained from AST.
    spec: FunSpec,
}

#[derive(Debug, Clone)]
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

    /// Gets the definition index of this function.
    pub fn get_def_idx(&self) -> FunctionDefinitionIndex {
        self.data.def_idx
    }

    /// Shortcut for accessing the symbol pool.
    pub fn symbol_pool(&self) -> &SymbolPool {
        self.module_env.symbol_pool()
    }

    /// Returns the location of this function.
    pub fn get_loc(&self) -> Loc {
        self.data.loc.clone()
    }

    /// Returns the location of the bytecode at the given offset.
    pub fn get_bytecode_loc(&self, offset: u16) -> Loc {
        if let Ok(fmap) = self
            .module_env
            .data
            .source_map
            .get_function_source_map(self.data.def_idx)
        {
            if let Some(loc) = fmap.get_code_location(offset) {
                return self.module_env.env.to_loc(&loc);
            }
        }
        self.get_loc()
    }

    /// Returns true if this function is native.
    pub fn is_native(&self) -> bool {
        let view = self.definition_view();
        view.is_native()
    }

    /// Returns true if this function is public.
    pub fn is_public(&self) -> bool {
        let view = self.definition_view();
        view.is_public()
    }

    /// Returns true if this function mutates any references (i.e. has &mut parameters).
    pub fn is_mutating(&self) -> bool {
        self.get_parameters()
            .iter()
            .any(|Parameter(_, ty)| ty.is_mutable_reference())
    }

    /// Returns the type parameters associated with this function.
    pub fn get_type_parameters(&self) -> Vec<TypeParameter> {
        // TODO: currently the translation scheme isn't working with using real type
        //   parameter names, so use indices instead.
        let view = self.definition_view();
        view.type_parameters()
            .iter()
            .enumerate()
            .map(|(i, k)| {
                TypeParameter(
                    self.module_env.env.symbol_pool.make(&format!("$tv{}", i)),
                    *k,
                )
            })
            .collect_vec()
    }

    pub fn get_parameter_count(&self) -> usize {
        let view = self.definition_view();
        view.arg_tokens().count()
    }

    /// Returns the regular parameters associated with this function.
    pub fn get_parameters(&self) -> Vec<Parameter> {
        let view = self.definition_view();
        view.arg_tokens()
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
        view.return_tokens()
            .map(|tv: SignatureTokenView<CompiledModule>| {
                self.module_env.globalize_signature(tv.signature_token())
            })
            .collect_vec()
    }

    /// Returns the number of return values of this function.
    pub fn get_return_count(&self) -> usize {
        let view = self.definition_view();
        view.return_count()
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
                // The move compiler produces temporay names of the form `<foo>%#<num>`.
                // Replace occurences of `%#` so they are accepted by boogie.
                let ident = ident.replace("%#", "$$");
                return self.module_env.env.symbol_pool.make(ident.as_str());
            }
        }
        self.module_env.env.symbol_pool.make(&format!("$t{}", idx))
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
    pub fn get_specification_on_decl(&'env self) -> &'env [Condition] {
        &self.data.spec.on_decl
    }

    /// Returns specification conditions associated with this function at bytecode offset.
    pub fn get_specification_on_impl(&'env self, offset: CodeOffset) -> Option<&'env [Condition]> {
        self.data.spec.on_impl.get(&offset).map(|x| x.as_slice())
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
