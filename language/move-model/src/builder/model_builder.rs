// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Translates and validates specification language fragments as they are output from the Move
//! compiler's expansion phase and adds them to the environment (which was initialized from the
//! byte code). This includes identifying the Move sub-language supported by the specification
//! system, as well as type checking it and translating it to the spec language ast.

use std::collections::{BTreeMap, BTreeSet};

#[allow(unused_imports)]
use log::{debug, info, warn};
use num::BigUint;

use move_lang::{expansion::ast as EA, parser::ast as PA, shared::AddressBytes};
use move_symbol_pool::Symbol as MoveStringSymbol;

use crate::{
    ast::{ModuleName, Operation, QualifiedSymbol, Spec, Value},
    builder::spec_builtins,
    model::{
        FunId, FunctionVisibility, GlobalEnv, Loc, ModuleId, QualifiedId, SpecFunId, SpecVarId,
        StructId,
    },
    project_2nd,
    symbol::Symbol,
    ty::Type,
};
use codespan_reporting::diagnostic::Severity;

/// A builder is used to enter a sequence of modules in acyclic dependency order into the model. The
/// builder maintains the incremental state of this process, such that the various tables
/// are extended with each module translated. Each table is a mapping from fully qualified names
/// (module names plus item name in the module) to the entity.
#[derive(Debug)]
pub(crate) struct ModelBuilder<'env> {
    /// The global environment we are building.
    pub env: &'env mut GlobalEnv,
    /// Set of known named addresses provided by the compiler
    pub named_address_mapping: BTreeMap<MoveStringSymbol, AddressBytes>,
    /// A symbol table for specification functions. Because of overloading, and entry can
    /// contain multiple functions.
    pub spec_fun_table: BTreeMap<QualifiedSymbol, Vec<SpecFunEntry>>,
    /// A symbol table for specification variables.
    pub spec_var_table: BTreeMap<QualifiedSymbol, SpecVarEntry>,
    /// A symbol table for specification schemas.
    pub spec_schema_table: BTreeMap<QualifiedSymbol, SpecSchemaEntry>,
    /// A symbol table storing unused schemas, used later to generate warnings. All schemas
    /// are initially in the table and are removed when they are used in expressions.
    pub unused_schema_set: BTreeSet<QualifiedSymbol>,
    // A symbol table for structs.
    pub struct_table: BTreeMap<QualifiedSymbol, StructEntry>,
    /// A reverse mapping from ModuleId/StructId pairs to QualifiedSymbol. This
    /// is used for visualization of types in error messages.
    pub reverse_struct_table: BTreeMap<(ModuleId, StructId), QualifiedSymbol>,
    /// A symbol table for functions.
    pub fun_table: BTreeMap<QualifiedSymbol, FunEntry>,
    /// A symbol table for constants.
    pub const_table: BTreeMap<QualifiedSymbol, ConstEntry>,
    /// A call graph mapping callers to callees that are Move functions.
    pub move_fun_call_graph: BTreeMap<QualifiedId<SpecFunId>, BTreeSet<QualifiedId<SpecFunId>>>,
}

/// A declaration of a specification function or operator in the builders state.
#[derive(Debug, Clone)]
pub(crate) struct SpecFunEntry {
    pub loc: Loc,
    pub oper: Operation,
    pub type_params: Vec<Type>,
    pub arg_types: Vec<Type>,
    pub result_type: Type,
}

/// A declaration of a specification variable in the builders state.
#[derive(Debug, Clone)]
pub(crate) struct SpecVarEntry {
    pub loc: Loc,
    pub module_id: ModuleId,
    pub var_id: SpecVarId,
    pub type_params: Vec<Type>,
    pub type_: Type,
}

/// A declaration of a schema in the builders state.
#[derive(Debug)]
pub(crate) struct SpecSchemaEntry {
    pub loc: Loc,
    pub name: QualifiedSymbol,
    pub module_id: ModuleId,
    pub type_params: Vec<(Symbol, Type)>,
    // The local variables declared in the schema.
    pub vars: Vec<(Symbol, Type)>,
    // The specifications in in this schema.
    pub spec: Spec,
    // All variables in scope of this schema, including those introduced by included schemas.
    pub all_vars: BTreeMap<Symbol, LocalVarEntry>,
    // The specification included from other schemas, after renaming and type instantiation.
    pub included_spec: Spec,
}

/// A declaration of a struct.
#[derive(Debug, Clone)]
pub(crate) struct StructEntry {
    pub loc: Loc,
    pub module_id: ModuleId,
    pub struct_id: StructId,
    pub is_resource: bool,
    pub type_params: Vec<(Symbol, Type)>,
    pub fields: Option<BTreeMap<Symbol, (usize, Type)>>,
}

/// A declaration of a function.
#[derive(Debug, Clone)]
pub(crate) struct FunEntry {
    pub loc: Loc,
    pub module_id: ModuleId,
    pub fun_id: FunId,
    pub visibility: FunctionVisibility,
    pub type_params: Vec<(Symbol, Type)>,
    pub params: Vec<(Symbol, Type)>,
    pub result_type: Type,
    pub is_pure: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ConstEntry {
    pub loc: Loc,
    pub ty: Type,
    pub value: Value,
}

impl<'env> ModelBuilder<'env> {
    /// Creates a builders.
    pub fn new(
        env: &'env mut GlobalEnv,
        named_address_mapping: BTreeMap<MoveStringSymbol, AddressBytes>,
    ) -> Self {
        let mut translator = ModelBuilder {
            env,
            named_address_mapping,
            spec_fun_table: BTreeMap::new(),
            spec_var_table: BTreeMap::new(),
            spec_schema_table: BTreeMap::new(),
            unused_schema_set: BTreeSet::new(),
            struct_table: BTreeMap::new(),
            reverse_struct_table: BTreeMap::new(),
            fun_table: BTreeMap::new(),
            const_table: BTreeMap::new(),
            move_fun_call_graph: BTreeMap::new(),
        };
        spec_builtins::declare_spec_builtins(&mut translator);
        translator
    }

    /// Shortcut for translating a Move AST location into ours.
    pub fn to_loc(&self, loc: &move_ir_types::location::Loc) -> Loc {
        self.env.to_loc(loc)
    }

    /// Reports a type checking error.
    pub fn error(&self, at: &Loc, msg: &str) {
        self.env.error(at, msg)
    }

    /// Reports a type checking error with notes.
    pub fn error_with_notes(&self, at: &Loc, msg: &str, notes: Vec<String>) {
        self.env.error_with_notes(at, msg, notes)
    }

    /// Defines a spec function, adding it to the spec fun table.
    pub fn define_spec_fun(&mut self, name: QualifiedSymbol, entry: SpecFunEntry) {
        // TODO: check whether overloads are distinguishable
        self.spec_fun_table
            .entry(name)
            .or_insert_with(Vec::new)
            .push(entry);
    }

    /// Defines a spec variable.
    pub fn define_spec_var(
        &mut self,
        loc: &Loc,
        name: QualifiedSymbol,
        module_id: ModuleId,
        var_id: SpecVarId,
        type_params: Vec<Type>,
        type_: Type,
    ) {
        let entry = SpecVarEntry {
            loc: loc.clone(),
            module_id,
            var_id,
            type_params,
            type_,
        };
        if let Some(old) = self.spec_var_table.insert(name.clone(), entry) {
            let var_name = name.display(self.env.symbol_pool());
            self.error(loc, &format!("duplicate declaration of `{}`", var_name));
            self.error(&old.loc, &format!("previous declaration of `{}`", var_name));
        }
    }

    /// Defines a spec schema.
    pub fn define_spec_schema(
        &mut self,
        loc: &Loc,
        name: QualifiedSymbol,
        module_id: ModuleId,
        type_params: Vec<(Symbol, Type)>,
        vars: Vec<(Symbol, Type)>,
    ) {
        let entry = SpecSchemaEntry {
            loc: loc.clone(),
            name: name.clone(),
            module_id,
            type_params,
            vars,
            spec: Spec::default(),
            all_vars: BTreeMap::new(),
            included_spec: Spec::default(),
        };
        if let Some(old) = self.spec_schema_table.insert(name.clone(), entry) {
            let schema_display = name.display(self.env.symbol_pool());
            self.error(
                loc,
                &format!("duplicate declaration of `{}`", schema_display),
            );
            self.error(
                &old.loc,
                &format!("previous declaration of `{}`", schema_display),
            );
        }
        self.unused_schema_set.insert(name);
    }

    /// Defines a struct type.
    pub fn define_struct(
        &mut self,
        loc: Loc,
        name: QualifiedSymbol,
        module_id: ModuleId,
        struct_id: StructId,
        is_resource: bool,
        type_params: Vec<(Symbol, Type)>,
        fields: Option<BTreeMap<Symbol, (usize, Type)>>,
    ) {
        let entry = StructEntry {
            loc,
            module_id,
            struct_id,
            is_resource,
            type_params,
            fields,
        };
        // Duplicate declarations have been checked by the Move compiler.
        assert!(self.struct_table.insert(name.clone(), entry).is_none());
        self.reverse_struct_table
            .insert((module_id, struct_id), name);
    }

    /// Defines a function.
    pub fn define_fun(
        &mut self,
        loc: Loc,
        name: QualifiedSymbol,
        module_id: ModuleId,
        fun_id: FunId,
        visibility: FunctionVisibility,
        type_params: Vec<(Symbol, Type)>,
        params: Vec<(Symbol, Type)>,
        result_type: Type,
    ) {
        let entry = FunEntry {
            loc,
            module_id,
            fun_id,
            visibility,
            type_params,
            params,
            result_type,
            is_pure: false,
        };
        // Duplicate declarations have been checked by the Move compiler.
        assert!(self.fun_table.insert(name, entry).is_none());
    }

    /// Defines a constant.
    pub fn define_const(&mut self, name: QualifiedSymbol, entry: ConstEntry) {
        // Duplicate declarations have been checked by the Move compiler.
        assert!(self.const_table.insert(name, entry).is_none());
    }

    pub fn resolve_address(&self, loc: &Loc, addr: &EA::Address) -> AddressBytes {
        match addr {
            EA::Address::Anonymous(bytes) => bytes.value,
            EA::Address::Named(n) => self
                .named_address_mapping
                .get(&n.value)
                .cloned()
                .unwrap_or_else(|| {
                    self.error(loc, &format!("Undeclared address `{}`", n));
                    AddressBytes::DEFAULT_ERROR_BYTES
                }),
        }
    }

    /// Looks up a type (struct), reporting an error if it is not found.
    pub fn lookup_type(&self, loc: &Loc, name: &QualifiedSymbol) -> Type {
        self.struct_table
            .get(name)
            .cloned()
            .map(|e| Type::Struct(e.module_id, e.struct_id, project_2nd(&e.type_params)))
            .unwrap_or_else(|| {
                self.error(
                    loc,
                    &format!("undeclared `{}`", name.display_full(self.env.symbol_pool())),
                );
                Type::Error
            })
    }

    // Generate warnings about unused schemas.
    pub fn warn_unused_schemas(&self) {
        for name in &self.unused_schema_set {
            let entry = self.spec_schema_table.get(name).expect("schema defined");
            let schema_name = name.display_simple(self.env.symbol_pool()).to_string();
            let module_env = self.env.get_module(entry.module_id);
            // Warn about unused schema only if the module is a target and schema name
            // does not start with 'UNUSED'
            if module_env.is_target() && !schema_name.starts_with("UNUSED") {
                self.env.diag(
                    Severity::Note,
                    &entry.loc,
                    &format!("unused schema {}", name.display(self.env.symbol_pool())),
                );
            }
        }
    }

    /// Returns the symbol for a binary op.
    pub fn bin_op_symbol(&self, op: &PA::BinOp_) -> QualifiedSymbol {
        QualifiedSymbol {
            module_name: self.builtin_module(),
            symbol: self.env.symbol_pool().make(op.symbol()),
        }
    }

    /// Returns the symbol for a unary op.
    pub fn unary_op_symbol(&self, op: &PA::UnaryOp_) -> QualifiedSymbol {
        QualifiedSymbol {
            module_name: self.builtin_module(),
            symbol: self.env.symbol_pool().make(op.symbol()),
        }
    }

    /// Returns the symbol for a name in the builtin module.
    pub fn builtin_qualified_symbol(&self, name: &str) -> QualifiedSymbol {
        QualifiedSymbol {
            module_name: self.builtin_module(),
            symbol: self.env.symbol_pool().make(name),
        }
    }

    /// Returns the symbol for the builtin function `old`.
    pub fn old_symbol(&self) -> Symbol {
        self.env.symbol_pool().make("old")
    }

    /// Returns the symbol for the builtin Move function `assert`.
    pub fn assert_symbol(&self) -> Symbol {
        self.env.symbol_pool().make("assert")
    }

    /// Returns the name for the pseudo builtin module.
    pub fn builtin_module(&self) -> ModuleName {
        ModuleName::new(BigUint::default(), self.env.symbol_pool().make("$$"))
    }

    /// Adds a spec function to used_spec_funs set.
    pub fn add_used_spec_fun(&mut self, qid: QualifiedId<SpecFunId>) {
        self.env.used_spec_funs.insert(qid);
        self.propagate_move_fun_usage(qid);
    }

    /// Adds an edge from the caller to the callee to the Move fun call graph. The callee is
    /// is instantiated in dependency of the type parameters of the caller.
    pub fn add_edge_to_move_fun_call_graph(
        &mut self,
        caller: QualifiedId<SpecFunId>,
        callee: QualifiedId<SpecFunId>,
    ) {
        self.move_fun_call_graph
            .entry(caller)
            .or_default()
            .insert(callee);
    }

    /// Runs DFS to propagate the usage of Move functions from callers
    /// to callees on the call graph.
    pub fn propagate_move_fun_usage(&mut self, qid: QualifiedId<SpecFunId>) {
        if let Some(neighbors) = self.move_fun_call_graph.get(&qid) {
            neighbors.clone().iter().for_each(|n| {
                if self.env.used_spec_funs.insert(*n) {
                    // If the callee's usage has not been recorded, recursively
                    // propagate the usage to the callee's callees, and so on.
                    self.propagate_move_fun_usage(*n);
                }
            });
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LocalVarEntry {
    pub loc: Loc,
    pub type_: Type,
    /// If this local is associated with an operation, this is set.
    pub operation: Option<Operation>,
    /// If this a temporary from Move code, this is it's index.
    pub temp_index: Option<usize>,
}
