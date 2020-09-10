// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides an environment -- global state -- for translation, including helper functions
//! to interpret metadata about the translation target.

#[allow(unused_imports)]
use log::{info, warn};

use std::cell::RefCell;

use codespan::{ByteIndex, ByteOffset, FileId, Files, Location, Span, SpanOutOfBoundsError};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label, Severity},
    term::{emit, termcolor::WriteColor, Config},
};
use itertools::Itertools;
use num::{BigUint, Num, ToPrimitive};

use bytecode_source_map::source_map::SourceMap;
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage, value::MoveValue,
};
use serde::{Deserialize, Serialize};
use vm::{
    access::ModuleAccess,
    file_format::{
        AddressIdentifierIndex, Constant as VMConstant, ConstantPoolIndex, FunctionDefinitionIndex,
        FunctionHandleIndex, Kind, SignatureIndex, SignatureToken, StructDefinitionIndex,
        StructFieldInformation, StructHandleIndex,
    },
    views::{
        FunctionDefinitionView, FunctionHandleView, SignatureTokenView, StructDefinitionView,
        StructHandleView,
    },
};

use crate::{
    ast::{
        GlobalInvariant, ModuleName, PropertyBag, Spec, SpecBlockInfo, SpecFunDecl, SpecVarDecl,
        Value,
    },
    symbol::{Symbol, SymbolPool},
    ty::{PrimitiveType, Type},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
};
use vm::{file_format::Bytecode, CompiledModule};

// =================================================================================================
/// # Constants

/// A name we use to represent a script as a module.
pub const SCRIPT_MODULE_NAME: &str = "<SELF>";

/// Names used in the bytecode/AST to represent the main function of a script
pub const SCRIPT_BYTECODE_FUN_NAME: &str = "<SELF>";

/// Pragma indicating whether verification should be performed for a function.
pub const VERIFY_PRAGMA: &str = "verify";

/// Pragma defining a timeout.
pub const TIMEOUT_PRAGMA: &str = "timeout";

/// Pragma defining a random seed.
pub const SEED_PRAGMA: &str = "seed";

/// Pragma indicating an estimate how long verification takes. Verification
/// is skipped if the timeout is smaller than this.
pub const VERIFY_DURATION_ESTIMATE_PRAGMA: &str = "verify_duration_estimate";

/// Pragma indicating whether implementation of function should be ignored and
/// instead treated to be like a native function.
pub const INTRINSIC_PRAGMA: &str = "intrinsic";

/// Pragma indicating whether implementation of function should be ignored and
/// instead interpreted by its pre and post conditions only.
pub const OPAQUE_PRAGMA: &str = "opaque";

/// Pragma indicating whether aborts_if specification should be considered partial.
pub const ABORTS_IF_IS_PARTIAL_PRAGMA: &str = "aborts_if_is_partial";

/// Pragma indicating whether no explicit aborts_if specification should be treated
/// like `aborts_if` false.
pub const ABORTS_IF_IS_STRICT_PRAGMA: &str = "aborts_if_is_strict";

/// Pragma indicating that requires are also enforced if the aborts condition is true.
pub const REQUIRES_IF_ABORTS: &str = "requires_if_aborts";

/// Pragma indicating that the function will run smoke tests
pub const ALWAYS_ABORTS_TEST_PRAGMA: &str = "always_aborts_test";

/// Pragma indicating that adding u64 or u128 values should not be checked
/// for overflow.
pub const ADDITION_OVERFLOW_UNCHECKED_PRAGMA: &str = "addition_overflow_unchecked";

/// Pragma indicating that aborts from this function shall be ignored.
pub const ASSUME_NO_ABORT_FROM_HERE_PRAGMA: &str = "assume_no_abort_from_here";

/// Internal property attached to conditions if they are injected via an apply or a module
/// invariant.
pub const CONDITION_INJECTED_PROP: &str = "$injected";

/// Property which can be attached to conditions to make them exported into the VC context
/// even if they are injected.
pub const CONDITION_EXPORT_PROP: &str = "export";

/// Property which can be attached to a module invariant to make it global.
pub const CONDITION_GLOBAL_PROP: &str = "global";

/// Property which can be attached to a global invariant to mark it as not to be used as
/// an assumption in other verification steps. This can be used for invariants which are
/// nonoperational constraints on system behavior, i.e. the systems "works" whether the
/// invariant holds or not. Invariant marked as such are not assumed when
/// memory is accessed, but only in the pre-state of a memory update.
pub const CONDITION_ISOLATED: &str = "isolated";

/// Abstract property which can be used together with an opaque specification. An abstract
/// property is not verified against the implementation, but will be used for the
/// function's behavior in the application context. This allows to "override" the specification
/// with a more abstract version. In general we would need to prover the abstraction is
/// subsumed by the implementation, but this is currently not done.
pub const CONDITION_ABSTRACT_PROP: &str = "abstract";

/// Opposite to the abstract property.
pub const CONDITION_CONCRETE_PROP: &str = "concrete";

/// Property which indicates that an aborts_if should be assumed.
/// For callers of a function with such an aborts_if, the negation of the condition becomes
/// an assumption.
pub const CONDITION_ABORT_ASSUME_PROP: &str = "assume";

/// Property which indicates that an aborts_if should be asserted.
/// For callers of a function with such an aborts_if, the negation of the condition becomes
/// an assertion.
pub const CONDITION_ABORT_ASSERT_PROP: &str = "assert";

/// Pragma which indicates that the functions aborts and ensure conditions shall be exported
/// to the verification context even if the implementation of the function is inlined.
pub const EXPORT_ENSURES_PRAGMA: &str = "export_ensures";

/// A property which can be attached to any condition to exclude it from verification. The
/// condition will still be type checked.
pub const CONDITION_DEACTIVATED: &str = "deactivated";

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

    /// Creates a location which encloses all the locations in the provided slice,
    /// which must not be empty. All locations are expected to be in the same file.
    pub fn enclosing(locs: &[&Loc]) -> Loc {
        assert!(!locs.is_empty());
        let loc = locs[0];
        let mut start = loc.span.start();
        let mut end = loc.span.end();
        for l in locs.iter().skip(1) {
            if l.file_id() == loc.file_id() {
                start = std::cmp::min(start, l.span().start());
                end = std::cmp::max(end, l.span().end());
            }
        }
        Loc::new(loc.file_id(), Span::new(start, end))
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

/// Identifier for a named constant, relative to module.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct NamedConstantId(Symbol);

/// Identifier for a structure/resource, relative to module.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct StructId(Symbol);

/// Identifier for a field of a structure, relative to struct.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct FieldId(Symbol);

/// Identifier for a Move function, relative to module.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct FunId(Symbol);

/// Identifier for a schema.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SchemaId(Symbol);

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

/// A global id. Instances of this type represent unique identifiers relative to `GlobalEnv`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct GlobalId(usize);

// Some identifier qualified by a module.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct QualifiedId<Id> {
    pub module_id: ModuleId,
    pub id: Id,
}

impl NamedConstantId {
    pub fn new(sym: Symbol) -> Self {
        Self(sym)
    }

    pub fn symbol(self) -> Symbol {
        self.0
    }
}

impl FunId {
    pub fn new(sym: Symbol) -> Self {
        Self(sym)
    }

    pub fn symbol(self) -> Symbol {
        self.0
    }
}

impl SchemaId {
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

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl SpecVarId {
    pub fn new(idx: usize) -> Self {
        Self(idx as RawIndex)
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl NodeId {
    pub fn new(idx: usize) -> Self {
        Self(idx as RawIndex)
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
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

impl GlobalId {
    pub fn new(idx: usize) -> Self {
        Self(idx)
    }

    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl ModuleId {
    pub fn qualified<Id>(self, id: Id) -> QualifiedId<Id> {
        QualifiedId {
            module_id: self,
            id,
        }
    }
}

// =================================================================================================
/// # Verification Scope

/// Defines what functions to verify.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VerificationScope {
    /// Verify only public functions.
    Public,
    /// Verify all functions.
    All,
    /// Verify no functions
    None,
}

impl Default for VerificationScope {
    fn default() -> Self {
        Self::Public
    }
}

// =================================================================================================
/// # Global Environment

/// Global environment for a set of modules.
#[derive(Debug)]
pub struct GlobalEnv {
    /// A Files database for the codespan crate which supports diagnostics.
    source_files: Files<String>,
    /// A map of FileId in the Files database to information about documentation comments in a file.
    /// The comments are represented as map from ByteIndex into string, where the index is the
    /// start position of the associated language item in the source.
    doc_comments: BTreeMap<FileId, BTreeMap<ByteIndex, String>>,
    /// A map of locations to information about verification conditions at the location.
    condition_infos: RefCell<BTreeMap<(Loc, ConditionTag), ConditionInfo>>,
    /// A mapping from file names to associated FileId. Though this information is
    /// already in `source_files`, we can't get it out of there so need to book keep here.
    file_name_map: BTreeMap<String, FileId>,
    /// Bijective mapping between FileId and a plain int. FileId's are themselves wrappers around
    /// ints, but the inner representation is opaque and cannot be accessed. This is used so we
    /// can emit FileId's to generated code and read them back.
    file_id_to_idx: BTreeMap<FileId, u16>,
    file_idx_to_id: BTreeMap<u16, FileId>,
    /// A set indicating whether a file id is a target or a dependency.
    file_id_is_dep: BTreeSet<FileId>,
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
    /// A counter for issuing global ids.
    global_id_counter: RefCell<usize>,
    /// A map of global invariants.
    global_invariants: BTreeMap<GlobalId, GlobalInvariant>,
    /// A map from spec vars to global invariants which refer to them.
    global_invariants_for_spec_var: BTreeMap<QualifiedId<SpecVarId>, BTreeSet<GlobalId>>,
    /// A map from global memories to global invariants which refer to them.
    global_invariants_for_memory: BTreeMap<QualifiedId<StructId>, BTreeSet<GlobalId>>,
    /// A set containing spec functions which are called/used in specs.
    pub used_spec_funs: BTreeSet<QualifiedId<SpecFunId>>,
}

/// Information about a verification condition stored in the environment.
#[derive(Debug, Clone, Default)]
pub struct ConditionInfo {
    /// The message to print when the condition fails.
    pub message: String,
    /// Whether execution traces shall be printed if this condition fails.
    pub omit_trace: bool,
    /// Whether passing this condition is actually a failure.
    pub negative_cond: bool,
}

/// A tag used to be associated with a condition info. Condition infos are
/// identified in the environment by a pair of Loc and this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConditionTag {
    Requires,
    Ensures,
    NegativeTest,
}

impl ConditionInfo {
    pub fn for_message<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
            omit_trace: false,
            negative_cond: false,
        }
    }
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
            doc_comments: Default::default(),
            condition_infos: Default::default(),
            unknown_loc,
            unknown_move_ir_loc,
            internal_loc,
            file_name_map,
            file_id_to_idx,
            file_idx_to_id,
            file_id_is_dep: BTreeSet::new(),
            diags: RefCell::new(vec![]),
            symbol_pool: SymbolPool::new(),
            module_data: vec![],
            global_id_counter: RefCell::new(0),
            global_invariants: Default::default(),
            global_invariants_for_memory: Default::default(),
            global_invariants_for_spec_var: Default::default(),
            used_spec_funs: BTreeSet::new(),
        }
    }

    /// Create a new global id unique to this environment.
    pub fn new_global_id(&self) -> GlobalId {
        let mut counter = self.global_id_counter.borrow_mut();
        let id = GlobalId::new(*counter);
        *counter += 1;
        id
    }

    /// Returns a reference to the symbol pool owned by this environment.
    pub fn symbol_pool(&self) -> &SymbolPool {
        &self.symbol_pool
    }

    /// Adds a source to this environment, returning a FileId for it.
    pub fn add_source(&mut self, file_name: &str, source: &str, is_dep: bool) -> FileId {
        let file_id = self.source_files.add(file_name, source.to_string());
        self.file_name_map.insert(file_name.to_string(), file_id);
        let file_idx = self.file_id_to_idx.len() as u16;
        self.file_id_to_idx.insert(file_id, file_idx);
        self.file_idx_to_id.insert(file_idx, file_id);
        if is_dep {
            self.file_id_is_dep.insert(file_id);
        }
        file_id
    }

    /// Adds documentation for a file.
    pub fn add_documentation(&mut self, file_id: FileId, docs: BTreeMap<ByteIndex, String>) {
        self.doc_comments.insert(file_id, docs);
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

    /// Adds a warning to this environment.
    pub fn warn(&self, loc: &Loc, msg: &str) {
        let diag = Diagnostic::new_warning(msg, Label::new(loc.file_id, loc.span, ""));
        self.add_diag(diag);
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
        Loc {
            file_id: self.get_file_id(loc.file()).expect("file name undefined"),
            span: loc.span(),
        }
    }

    /// Returns the file id for a file name, if defined.
    pub fn get_file_id(&self, fname: &str) -> Option<FileId> {
        self.file_name_map.get(fname).cloned()
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

    /// Return the source text for the given location.
    pub fn get_source(&self, loc: &Loc) -> Result<&str, SpanOutOfBoundsError> {
        self.source_files.source_slice(loc.file_id, loc.span)
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

    /// Returns true if diagnostics have warning severity or worse.
    pub fn has_warnings(&self) -> bool {
        self.diags
            .borrow()
            .iter()
            .any(|d| d.severity >= Severity::Warning)
    }

    /// Writes accumulated errors to writer.
    pub fn report_errors<W: WriteColor>(&self, writer: &mut W) {
        for diag in self
            .diags
            .borrow()
            .iter()
            .filter(|d| d.severity >= Severity::Error)
        {
            emit(writer, &Config::default(), &self.source_files, diag).expect("emit must not fail");
        }
    }

    /// Writes accumulated diagnostics with warning severity or worse to writer.
    pub fn report_warnings<W: WriteColor>(&self, writer: &mut W) {
        for diag in self
            .diags
            .borrow()
            .iter()
            .filter(|d| d.severity >= Severity::Warning)
        {
            emit(writer, &Config::default(), &self.source_files, diag).expect("emit must not fail");
        }
    }

    /// Adds a global invariant to this environment.
    pub fn add_global_invariant(&mut self, inv: GlobalInvariant) {
        let id = inv.id;
        for spec_var in &inv.spec_var_usage {
            self.global_invariants_for_spec_var
                .entry(*spec_var)
                .or_insert_with(BTreeSet::new)
                .insert(id);
        }
        for memory in &inv.mem_usage {
            self.global_invariants_for_memory
                .entry(*memory)
                .or_insert_with(BTreeSet::new)
                .insert(id);
        }
        self.global_invariants.insert(id, inv);
    }

    /// Get global invariant by id.
    pub fn get_global_invariant(&self, id: GlobalId) -> Option<&GlobalInvariant> {
        self.global_invariants.get(&id)
    }

    /// Return the global invariants which refer to the given memory.
    pub fn get_global_invariants_for_memory(&self, memory: QualifiedId<StructId>) -> Vec<GlobalId> {
        self.global_invariants_for_memory
            .get(&memory)
            .map(|ids| ids.iter().cloned().collect_vec())
            .unwrap_or_else(Default::default)
    }

    /// Returns true if a spec fun is used in specs.
    pub fn is_spec_fun_used(&self, module_id: ModuleId, spec_fun_id: SpecFunId) -> bool {
        self.used_spec_funs
            .contains(&module_id.qualified(spec_fun_id))
    }

    /// Adds a new module to the environment. StructData and FunctionData need to be provided
    /// in definition index order. See `create_function_data` and `create_struct_data` for how
    /// to create them.
    #[allow(clippy::too_many_arguments)]
    pub fn add(
        &mut self,
        loc: Loc,
        module: CompiledModule,
        source_map: SourceMap<MoveIrLoc>,
        named_constants: BTreeMap<NamedConstantId, NamedConstantData>,
        struct_data: BTreeMap<StructId, StructData>,
        function_data: BTreeMap<FunId, FunctionData>,
        spec_vars: Vec<SpecVarDecl>,
        spec_funs: Vec<SpecFunDecl>,
        module_spec: Spec,
        loc_map: BTreeMap<NodeId, Loc>,
        type_map: BTreeMap<NodeId, Type>,
        instantiation_map: BTreeMap<NodeId, Vec<Type>>,
        spec_block_infos: Vec<SpecBlockInfo>,
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

        let next_free_node_id = loc_map
            .keys()
            .chain(type_map.keys())
            .map(|i| i.as_usize())
            .max()
            .unwrap_or(0)
            + 1;

        self.module_data.push(ModuleData {
            name,
            id: ModuleId(idx as RawIndex),
            module,
            named_constants,
            struct_data,
            struct_idx_to_id,
            function_data,
            function_idx_to_id,
            spec_vars,
            spec_funs,
            module_spec,
            source_map,
            loc,
            next_free_node_id: RefCell::new(next_free_node_id),
            loc_map: RefCell::new(loc_map),
            type_map: RefCell::new(type_map),
            instantiation_map,
            spec_block_infos,
        });
    }

    /// Creates data for a named constant.
    pub fn create_named_constant_data(
        &self,
        name: Symbol,
        loc: Loc,
        typ: Type,
        value: Value,
    ) -> NamedConstantData {
        NamedConstantData {
            name,
            loc,
            typ,
            value,
        }
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
        spec: Spec,
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
        spec: Spec,
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
        StructData {
            name,
            loc,
            def_idx,
            handle_idx,
            field_data,
            spec,
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

    /// Gets a StructEnv in this module by its `StructTag`
    pub fn find_struct_by_tag(
        &self,
        tag: &language_storage::StructTag,
    ) -> Option<QualifiedId<StructId>> {
        self.find_module(&self.to_module_name(&tag.module_id()))
            .map(|menv| {
                menv.find_struct_by_identifier(tag.name.clone())
                    .map(|sid| menv.get_id().qualified(sid))
            })
            .flatten()
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

    /// Return the `FunctionEnv` for `fun`
    pub fn get_function(&self, fun: QualifiedId<FunId>) -> FunctionEnv<'_> {
        self.get_module(fun.module_id).into_function(fun.id)
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
    pub fn get_all_structs_with_conditions(&self) -> Vec<Type> {
        let mut res = vec![];
        for module_env in self.get_modules() {
            for struct_env in module_env.get_structs() {
                if struct_env.has_conditions() {
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

    /// Get documentation associated with an item at Loc.
    pub fn get_doc(&self, loc: &Loc) -> &str {
        self.doc_comments
            .get(&loc.file_id)
            .and_then(|comments| comments.get(&loc.span.start()).map(|s| s.as_str()))
            .unwrap_or("")
    }

    /// Get verification condition info associated with location.
    pub fn get_condition_info(&self, loc: &Loc, tag: ConditionTag) -> Option<ConditionInfo> {
        self.condition_infos
            .borrow()
            .get(&(loc.clone(), tag))
            .cloned()
    }

    /// Set verification condition info.
    pub fn set_condition_info(&self, loc: Loc, tag: ConditionTag, info: ConditionInfo) {
        self.condition_infos.borrow_mut().insert((loc, tag), info);
    }

    /// Execute function on each condition info.
    pub fn with_condition_infos<F>(&self, mut f: F)
    where
        F: FnMut(&(Loc, ConditionTag), &ConditionInfo),
    {
        self.condition_infos
            .borrow()
            .iter()
            .for_each(|(l, i)| f(l, i))
    }

    /// Returns true if the boolean property is true.
    pub fn is_property_true(&self, properties: &PropertyBag, name: &str) -> Option<bool> {
        let sym = &self.symbol_pool().make(name);
        if let Some(Value::Bool(b)) = properties.get(sym) {
            return Some(*b);
        }
        None
    }

    /// Returns the value of a number property.
    pub fn get_num_property(&self, properties: &PropertyBag, name: &str) -> Option<usize> {
        let sym = &self.symbol_pool().make(name);
        if let Some(Value::Number(n)) = properties.get(sym) {
            return n.to_usize();
        }
        None
    }

    /// Attempt to compute a struct tag for (`mid`, `sid`, `ts`). Returns `Some` if all types in
    /// `ts` are closed, `None` otherwise
    pub fn get_struct_tag(
        &self,
        mid: ModuleId,
        sid: StructId,
        ts: &[Type],
    ) -> Option<language_storage::StructTag> {
        if ts.iter().any(|t| t.is_open()) {
            None
        } else {
            let menv = self.get_module(mid);
            Some(language_storage::StructTag {
                address: *menv.self_address(),
                module: menv.get_identifier(),
                name: menv.get_struct(sid).get_identifier(),
                type_params: ts
                    .iter()
                    .map(|t| t.clone().into_type_tag(self).unwrap())
                    .collect(),
            })
        }
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

    /// Named constant data
    pub named_constants: BTreeMap<NamedConstantId, NamedConstantData>,

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

    /// Module level specification.
    pub module_spec: Spec,

    /// Module source location information.
    pub source_map: SourceMap<MoveIrLoc>,

    /// The location of this module.
    pub loc: Loc,

    /// A map from node id to associated location.
    pub loc_map: RefCell<BTreeMap<NodeId, Loc>>,

    /// A map from node id to associated type.
    pub type_map: RefCell<BTreeMap<NodeId, Type>>,

    /// A counter for allocating node ids.
    pub next_free_node_id: RefCell<usize>,

    /// A map from node id to associated instantiation of type parameters.
    pub instantiation_map: BTreeMap<NodeId, Vec<Type>>,

    /// A list of spec block infos, for documentation generation.
    pub spec_block_infos: Vec<SpecBlockInfo>,
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

    /// Returns the VM identifier for this module
    pub fn get_identifier(&'env self) -> Identifier {
        self.data.module.name().to_owned()
    }

    /// Returns true if this is a module representing a script.
    pub fn is_script_module(&self) -> bool {
        self.symbol_pool().string(self.data.name.name()).as_str() == SCRIPT_MODULE_NAME
    }

    /// Returns true of this module is from a dependency, i.e. not the target of verification.
    pub fn is_dependency(&self) -> bool {
        let file_id = self.data.loc.file_id;
        self.env.file_id_is_dep.contains(&file_id)
    }

    /// Returns the path to source file of this module.
    pub fn get_source_path(&self) -> &OsStr {
        let file_id = self.data.loc.file_id;
        self.env.source_files.name(file_id)
    }

    /// Return the set of ModuleId's that this module depends on (including itself)
    pub fn get_dependencies(&self) -> Vec<language_storage::ModuleId> {
        let compiled_module = &self.data.module;
        compiled_module
            .module_handles()
            .iter()
            .map(|h| compiled_module.module_id_for_handle(h))
            .collect()
    }

    /// Returns documentation associated with this module.
    pub fn get_doc(&self) -> &str {
        self.env.get_doc(&self.data.loc)
    }

    /// Returns spec block documentation infos.
    pub fn get_spec_block_infos(&self) -> &[SpecBlockInfo] {
        &self.data.spec_block_infos
    }

    /// Shortcut for accessing the symbol pool.
    pub fn symbol_pool(&self) -> &SymbolPool {
        &self.env.symbol_pool
    }

    /// Gets the underlying bytecode module.
    pub fn get_verified_module(&'env self) -> &'env CompiledModule {
        &self.data.module
    }

    /// Gets a `NamedConstantEnv` in this module by name
    pub fn find_named_constant(&'env self, name: Symbol) -> Option<NamedConstantEnv<'env>> {
        let id = NamedConstantId(name);
        self.data
            .named_constants
            .get(&id)
            .map(|data| NamedConstantEnv {
                module_env: self.clone(),
                data,
            })
    }

    /// Gets a `NamedConstantEnv` in this module by the constant's id
    pub fn get_named_constant(&'env self, id: NamedConstantId) -> NamedConstantEnv<'env> {
        self.clone().into_named_constant(id)
    }

    /// Gets a `NamedConstantEnv` by id
    pub fn into_named_constant(self, id: NamedConstantId) -> NamedConstantEnv<'env> {
        let data = self
            .data
            .named_constants
            .get(&id)
            .expect("NamedConstantId undefined");
        NamedConstantEnv {
            module_env: self,
            data,
        }
    }

    /// Gets the number of named constants in this module.
    pub fn get_named_constant_count(&self) -> usize {
        self.data.named_constants.len()
    }

    /// Returns iterator over `NamedConstantEnv`s in this module.
    pub fn get_named_constants(&'env self) -> impl Iterator<Item = NamedConstantEnv<'env>> {
        self.clone().into_named_constants()
    }

    /// Returns an iterator over `NamedConstantEnv`s in this module.
    pub fn into_named_constants(self) -> impl Iterator<Item = NamedConstantEnv<'env>> {
        self.data
            .named_constants
            .iter()
            .map(move |(_, data)| NamedConstantEnv {
                module_env: self.clone(),
                data,
            })
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

    /// Gets a StructEnv in this module by identifier
    pub fn find_struct_by_identifier(&self, identifier: Identifier) -> Option<StructId> {
        for data in self.data.struct_data.values() {
            let senv = StructEnv {
                module_env: self.clone(),
                data,
            };
            if senv.get_identifier() == identifier {
                return Some(senv.get_id());
            }
        }
        None
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
            SignatureToken::Signer => Type::Primitive(PrimitiveType::Signer),
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

    /// Retrieve a constant from the pool
    pub fn get_constant(&self, idx: ConstantPoolIndex) -> &VMConstant {
        &self.data.module.constant_pool()[idx.0 as usize]
    }

    /// Converts a constant to the specified type. The type must correspond to the expected
    /// cannonical representation as defined in `move_core_types::values`
    pub fn get_constant_value(&self, constant: &VMConstant) -> MoveValue {
        VMConstant::deserialize_constant(constant).unwrap()
    }

    /// Return the `AccountAdress` of this module
    pub fn self_address(&self) -> &AccountAddress {
        self.data.module.address()
    }

    /// Retrieve an address identifier from the pool
    pub fn get_address_identifier(&self, idx: AddressIdentifierIndex) -> BigUint {
        let addr = &self.data.module.address_identifiers()[idx.0 as usize];
        Self::addr_to_big_uint(addr)
    }

    /// Converts an address identifier to a number representing the address.
    pub fn addr_to_big_uint(addr: &AccountAddress) -> BigUint {
        BigUint::from_str_radix(&addr.to_string(), 16).unwrap()
    }

    /// Returns specification variables of this module.
    pub fn get_spec_vars(&'env self) -> impl Iterator<Item = (&'env SpecVarId, &'env SpecVarDecl)> {
        self.data.spec_vars.iter()
    }

    /// Gets spec var by id.
    pub fn get_spec_var(&self, id: SpecVarId) -> &SpecVarDecl {
        self.data.spec_vars.get(&id).expect("spec var id defined")
    }

    /// Find spec var by name.
    pub fn find_spec_var(&self, name: Symbol) -> Option<&SpecVarDecl> {
        self.data
            .spec_vars
            .iter()
            .find(|(_, svar)| svar.name == name)
            .map(|(_, svar)| svar)
    }

    /// Returns specification functions of this module.
    pub fn get_spec_funs(&'env self) -> impl Iterator<Item = (&'env SpecFunId, &'env SpecFunDecl)> {
        self.data.spec_funs.iter()
    }

    /// Gets spec fun by id.
    pub fn get_spec_fun(&self, id: SpecFunId) -> &SpecFunDecl {
        self.data.spec_funs.get(&id).expect("spec fun id defined")
    }

    /// Gets module specification.
    pub fn get_spec(&self) -> &Spec {
        &self.data.module_spec
    }

    /// Returns whether a spec fun is ever called or not.
    pub fn spec_fun_is_used(&self, spec_fun_id: SpecFunId) -> bool {
        self.env
            .used_spec_funs
            .contains(&self.get_id().qualified(spec_fun_id))
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
            .borrow()
            .get(&node_id)
            .cloned()
            .unwrap_or_else(|| self.env.unknown_loc())
    }

    /// Gets the type of the given node.
    pub fn get_node_type(&self, node_id: NodeId) -> Type {
        self.data
            .type_map
            .borrow()
            .get(&node_id)
            .cloned()
            .unwrap_or_else(|| Type::Error)
    }

    /// Allocates a new node id.
    pub fn new_node_id(&self) -> NodeId {
        let id = NodeId::new(*self.data.next_free_node_id.borrow());
        *self.data.next_free_node_id.borrow_mut() += 1;
        id
    }

    /// Allocates a new node id and assigns location and type to it.
    pub fn new_node(&self, loc: Loc, ty: Type) -> NodeId {
        let id = self.new_node_id();
        self.data.loc_map.borrow_mut().insert(id, loc);
        self.data.type_map.borrow_mut().insert(id, ty);
        id
    }

    /// Gets the type parameter instantiation associated with the given node.
    pub fn get_node_instantiation(&self, node_id: NodeId) -> Vec<Type> {
        self.data
            .instantiation_map
            .get(&node_id)
            .cloned()
            .unwrap_or_else(Vec::new)
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

    // Associated specification.
    spec: Spec,
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

    /// Returns the VM identifier for this struct
    pub fn get_identifier(&self) -> Identifier {
        let handle = self
            .module_env
            .data
            .module
            .struct_handle_at(self.data.handle_idx);
        self.module_env
            .data
            .module
            .identifier_at(handle.name)
            .to_owned()
    }

    /// Shortcut for accessing the symbol pool.
    pub fn symbol_pool(&self) -> &SymbolPool {
        self.module_env.symbol_pool()
    }

    /// Returns the location of this struct.
    pub fn get_loc(&self) -> Loc {
        self.data.loc.clone()
    }

    /// Get documentation associated with this struct.
    pub fn get_doc(&self) -> &str {
        self.module_env.env.get_doc(&self.data.loc)
    }

    /// Returns properties from pragmas.
    pub fn get_properties(&self) -> &PropertyBag {
        &self.data.spec.properties
    }

    /// Gets the id associated with this struct.
    pub fn get_id(&self) -> StructId {
        StructId(self.data.name)
    }

    /// Gets the qualified id of this struct.
    pub fn get_qualified_id(&self) -> QualifiedId<StructId> {
        self.module_env.get_id().qualified(self.get_id())
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

    /// Return the number of fields in the struct.
    pub fn get_field_count(&self) -> usize {
        self.data.field_data.len()
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
                    TypeConstraint::from(*k),
                )
            })
            .collect_vec()
    }

    /// Returns the type parameters associated with this struct, with actual names.
    pub fn get_named_type_parameters(&self) -> Vec<TypeParameter> {
        let view = StructDefinitionView::new(
            &self.module_env.data.module,
            self.module_env.data.module.struct_def_at(self.data.def_idx),
        );
        view.type_parameters()
            .iter()
            .enumerate()
            .map(|(i, k)| {
                let name = self
                    .module_env
                    .data
                    .source_map
                    .get_struct_source_map(self.data.def_idx)
                    .ok()
                    .and_then(|smap| smap.type_parameters.get(i))
                    .map(|(s, _)| s.clone())
                    .unwrap_or_else(|| format!("unknown#{}", i));
                TypeParameter(
                    self.module_env.env.symbol_pool.make(&name),
                    TypeConstraint::from(*k),
                )
            })
            .collect_vec()
    }

    /// Returns true if this struct has specifcation conditions.
    pub fn has_conditions(&self) -> bool {
        !self.data.spec.conditions.is_empty()
    }

    /// Returns the data invariants associated with this struct.
    pub fn get_spec(&'env self) -> &'env Spec {
        &self.data.spec
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

    /// Get documentation associated with this field.
    pub fn get_doc(&self) -> &str {
        if let Ok(smap) = self
            .struct_env
            .module_env
            .data
            .source_map
            .get_struct_source_map(self.data.def_idx)
        {
            let loc = self
                .struct_env
                .module_env
                .env
                .to_loc(&smap.fields[self.data.offset]);
            self.struct_env.module_env.env.get_doc(&loc)
        } else {
            ""
        }
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
/// # Named Constant Environment

#[derive(Debug)]
pub struct NamedConstantData {
    /// The name of this constant
    name: Symbol,

    /// The location of this constant
    loc: Loc,

    /// The type of this constant
    typ: Type,

    /// The value of this constant
    value: Value,
}

#[derive(Debug)]
pub struct NamedConstantEnv<'env> {
    /// Reference to enclosing module.
    pub module_env: ModuleEnv<'env>,

    data: &'env NamedConstantData,
}

impl<'env> NamedConstantEnv<'env> {
    /// Returns the name of this constant
    pub fn get_name(&self) -> Symbol {
        self.data.name
    }

    /// Returns the id of this constant
    pub fn get_id(&self) -> NamedConstantId {
        NamedConstantId(self.data.name)
    }

    /// Returns documentation associated with this constant
    pub fn get_doc(&self) -> &str {
        self.module_env.env.get_doc(&self.data.loc)
    }

    /// Returns the location of this constant
    pub fn get_loc(&self) -> Loc {
        self.data.loc.clone()
    }

    /// Returns the type of the constant
    pub fn get_type(&self) -> Type {
        self.data.typ.clone()
    }

    /// Returns the value of this constant
    pub fn get_value(&self) -> Value {
        self.data.value.clone()
    }
}

// =================================================================================================
/// # Function Environment

/// Represents a type parameter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeParameter(pub Symbol, pub TypeConstraint);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeConstraint {
    None,
    Copyable,
    Resource,
}

impl TypeConstraint {
    fn from(k: Kind) -> Self {
        match k {
            Kind::All => TypeConstraint::None,
            Kind::Copyable => TypeConstraint::Copyable,
            Kind::Resource => TypeConstraint::Resource,
        }
    }
}

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

    /// Specification associated with this function.
    spec: Spec,
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

    /// Returns the VM identifier for this function
    pub fn get_identifier(&'env self) -> Identifier {
        let m = &self.module_env.data.module;
        m.identifier_at(m.function_handle_at(self.data.handle_idx).name)
            .to_owned()
    }

    /// Gets the id of this function.
    pub fn get_id(&self) -> FunId {
        FunId(self.data.name)
    }

    /// Gets the qualified id of this function.
    pub fn get_qualified_id(&self) -> QualifiedId<FunId> {
        self.module_env.get_id().qualified(self.get_id())
    }

    /// Get documentation associated with this function.
    pub fn get_doc(&self) -> &str {
        self.module_env.env.get_doc(&self.data.loc)
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

    /// Returns the bytecode associated with this function.
    pub fn get_bytecode(&self) -> &[Bytecode] {
        let function_definition = self
            .module_env
            .data
            .module
            .function_def_at(self.get_def_idx());
        let function_definition_view =
            FunctionDefinitionView::new(&self.module_env.data.module, function_definition);
        match function_definition_view.code() {
            Some(code) => &code.code,
            None => &[],
        }
    }

    /// Returns the value of a boolean pragma for this function. This first looks up a
    /// pragma in this function, then the enclosing module, and finally uses the provided default.
    /// value
    pub fn is_pragma_true(&self, name: &str, default: impl FnOnce() -> bool) -> bool {
        let env = self.module_env.env;
        if let Some(b) = env.is_property_true(&self.get_spec().properties, name) {
            return b;
        }
        if let Some(b) = env.is_property_true(&self.module_env.get_spec().properties, name) {
            return b;
        }
        default()
    }

    /// Returns whether the value of a numeric pragma is explicitly set for this function.
    pub fn is_num_pragma_set(&self, name: &str) -> bool {
        let env = self.module_env.env;
        env.get_num_property(&self.get_spec().properties, name)
            .is_some()
            || env
                .get_num_property(&self.module_env.get_spec().properties, name)
                .is_some()
    }

    /// Returns the value of a numeric pragma for this function. This first looks up a
    /// pragma in this function, then the enclosing module, and finally uses the provided default.
    /// value
    pub fn get_num_pragma(&self, name: &str, default: impl FnOnce() -> usize) -> usize {
        let env = self.module_env.env;
        if let Some(n) = env.get_num_property(&self.get_spec().properties, name) {
            return n;
        }
        if let Some(n) = env.get_num_property(&self.module_env.get_spec().properties, name) {
            return n;
        }
        default()
    }

    /// Returns true if this function is native. The function is also marked as native
    /// if it has the pragma intrinsic set to true.
    pub fn is_native(&self) -> bool {
        let view = self.definition_view();
        view.is_native() || self.is_pragma_true(INTRINSIC_PRAGMA, || false)
    }

    /// Returns true if this function is opaque.
    pub fn is_opaque(&self) -> bool {
        self.is_pragma_true(OPAQUE_PRAGMA, || false)
    }

    /// Returns true if this function is public.
    pub fn is_public(&self) -> bool {
        let view = self.definition_view();
        view.is_public()
            // The main function of a script is implicitly public
            || self.module_env.is_script_module()
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
                    TypeConstraint::from(*k),
                )
            })
            .collect_vec()
    }

    /// Returns the type parameters with the real names.
    pub fn get_named_type_parameters(&self) -> Vec<TypeParameter> {
        let view = self.definition_view();
        view.type_parameters()
            .iter()
            .enumerate()
            .map(|(i, k)| {
                let name = self
                    .module_env
                    .data
                    .source_map
                    .get_function_source_map(self.data.def_idx)
                    .ok()
                    .and_then(|fmap| fmap.type_parameters.get(i))
                    .map(|(s, _)| s.clone())
                    .unwrap_or_else(|| format!("unknown#{}", i));
                TypeParameter(
                    self.module_env.env.symbol_pool.make(&name),
                    TypeConstraint::from(*k),
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
            if let Some((ident, _)) = fmap.get_parameter_or_local_name(idx as u64) {
                // The Move compiler produces temporary names of the form `<foo>%#<num>`,
                // where <num> seems to be generated non-deterministically.
                // Substitute this by a deterministic name which the backend accepts.
                let clean_ident = if ident.contains("%#") {
                    format!("tmp#${}", idx)
                } else {
                    ident
                };
                return self.module_env.env.symbol_pool.make(clean_ident.as_str());
            }
        }
        self.module_env.env.symbol_pool.make(&format!("$t{}", idx))
    }

    /// Returns true if the index is for a temporary, not user declared local.
    pub fn is_temporary(&self, idx: usize) -> bool {
        if idx >= self.get_local_count() {
            return true;
        }
        let name = self.get_local_name(idx);
        self.symbol_pool().string(name).contains("tmp#$")
    }

    /// Gets the number of proper locals of this function. Those are locals which are declared
    /// by the user and also have a user assigned name which can be discovered via `get_local_name`.
    /// Note we may have more anonymous locals generated e.g by the 'stackless' transformation.
    pub fn get_local_count(&self) -> usize {
        let view = self.definition_view();
        match view.locals_signature() {
            Some(locals_view) => locals_view.len(),
            None => view.parameters().len(),
        }
    }

    /// Gets the type of the local at index. This must use an index in the range as determined by
    /// `get_local_count`.
    pub fn get_local_type(&self, idx: usize) -> Type {
        let view = self.definition_view();

        let parameters = view.parameters();

        if idx < parameters.len() {
            self.module_env.globalize_signature(&parameters.0[idx])
        } else {
            self.module_env.globalize_signature(
                view.locals_signature()
                    .unwrap()
                    .token_at(idx as u8)
                    .signature_token(),
            )
        }
    }

    /// Returns associated specification.
    pub fn get_spec(&'env self) -> &'env Spec {
        &self.data.spec
    }

    /// Returns the acquired global resource types.
    pub fn get_acquires_global_resources(&'env self) -> Vec<StructId> {
        let function_definition = self
            .module_env
            .data
            .module
            .function_def_at(self.get_def_idx());
        function_definition
            .acquires_global_resources
            .iter()
            .map(|x| self.module_env.get_struct_id(*x))
            .collect()
    }

    /// Determine whether the function is target of verification.
    pub fn should_verify(&self, default_scope: VerificationScope) -> bool {
        if self.module_env.is_dependency() {
            // Never generate verify method for functions from dependencies.
            return false;
        }
        // We look up the `verify` pragma property first in this function, then in
        // the module, and finally fall back to the value specified by default_scope.
        let default = || match default_scope {
            VerificationScope::Public => self.is_public(),
            VerificationScope::All => true,
            VerificationScope::None => false,
        };
        self.is_pragma_true(VERIFY_PRAGMA, default)
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
