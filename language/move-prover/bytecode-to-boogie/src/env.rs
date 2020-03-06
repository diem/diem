// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides an environment -- global state -- for translation, including helper functions
//! to interpret metadata about the translation target.

use std::cell::RefCell;

use itertools::Itertools;
use num::{BigInt, Num};

use bytecode_source_map::source_map::ModuleSourceMap;
use bytecode_verifier::VerifiedModule;
use libra_types::language_storage::ModuleId;
use move_core_types::identifier::{IdentStr, Identifier};
use move_ir_types::{
    ast::{QualifiedStructIdent, Type, TypeVar_},
    location::{Loc, Spanned},
    spec_language_ast::{Condition, Invariant, SyntheticDefinition},
};
use vm::{
    access::ModuleAccess,
    file_format::{
        AddressPoolIndex, FieldDefinitionIndex, FunctionDefinitionIndex, FunctionHandleIndex, Kind,
        LocalsSignatureIndex, SignatureToken, StructDefinitionIndex, StructFieldInformation,
        StructHandleIndex, TypeParameterIndex,
    },
    views::{
        FieldDefinitionView, FunctionDefinitionView, FunctionHandleView, SignatureTokenView,
        StructDefinitionView, StructHandleView, ViewInternals,
    },
};

use crate::cli::Options;
use codespan::{ColumnIndex, Files, LineIndex};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label, Severity},
    term::{
        emit,
        termcolor::{ColorChoice, StandardStream},
        Config,
    },
};
use std::fs;

/// # Types

/// An index for a module, pointing into the table of modules loaded into an environment.
pub type ModuleIndex = usize;

/// The type declaration of a location. This is the same as `SignatureToken` except it is not scoped
/// to a single module, and can contain Struct types coming from any of the modules in the
/// environment.
#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GlobalType {
    Bool,
    U8,
    U64,
    U128,
    ByteArray,
    Address,
    Vector(Box<GlobalType>),
    Struct(ModuleIndex, StructDefinitionIndex, Vec<GlobalType>),
    Reference(Box<GlobalType>),
    MutableReference(Box<GlobalType>),
    TypeParameter(TypeParameterIndex),
    // TODO: Separate, so we can't make References, etc. to these.
    Subrange, // spec type only.
}

impl GlobalType {
    /// Determines whether this is a reference.
    pub fn is_reference(&self) -> bool {
        match self {
            GlobalType::Reference(_) | GlobalType::MutableReference(_) => true,
            _ => false,
        }
    }

    /// Determines whether this is a mutual reference.
    pub fn is_mutual_reference(&self) -> bool {
        if let GlobalType::MutableReference(_) = self {
            true
        } else {
            false
        }
    }

    /// Instantiates type parameters in this type.
    pub fn instantiate(&self, params: &[GlobalType]) -> GlobalType {
        match self {
            GlobalType::TypeParameter(i) => params[*i as usize].clone(),
            GlobalType::Reference(bt) => GlobalType::Reference(Box::new(bt.instantiate(params))),
            GlobalType::MutableReference(bt) => {
                GlobalType::MutableReference(Box::new(bt.instantiate(params)))
            }
            GlobalType::Struct(midx, sidx, args) => GlobalType::Struct(
                *midx,
                *sidx,
                args.iter().map(|t| t.instantiate(params)).collect_vec(),
            ),
            _ => self.clone(),
        }
    }
}

/// A dummy type to represent an error. We use a type parameter for this, with an index
/// which no one will/can ever realistically use.
pub const ERROR_TYPE: GlobalType = GlobalType::TypeParameter(std::u16::MAX);

/// A dummy type to represent an unknown type.
pub const UNKNOWN_TYPE: GlobalType = GlobalType::TypeParameter(std::u16::MAX - 1);

/// Line/Column position pair.
pub type Position = (LineIndex, ColumnIndex);

/// # Global Environment

/// Global environment for a set of modules.
#[derive(Debug)]
pub struct GlobalEnv {
    /// Options passed via the cli.
    pub options: Options,

    /// List of loaded modules, in order they have been provided using `add`.
    module_data: Vec<ModuleData>,
}

impl GlobalEnv {
    /// Creates a new environment.
    pub fn new(options: Options) -> Self {
        GlobalEnv {
            options,
            module_data: vec![],
        }
    }

    /// Adds a new module to the environment. StructData and FunctionData need to be provided
    /// in definition index order. See `create_function_data` and `create_struct_data` for how
    /// to create them.
    pub fn add(
        &mut self,
        source_file_path: &str,
        module: VerifiedModule,
        source_map: ModuleSourceMap<Loc>,
        struct_data: Vec<StructData>,
        function_data: Vec<FunctionData>,
        synthetics: Vec<SyntheticDefinition>,
    ) {
        let idx = self.module_data.len();
        self.module_data.push(ModuleData {
            id: module.self_id(),
            idx,
            module,
            struct_data,
            function_data,
            synthetics: vec![],
            source_map,
            source_file_path: source_file_path.to_owned(),
            source_text: RefCell::new(None),
            diags: RefCell::new(vec![]),
        });
        let typed_synthetics = {
            let module_env = self.get_module(self.module_data.len() - 1);
            synthetics
                .into_iter()
                .map(|syn| {
                    let ty = module_env.translate_ast_type(syn.loc, &syn.value.type_, &[]);
                    (syn, ty)
                })
                .collect_vec()
        };
        self.module_data.last_mut().unwrap().synthetics = typed_synthetics;
    }

    /// Creates data for a function, adding any information not contained in bytecode. This is
    /// a helper for adding a new module to the environment.
    pub fn create_function_data(
        &self,
        module: &VerifiedModule,
        def_idx: FunctionDefinitionIndex,
        arg_names: Vec<Identifier>,
        type_arg_names: Vec<Identifier>,
        spec: Vec<Condition>,
    ) -> FunctionData {
        let handle_idx = module.function_def_at(def_idx).function;
        FunctionData {
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
        module: &VerifiedModule,
        def_idx: StructDefinitionIndex,
        invariants: Vec<Invariant>,
    ) -> StructData {
        let handle_idx = module.struct_def_at(def_idx).struct_handle;
        let field_data = if let StructFieldInformation::Declared {
            field_count,
            fields,
        } = module.struct_def_at(def_idx).field_information
        {
            (fields.0..fields.0 + field_count)
                .map(|idx| FieldData {
                    def_idx: FieldDefinitionIndex(idx),
                })
                .collect()
        } else {
            vec![]
        };
        let mut data_invariants = vec![];
        let mut update_invariants = vec![];
        let mut pack_invariants = vec![];
        let mut unpack_invariants = vec![];
        let mut other_invariants = vec![];

        for inv in invariants {
            match inv.value.modifier.as_str() {
                "" | DATA_MODIFIER => data_invariants.push(inv),
                UPDATE_MODIFIER => update_invariants.push(inv),
                PACK_MODIFIER => pack_invariants.push(inv),
                UNPACK_MODIFIER => unpack_invariants.push(inv),
                _ => other_invariants.push(inv),
            }
        }

        StructData {
            def_idx,
            handle_idx,
            field_data,
            data_invariants,
            update_invariants,
            pack_invariants,
            unpack_invariants,
            other_invariants,
        }
    }

    /// Finds a module by id and returns an environment for it.
    pub fn find_module<'env>(&'env self, id: &ModuleId) -> Option<ModuleEnv<'env>> {
        for module_data in &self.module_data {
            let module_env = ModuleEnv {
                env: self,
                data: module_data,
            };
            if module_env.get_id() == id {
                return Some(module_env);
            }
        }
        None
    }

    /// Finds a module by name and returns an environment for it.
    /// TODO: we may need to disallow this to support modules of the same name but with
    ///    different addresses in one verification session.
    pub fn find_module_by_name<'env>(&'env self, name: &str) -> Option<ModuleEnv<'env>> {
        self.get_modules()
            .find(|m| m.get_id().name().as_str() == name)
    }

    // Gets the number of modules in this environment.
    pub fn get_module_count(&self) -> usize {
        self.module_data.len()
    }

    /// Gets a module by index.
    pub fn get_module(&self, idx: usize) -> ModuleEnv<'_> {
        let module_data = &self.module_data[idx];
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
    pub fn get_bytecode_modules(&self) -> impl Iterator<Item = &VerifiedModule> {
        self.module_data
            .iter()
            .map(|module_data| &module_data.module)
    }

    /// Returns all structs in all modules which carry invariants.
    pub fn get_all_structs_with_invariants(&self) -> Vec<GlobalType> {
        let mut res = vec![];
        for module_env in self.get_modules() {
            for struct_env in module_env.get_structs() {
                if struct_env.has_invariants() {
                    let formals = struct_env
                        .get_type_parameters()
                        .iter()
                        .enumerate()
                        .map(|(idx, _)| GlobalType::TypeParameter(idx as u16))
                        .collect_vec();
                    res.push(GlobalType::Struct(
                        module_env.get_module_idx(),
                        struct_env.get_def_idx(),
                        formals,
                    ));
                }
            }
        }
        res
    }
}

/// # Module Environment

/// Represents data for a module.
#[derive(Debug)]
pub struct ModuleData {
    /// Module id (pair of address and name)
    id: ModuleId,

    /// Index of this module in the global env.
    idx: ModuleIndex,

    /// Module byte code.
    module: VerifiedModule,

    /// Struct data, in definition index order.
    struct_data: Vec<StructData>,

    /// Function data, in definition index order.
    function_data: Vec<FunctionData>,

    /// Synthetic variables.
    synthetics: Vec<(SyntheticDefinition, GlobalType)>,

    /// Module source location information.
    source_map: ModuleSourceMap<Loc>,

    /// File path to the source of this module.
    source_file_path: String,

    /// Cached source for text position calculation.
    source_text: RefCell<Option<String>>,

    // Accumulated diagnostics.
    diags: RefCell<Vec<Diagnostic>>,
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
    /// Returns the index of this module in the global env.
    pub fn get_module_idx(&self) -> ModuleIndex {
        self.data.idx
    }

    /// Returns the id of this module.
    pub fn get_id(&'env self) -> &'env ModuleId {
        &self.data.id
    }

    /// Adds diagnostic to the environment.
    pub fn add_diag(&self, diag: Diagnostic) {
        self.data.diags.borrow_mut().push(diag);
    }

    /// Returns true if diagnostics have error severity or worse.
    pub fn has_errors(&self) -> bool {
        self.data
            .diags
            .borrow()
            .iter()
            .any(|d| d.severity >= Severity::Error)
    }

    /// Reports diagnostics associated with this module.
    pub fn report_diagnostics(&self) {
        if !self.has_errors() {
            return;
        }
        self.ensure_source_available();
        let mut codemap = Files::new();
        codemap.add(
            &self.data.source_file_path,
            self.data.source_text.borrow().as_ref().unwrap().clone(),
        );
        for diag in self.data.diags.borrow().iter() {
            let writer = &mut StandardStream::stderr(ColorChoice::Auto);
            emit(writer, &Config::default(), &codemap, diag).expect("emitting diagnostic failed")
        }
    }

    /// Returns file name and line/column position for location in this module.
    pub fn get_position(&self, loc: Loc) -> (String, Position) {
        self.ensure_source_available();
        let source = self.data.source_text.borrow();
        let source_ref = source.as_ref().unwrap();
        let mut file_map = Files::new();
        let id = file_map.add(&self.data.source_file_path, source_ref);
        let location = file_map.location(id, loc.span().start()).unwrap();
        (
            self.data.source_file_path.clone(),
            (location.line, location.column),
        )
    }

    /// Helper to ensure source text is available.
    fn ensure_source_available(&self) {
        if self.data.source_text.borrow().is_some() {
            return;
        }
        let source =
            fs::read_to_string(&self.data.source_file_path).expect("original source not found");
        *self.data.source_text.borrow_mut() = Some(source);
    }

    /// Gets the underlying bytecode module.
    pub fn get_verified_module(&'env self) -> &'env VerifiedModule {
        &self.data.module
    }

    /// Gets a FunctionEnv in this module by name.
    pub fn find_function(&'env self, name: &IdentStr) -> Option<FunctionEnv<'env>> {
        // TODO: we may want to represent this as hash table or btree. For now we just search.
        for data in &self.data.function_data {
            let func_env = FunctionEnv {
                module_env: self.clone(),
                data,
            };
            if func_env.get_name() == name {
                return Some(func_env);
            }
        }
        None
    }

    /// Gets a FunctionEnv by index.
    pub fn get_function(&'env self, idx: FunctionDefinitionIndex) -> FunctionEnv<'env> {
        let data = &self.data.function_data[idx.0 as usize];
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
        self.data.function_data.iter().map(move |data| FunctionEnv {
            module_env: self.clone(),
            data,
        })
    }

    /// Returns the FunctionEnv which contains the location. This returns any function
    /// which location encloses the given one.
    pub fn get_enclosing_function(&'env self, loc: Loc) -> Option<FunctionEnv<'env>> {
        // Currently we do a brute-force linear search, may need to speed this up if it appears
        // to be a bottleneck.
        for func_env in self.get_functions() {
            let env_span = func_env.get_loc().span();
            let loc_span = loc.span();
            if env_span.start() <= loc_span.start() && loc_span.end() <= env_span.end() {
                return Some(func_env);
            }
        }
        None
    }

    /// Get ModuleEnv which declares the function called by this module, as described by a
    /// handle index. This also returns the definition index in that other (or same module)
    /// to access the FunctionEnv. The usage pattern is typically:
    ///
    /// ```ignore
    /// let (other_module_env, def_idx) = module_env.get_callee_info(handle_idx);
    /// let func_env = other_module_env.get_function(def_idx);
    /// ```
    ///
    /// Notice that because of Rust lifetime rules, we cannot(?) abstract these two lines in
    /// a single function. We need the first call to establish an owner of the `other_module_env`
    /// for which the `func_env` contains a reference.
    pub fn get_callee_info(
        &'env self,
        idx: FunctionHandleIndex,
    ) -> (ModuleEnv<'env>, FunctionDefinitionIndex) {
        let view =
            FunctionHandleView::new(&self.data.module, self.data.module.function_handle_at(idx));
        let module_env = self
            .env
            .find_module(&view.module_id())
            .expect("unexpected reference to module not found in global env");
        let func_env = module_env
            .find_function(view.name())
            .expect("unexpected reference to function not found in associated module");
        let def_idx = func_env.get_def_idx();
        (module_env, def_idx)
    }

    /// Gets a StructEnv in this module by name.
    pub fn find_struct(&'env self, name: &IdentStr) -> Option<StructEnv<'env>> {
        // TODO: we may want to represent this as hash table or btree. For now we just search.
        for data in &self.data.struct_data {
            let struct_env = StructEnv {
                module_env: self.clone(),
                data,
            };
            if struct_env.get_name() == name {
                return Some(struct_env);
            }
        }
        None
    }

    /// Gets a StructEnv by index.
    pub fn get_struct(&'env self, idx: StructDefinitionIndex) -> StructEnv<'env> {
        let data = &self.data.struct_data[idx.0 as usize];
        StructEnv {
            module_env: self.clone(),
            data,
        }
    }

    /// Gets a StructEnv by index.
    pub fn into_get_struct(self, idx: StructDefinitionIndex) -> StructEnv<'env> {
        let data = &self.data.struct_data[idx.0 as usize];
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
    pub fn get_struct_of_field(&'env self, idx: FieldDefinitionIndex) -> StructEnv<'env> {
        let field_view =
            FieldDefinitionView::new(&self.data.module, self.data.module.field_def_at(idx));
        let struct_name = field_view.member_of().name();
        self.find_struct(struct_name).expect("struct undefined")
    }

    /// Returns iterator over structs in this module.
    pub fn get_structs(&'env self) -> impl Iterator<Item = StructEnv<'env>> {
        self.data.struct_data.iter().map(move |data| StructEnv {
            module_env: self.clone(),
            data,
        })
    }

    /// Globalizes a signature local to this module.
    pub fn globalize_signature(&self, sig: &SignatureToken) -> GlobalType {
        match sig {
            SignatureToken::Bool => GlobalType::Bool,
            SignatureToken::U8 => GlobalType::U8,
            SignatureToken::U64 => GlobalType::U64,
            SignatureToken::U128 => GlobalType::U128,
            SignatureToken::Address => GlobalType::Address,
            SignatureToken::Vector(t) => GlobalType::Vector(Box::new(self.globalize_signature(t))),
            SignatureToken::Reference(t) => {
                GlobalType::Reference({ Box::new(self.globalize_signature(t)) })
            }
            SignatureToken::MutableReference(t) => {
                GlobalType::MutableReference({ Box::new(self.globalize_signature(t)) })
            }
            SignatureToken::TypeParameter(index) => GlobalType::TypeParameter(*index),
            SignatureToken::Struct(handle_idx, args) => {
                let struct_view = StructHandleView::new(
                    &self.data.module,
                    self.data.module.struct_handle_at(*handle_idx),
                );
                let declaring_module_env = self
                    .env
                    .find_module(&struct_view.module_id())
                    .expect("undefined module");
                let struct_env = declaring_module_env
                    .find_struct(struct_view.name())
                    .expect("undefined struct");
                GlobalType::Struct(
                    declaring_module_env.data.idx,
                    struct_env.get_def_idx(),
                    self.globalize_signatures(args),
                )
            }
        }
    }

    /// Globalizes a list of signatures.
    fn globalize_signatures(&self, sigs: &[SignatureToken]) -> Vec<GlobalType> {
        sigs.iter()
            .map(|s| self.globalize_signature(s))
            .collect_vec()
    }

    /// Gets a list of type actuals associated with the index in the bytecode.
    pub fn get_type_actuals(&self, idx: LocalsSignatureIndex) -> Vec<GlobalType> {
        let actuals = &self.data.module.locals_signature_at(idx).0;
        self.globalize_signatures(actuals)
    }

    /// Converts an address pool index for this module into a number representing the address.
    pub fn get_address(&self, idx: AddressPoolIndex) -> BigInt {
        let addr = &self.data.module.address_pool()[idx.0 as usize];
        BigInt::from_str_radix(&addr.to_string(), 16).unwrap()
    }

    /// Returns synthetic definitions in this module.
    pub fn get_synthetics(&'env self) -> &'env [(SyntheticDefinition, GlobalType)] {
        &self.data.synthetics
    }

    /// Find a synthetic by name.
    pub fn find_synthetic(
        &'env self,
        name: &str,
    ) -> Option<&'env (SyntheticDefinition, GlobalType)> {
        for syn in &self.data.synthetics {
            if syn.0.value.name.as_str() == name {
                return Some(syn);
            }
        }
        None
    }

    /// Translate an AST type declared in this module into a global type. Translation errors
    /// will be reported on the module, and the ERROR_TYPE will be used where they occur.
    pub fn translate_ast_type(
        &self,
        loc: Loc,
        ast_type: &Type,
        type_params: &[TypeParameter],
    ) -> GlobalType {
        match ast_type {
            Type::Address => GlobalType::Address,
            Type::U8 => GlobalType::U8,
            Type::U64 => GlobalType::U64,
            Type::U128 => GlobalType::U128,
            Type::Bool => GlobalType::Bool,
            Type::Vector(bt) => {
                GlobalType::Vector(Box::new(self.translate_ast_type(loc, bt, type_params)))
            }
            Type::Reference(is_mut, bt) => {
                if *is_mut {
                    GlobalType::MutableReference(Box::new(self.translate_ast_type(
                        loc,
                        bt,
                        type_params,
                    )))
                } else {
                    GlobalType::Reference(Box::new(self.translate_ast_type(loc, bt, type_params)))
                }
            }
            Type::TypeParameter(param) => {
                self.translate_type_param_ast_type(loc, param, type_params)
            }
            Type::Struct(ident, args) => {
                self.translate_struct_ast_type(loc, ident, args, type_params)
            }
        }
    }

    /// Translates a struct AST type into a GlobalType, context checking it.
    pub fn translate_struct_ast_type(
        &self,
        loc: Loc,
        ident: &QualifiedStructIdent,
        actuals: &[Type],
        type_params: &[TypeParameter],
    ) -> GlobalType {
        let struct_name = ident.name.as_inner();
        let mut module_name = ident.module.as_inner().to_string();
        if module_name == "Self" {
            module_name = self.get_id().name().to_string();
        }
        let translated_actuals = actuals
            .iter()
            .map(|t| self.translate_ast_type(loc, t, type_params))
            .collect_vec();
        if let Some(module_env) = self.env.find_module_by_name(&module_name) {
            if let Some(struct_env) = module_env.find_struct(IdentStr::new(struct_name).unwrap()) {
                let expected_count = struct_env.get_type_parameters().len();
                if expected_count != translated_actuals.len() {
                    self.error(
                        loc,
                        &format!(
                            "unexpected number of type actuals for `{}`; found {}, expected {}",
                            struct_env.get_name(),
                            expected_count,
                            translated_actuals.len()
                        ),
                        ERROR_TYPE,
                    )
                } else {
                    GlobalType::Struct(
                        module_env.get_module_idx(),
                        struct_env.get_def_idx(),
                        translated_actuals,
                    )
                }
            } else {
                self.error(
                    loc,
                    &format!("no struct `{}` in module `{}`", struct_name, module_name),
                    ERROR_TYPE,
                )
            }
        } else {
            self.error(loc, &format!("no module `{}`", module_name), ERROR_TYPE)
        }
    }

    /// Translate a type parameter AST type.
    fn translate_type_param_ast_type(
        &self,
        loc: Loc,
        param: &TypeVar_,
        type_params: &[TypeParameter],
    ) -> GlobalType {
        let name = param.name();
        if let Some(pos) = type_params.iter().position(|p| p.0.as_str() == name) {
            GlobalType::TypeParameter(pos as u16)
        } else {
            self.error(
                loc,
                &format!("cannot resolve type parameter `{}`", name),
                ERROR_TYPE,
            )
        }
    }

    /// Reports an error in this module.
    pub fn error<T>(&self, loc: Loc, msg: &str, pass_through: T) -> T {
        // FIXME real file id
        let id = Files::new().add(loc.file(), "");
        let diag = Diagnostic::new_error(msg, Label::new(id, loc.span(), ""));
        self.add_diag(diag);
        pass_through
    }
}

/// # Struct Environment

#[derive(Debug)]
pub struct StructData {
    /// The definition index of this struct in its module.
    def_idx: StructDefinitionIndex,

    /// The handle index of this struct in its module.
    handle_idx: StructHandleIndex,

    /// Field definitions.
    field_data: Vec<FieldData>,

    // Invariants
    data_invariants: Vec<Invariant>,
    update_invariants: Vec<Invariant>,
    pack_invariants: Vec<Invariant>,
    unpack_invariants: Vec<Invariant>,
    other_invariants: Vec<Invariant>,
}

const DATA_MODIFIER: &str = "data";
const PACK_MODIFIER: &str = "pack";
const UNPACK_MODIFIER: &str = "unpack";
const UPDATE_MODIFIER: &str = "update";

#[derive(Debug, Clone)]
pub struct StructEnv<'env> {
    /// Reference to enclosing module.
    pub module_env: ModuleEnv<'env>,

    /// Reference to the struct data.
    data: &'env StructData,
}

impl<'env> StructEnv<'env> {
    /// Returns the name of this struct.
    pub fn get_name(&self) -> &IdentStr {
        let handle = self
            .module_env
            .data
            .module
            .struct_handle_at(self.data.handle_idx);
        let view = StructHandleView::new(&self.module_env.data.module, handle);
        view.name()
    }

    /// Returns the location of this struct.
    pub fn get_loc(&self) -> Loc {
        if let Ok(source_map) = self
            .module_env
            .data
            .source_map
            .get_struct_source_map(self.data.def_idx)
        {
            source_map.decl_location
        } else {
            Spanned::unsafe_no_loc(()).loc
        }
    }

    /// Gets the definition index associated with this struct.
    pub fn get_def_idx(&self) -> StructDefinitionIndex {
        self.data.def_idx
    }

    /// Determines whether this struct is native.
    pub fn is_native(&self) -> bool {
        let def = self.module_env.data.module.struct_def_at(self.data.def_idx);
        def.field_information == StructFieldInformation::Native
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
        self.data.field_data.iter().map(move |data| FieldEnv {
            struct_env: self.clone(),
            data,
        })
    }

    /// Gets a field by its definition index.
    pub fn get_field(&'env self, idx: FieldDefinitionIndex) -> FieldEnv<'env> {
        for data in &self.data.field_data {
            if data.def_idx == idx {
                return FieldEnv {
                    struct_env: self.clone(),
                    data,
                };
            }
        }
        unreachable!();
    }

    /// Find a field by its name.
    pub fn find_field(&'env self, name: &IdentStr) -> Option<FieldEnv<'env>> {
        for data in &self.data.field_data {
            let env = FieldEnv {
                struct_env: self.clone(),
                data,
            };
            if env.get_name() == name {
                return Some(env);
            }
        }
        None
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
            .map(|(i, k)| TypeParameter(new_identifier(&format!("tv{}", i)), *k))
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

    /// Returns the other invariants associated with this struct.
    pub fn get_other_invariants(&'env self) -> &'env [Invariant] {
        &self.data.other_invariants
    }
}

/// # Field Environment

#[derive(Debug)]
pub struct FieldData {
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
    pub fn get_name(&self) -> &IdentStr {
        let def = self
            .struct_env
            .module_env
            .data
            .module
            .field_def_at(self.data.def_idx);
        self.struct_env
            .module_env
            .data
            .module
            .identifier_at(def.name)
    }

    /// Gets the type of this field.
    pub fn get_type(&self) -> GlobalType {
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

/// # Function Environment

/// Represents a type parameter.
#[derive(Debug, Clone)]
pub struct TypeParameter(pub Identifier, pub Kind);

/// Represents a parameter.
#[derive(Debug, Clone)]
pub struct Parameter(pub Identifier, pub GlobalType);

#[derive(Debug)]
pub struct FunctionData {
    /// The definition index of this function in its module.
    def_idx: FunctionDefinitionIndex,

    /// The handle index of this function in its module.
    handle_idx: FunctionHandleIndex,

    /// List of function argument names. Not in bytecode but obtained from AST.
    arg_names: Vec<Identifier>,

    /// List of type argument names. Not in bytecode but obtained from AST.
    type_arg_names: Vec<Identifier>,

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
    pub fn get_name(&self) -> &IdentStr {
        let view = self.handle_view();
        view.name()
    }

    /// Returns the location of this function.
    pub fn get_loc(&self) -> Loc {
        if let Ok(source_map) = self
            .module_env
            .data
            .source_map
            .get_function_source_map(self.data.def_idx)
        {
            source_map.decl_location
        } else {
            Spanned::unsafe_no_loc(()).loc
        }
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
            .map(|(i, k)| TypeParameter(new_identifier(&format!("tv{}", i)), *k))
            .collect_vec()
    }

    /// Returns the regular parameters associated with this function.
    pub fn get_parameters(&self) -> Vec<Parameter> {
        let view = self.definition_view();
        view.signature()
            .arg_tokens()
            .map(|tv: SignatureTokenView<VerifiedModule>| {
                self.module_env.globalize_signature(tv.signature_token())
            })
            .zip(self.data.arg_names.iter())
            .map(|(s, i)| Parameter(i.clone(), s))
            .collect_vec()
    }

    /// Returns return types of this function.
    pub fn get_return_types(&self) -> Vec<GlobalType> {
        let view = self.definition_view();
        view.signature()
            .return_tokens()
            .map(|tv: SignatureTokenView<VerifiedModule>| {
                self.module_env.globalize_signature(tv.signature_token())
            })
            .collect_vec()
    }

    /// Returns the number of return values of this function.
    pub fn get_return_count(&self) -> usize {
        let view = self.definition_view();
        view.signature().return_count()
    }

    /// Gets the definition index of this function.
    pub fn get_def_idx(&self) -> FunctionDefinitionIndex {
        self.data.def_idx
    }

    /// Get the name to be used for a local. If the local is an argument, use that for naming,
    /// otherwise generate a unique name.
    pub fn get_local_name(&self, idx: usize) -> String {
        if idx < self.data.arg_names.len() {
            return self.data.arg_names[idx as usize].to_string();
        }
        // Try to obtain name from source map.
        if let Ok(fmap) = self
            .module_env
            .data
            .source_map
            .get_function_source_map(self.data.def_idx)
        {
            if let Some((ident, _)) = fmap.get_local_name(idx as u64) {
                return ident;
            }
        }
        format!("__t{}", idx)
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
    pub fn get_local_type(&self, idx: usize) -> GlobalType {
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

    fn handle_view(&'env self) -> FunctionHandleView<'env, VerifiedModule> {
        FunctionHandleView::new(
            &self.module_env.data.module,
            self.module_env
                .data
                .module
                .function_handle_at(self.data.handle_idx),
        )
    }

    fn definition_view(&'env self) -> FunctionDefinitionView<'env, VerifiedModule> {
        FunctionDefinitionView::new(
            &self.module_env.data.module,
            self.module_env
                .data
                .module
                .function_def_at(self.data.def_idx),
        )
    }
}

/// Helper to create a new identifier.
fn new_identifier(s: &str) -> Identifier {
    Identifier::new(s).expect("valid identifier")
}
