// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides an environment -- global state -- for translation, including helper functions
//! to interpret metadata about the translation target.

use std::cell::RefCell;

use itertools::Itertools;
use num::{BigInt, Num};

use bytecode_source_map::source_map::ModuleSourceMap;
use bytecode_verifier::VerifiedModule;
use ir_to_bytecode_syntax::ast::Loc;
use ir_to_bytecode_syntax::spec_language_ast::Condition;
use libra_types::{identifier::IdentStr, identifier::Identifier, language_storage::ModuleId};
use vm::access::ModuleAccess;
use vm::file_format::{
    AddressPoolIndex, FieldDefinitionIndex, FunctionDefinitionIndex, FunctionHandleIndex, Kind,
    LocalsSignatureIndex, SignatureToken, StructDefinitionIndex, StructFieldInformation,
    StructHandleIndex, TypeParameterIndex,
};
use vm::views::{
    FieldDefinitionView, FunctionDefinitionView, FunctionHandleView, SignatureTokenView,
    StructDefinitionView, StructHandleView, ViewInternals,
};

use crate::cli::Options;

/// # Diagnostics

/// An index for a module, pointing into the table of modules loaded into an environment.
pub type ModuleIndex = usize;

/// A global unique location, consisting of a module index and a Span in its source.
#[derive(Debug, Copy, Clone)]
pub struct GlobalLocation(ModuleIndex, Loc);

/// Type to represent diagnosis, like an error message.
#[derive(Debug)]
pub struct Diag {
    /// A location for this diagnosis.
    pub location: GlobalLocation,

    /// The severity.
    pub level: DiagLevel,

    /// The message.
    pub message: String,
}

/// The severity level of a diagnosis.
#[derive(Debug)]
pub enum DiagLevel {
    ERROR,
    WARNING,
    HINT,
}

/// # Types

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
    Struct(ModuleIndex, StructDefinitionIndex, Vec<GlobalType>),
    Reference(Box<GlobalType>),
    MutableReference(Box<GlobalType>),
    TypeParameter(TypeParameterIndex),
}

impl GlobalType {
    pub fn is_reference(&self) -> bool {
        match self {
            GlobalType::Reference(_) | GlobalType::MutableReference(_) => true,
            _ => false,
        }
    }
}

/// # Global Environment

/// Global environment for a set of modules.
#[derive(Debug)]
pub struct GlobalEnv {
    /// Options passed via the cli.
    pub options: Options,

    /// List of loaded modules, in order they have been provided using add().
    pub module_data: Vec<ModuleData>,

    // Accumulated diagnostics.
    pub diags: RefCell<Vec<Diag>>,
}

impl GlobalEnv {
    /// Creates a new environment.
    pub fn new(options: Options) -> Self {
        GlobalEnv {
            options,
            module_data: vec![],
            diags: RefCell::new(vec![]),
        }
    }

    /// Adds diagnosis to the environment.
    pub fn add_diag(&self, loc: &GlobalLocation, level: DiagLevel, msg: &str) {
        self.diags.borrow_mut().push(Diag {
            location: *loc,
            level,
            message: msg.to_string(),
        })
    }

    /// Adds a new module to the environment. StructData and FunctionData need to be provided
    /// in definition index order. See `create_function_data` and `create_struct_data` for how
    /// to create them.
    pub fn add(
        &mut self,
        module: VerifiedModule,
        source_map: ModuleSourceMap<Loc>,
        struct_data: Vec<StructData>,
        function_data: Vec<FunctionData>,
    ) {
        let idx = self.module_data.len();
        self.module_data.push(ModuleData {
            id: module.self_id(),
            idx,
            module,
            struct_data,
            function_data,
            source_map,
        });
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

        StructData {
            def_idx,
            handle_idx,
            field_data,
        }
    }

    /// Finds a module by name and returns an environment for it.
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

    /// Gets a module by index.
    pub fn get_module<'env>(&'env self, idx: usize) -> ModuleEnv<'env> {
        let module_data = &self.module_data[idx];
        ModuleEnv {
            env: self,
            data: module_data,
        }
    }

    /// Returns an iterator for all modules in the environment.
    pub fn get_modules<'env>(&'env self) -> impl Iterator<Item = ModuleEnv<'env>> {
        self.module_data.iter().map(move |module_data| ModuleEnv {
            env: self,
            data: module_data,
        })
    }

    /// Returns an iterator for all bytecode modules in the environment.
    pub fn get_bytecode_modules<'env>(&'env self) -> impl Iterator<Item = &'env VerifiedModule> {
        self.module_data
            .iter()
            .map(|module_data| &module_data.module)
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

    /// Module source location information.
    source_map: ModuleSourceMap<Loc>,
}

/// Represents a module environment.
#[derive(Debug)]
pub struct ModuleEnv<'env> {
    /// Reference to the outer env.
    pub env: &'env GlobalEnv,

    /// Reference to the data of the module.
    data: &'env ModuleData,
}

impl<'env> ModuleEnv<'env> {
    /// Returns the index of this module in the global env.
    pub fn get_module_idx(&self) -> usize {
        self.data.idx
    }

    /// Returns the name of this module.
    pub fn get_id(&'env self) -> &'env ModuleId {
        &self.data.id
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
                module_env: self,
                data,
            };
            if func_env.get_name() == name {
                return Some(func_env);
            }
        }
        None
    }

    /// Gets a FunctionEnv by index.
    pub fn get_function(&'env self, idx: &FunctionDefinitionIndex) -> FunctionEnv<'env> {
        let data = &self.data.function_data[idx.0 as usize];
        FunctionEnv {
            module_env: self,
            data,
        }
    }

    /// Returns iterator over FunctionEnvs in this module.
    pub fn get_functions(&'env self) -> impl Iterator<Item = FunctionEnv<'env>> {
        self.data.function_data.iter().map(move |data| FunctionEnv {
            module_env: self,
            data,
        })
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
        idx: &FunctionHandleIndex,
    ) -> (ModuleEnv<'env>, FunctionDefinitionIndex) {
        let view =
            FunctionHandleView::new(&self.data.module, self.data.module.function_handle_at(*idx));
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
                module_env: self,
                data,
            };
            if struct_env.get_name() == name {
                return Some(struct_env);
            }
        }
        None
    }

    /// Gets a StructEnv by index.
    pub fn get_struct(&'env self, idx: &StructDefinitionIndex) -> StructEnv<'env> {
        let data = &self.data.struct_data[idx.0 as usize];
        StructEnv {
            module_env: self,
            data,
        }
    }

    /// Gets the struct declaring a field specified by FieldDefinitionIndex,
    /// as it is globally unique for this module.
    pub fn get_struct_of_field(&'env self, idx: &FieldDefinitionIndex) -> StructEnv<'env> {
        let field_view =
            FieldDefinitionView::new(&self.data.module, self.data.module.field_def_at(*idx));
        let struct_name = field_view.member_of().name();
        self.find_struct(struct_name).expect("struct undefined")
    }

    /// Returns iterator over structs in this module.
    pub fn get_structs(&'env self) -> impl Iterator<Item = StructEnv<'env>> {
        self.data.struct_data.iter().map(move |data| StructEnv {
            module_env: self,
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
            SignatureToken::ByteArray => GlobalType::ByteArray,
            SignatureToken::Address => GlobalType::Address,
            SignatureToken::Reference(t) => {
                GlobalType::Reference({ Box::new(self.globalize_signature(&*t)) })
            }
            SignatureToken::MutableReference(t) => {
                GlobalType::MutableReference({ Box::new(self.globalize_signature(&*t)) })
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
    pub fn get_address(&self, idx: &AddressPoolIndex) -> BigInt {
        let addr = &self.data.module.address_pool()[idx.0 as usize];
        BigInt::from_str_radix(&addr.to_string(), 16).unwrap()
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
}

#[derive(Debug)]
pub struct StructEnv<'env> {
    /// Reference to enclosing module.
    pub module_env: &'env ModuleEnv<'env>,

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

    /// Gets the definition index associated with this struct.
    pub fn get_def_idx(&self) -> StructDefinitionIndex {
        self.data.def_idx
    }

    /// Determines whether this struct is native.
    pub fn is_native(&self) -> bool {
        let def = self.module_env.data.module.struct_def_at(self.data.def_idx);
        def.field_information == StructFieldInformation::Native
    }

    /// Get an iterator for the fields.
    pub fn get_fields(&'env self) -> impl Iterator<Item = FieldEnv<'env>> {
        self.data.field_data.iter().map(move |data| FieldEnv {
            struct_env: self,
            data,
        })
    }

    /// Gets a field by its definition index.
    pub fn get_field(&'env self, idx: &FieldDefinitionIndex) -> FieldEnv<'env> {
        for data in &self.data.field_data {
            if data.def_idx == *idx {
                return FieldEnv {
                    struct_env: self,
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
                struct_env: self,
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
    pub struct_env: &'env StructEnv<'env>,

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
    pub module_env: &'env ModuleEnv<'env>,

    /// Reference to the function data.
    data: &'env FunctionData,
}

impl<'env> FunctionEnv<'env> {
    /// Returns the name of this function.
    pub fn get_name(&self) -> &IdentStr {
        let handle = self
            .module_env
            .data
            .module
            .function_handle_at(self.data.handle_idx);
        let view = FunctionHandleView::new(&self.module_env.data.module, handle);
        view.name()
    }

    /// Returns true if this function is native.
    pub fn is_native(&self) -> bool {
        let view = FunctionDefinitionView::new(
            &self.module_env.data.module,
            self.module_env
                .data
                .module
                .function_def_at(self.data.def_idx),
        );
        view.is_native()
    }

    /// Returns the type parameters associated with this function.
    pub fn get_type_parameters(&self) -> Vec<TypeParameter> {
        // TODO: currently the translation scheme isn't working with using real type
        //   parameter names, so use indices instead.
        let view = FunctionDefinitionView::new(
            &self.module_env.data.module,
            self.module_env
                .data
                .module
                .function_def_at(self.data.def_idx),
        );
        view.signature()
            .type_formals()
            .iter()
            .enumerate()
            .map(|(i, k)| TypeParameter(new_identifier(&format!("tv{}", i)), *k))
            .collect_vec()
    }

    /// Returns the regular parameters associated with this function.
    pub fn get_parameters(&self) -> Vec<Parameter> {
        let view = FunctionDefinitionView::new(
            &self.module_env.data.module,
            self.module_env
                .data
                .module
                .function_def_at(self.data.def_idx),
        );
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
        let view = FunctionDefinitionView::new(
            &self.module_env.data.module,
            self.module_env
                .data
                .module
                .function_def_at(self.data.def_idx),
        );
        view.signature()
            .return_tokens()
            .map(|tv: SignatureTokenView<VerifiedModule>| {
                self.module_env.globalize_signature(tv.signature_token())
            })
            .collect_vec()
    }

    /// Gets the definition index of this function.
    pub fn get_def_idx(&self) -> FunctionDefinitionIndex {
        self.data.def_idx
    }

    /// Get the name to be used for a local. If the local is an argument, use that for naming,
    /// otherwise generate a unique name.
    pub fn get_local_name(&self, idx: u8) -> String {
        if (idx as usize) < self.data.arg_names.len() {
            return self.data.arg_names[idx as usize].to_string();
        }
        format!("t{}", idx)
    }

    /// Returns specification conditions associated with this function.
    pub fn get_specification(&'env self) -> &'env [Condition] {
        &self.data.spec
    }
}

/// Helper to create a new identifier.
fn new_identifier(s: &str) -> Identifier {
    Identifier::new(s).expect("valid identifier")
}
