// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Translates and validates specification language fragments as they are output from the Move
//! compiler's expansion phase and adds them to the environment (which was initialized from the
//! byte code). This includes identifying the Move sub-language supported by the specification
//! system, as well as type checking it and translating it to the spec language ast.

use std::collections::{BTreeMap, BTreeSet, LinkedList, VecDeque};

use itertools::Itertools;
use num::{BigUint, FromPrimitive, Num};

use bytecode_source_map::source_map::SourceMap;
#[allow(unused_imports)]
use log::{debug, info, warn};
use move_lang::{
    compiled_unit::{FunctionInfo, SpecInfo},
    expansion::ast::{self as EA},
    hlir::ast::{BaseType, SingleType},
    naming::ast::TParam,
    parser::ast::{self as PA, FunctionName},
    shared::{unique_map::UniqueMap, Name},
};
use vm::{
    access::ModuleAccess,
    file_format::{FunctionDefinitionIndex, StructDefinitionIndex},
    views::{FunctionHandleView, StructHandleView},
    CompiledModule,
};

use crate::{
    ast::{
        Condition, ConditionKind, Exp, LocalVarDecl, ModuleName, Operation, QualifiedSymbol, Spec,
        SpecFunDecl, SpecVarDecl, Value,
    },
    env::{
        FieldId, FunId, FunctionData, GlobalEnv, Loc, ModuleId, MoveIrLoc, NodeId, SpecFunId,
        SpecVarId, StructData, StructId, SCRIPT_AST_FUN_NAME, SCRIPT_BYTECODE_FUN_NAME,
    },
    project_1st, project_2nd,
    symbol::{Symbol, SymbolPool},
    ty::{PrimitiveType, Substitution, Type, TypeDisplayContext, BOOL_TYPE},
};
use move_ir_types::location::Spanned;
use move_lang::parser::ast::BinOp_;
use regex::Regex;

// =================================================================================================
/// # Translator

/// A translator. This is used to translate a sequence of modules in acyclic dependency order. The
/// translator maintains the incremental state of this process, such that the various tables
/// are extended with each module translated. Each table is a mapping from fully qualified names
/// (module names plus item name in the module) to the entity.
#[derive(Debug)]
pub struct Translator<'env> {
    /// The global environment we are building.
    pub env: &'env mut GlobalEnv,
    /// A symbol table for specification functions. Because of overloading, and entry can
    /// contain multiple functions.
    spec_fun_table: BTreeMap<QualifiedSymbol, Vec<SpecFunEntry>>,
    /// A symbol table for specification variables.
    spec_var_table: BTreeMap<QualifiedSymbol, SpecVarEntry>,
    /// A symbol table for specification schemas.
    spec_schema_table: BTreeMap<QualifiedSymbol, SpecSchemaEntry>,
    // A symbol table for structs.
    struct_table: BTreeMap<QualifiedSymbol, StructEntry>,
    /// A reverse mapping from ModuleId/StructId pairs to QualifiedSymbol. This
    /// is used for visualization of types in error messages.
    reverse_struct_table: BTreeMap<(ModuleId, StructId), QualifiedSymbol>,
    /// A symbol table for functions.
    fun_table: BTreeMap<QualifiedSymbol, FunEntry>,
}

/// A declaration of a specification function or operator in the translator state.
#[derive(Debug, Clone)]
struct SpecFunEntry {
    loc: Loc,
    oper: Operation,
    type_params: Vec<Type>,
    arg_types: Vec<Type>,
    result_type: Type,
}

/// A declaration of a specification variable in the translator state.
#[derive(Debug, Clone)]
struct SpecVarEntry {
    loc: Loc,
    module_id: ModuleId,
    var_id: SpecVarId,
    type_params: Vec<Type>,
    type_: Type,
}

/// A declaration of a schema in the translator state.
#[derive(Debug)]
struct SpecSchemaEntry {
    loc: Loc,
    name: QualifiedSymbol,
    module_id: ModuleId,
    type_params: Vec<(Symbol, Type)>,
    // The local variables declared in the schema.
    vars: Vec<(Symbol, Type)>,
    // The specifications in in this schema.
    spec: Spec,
    // All variables in scope of this schema, including those introduced by included schemas.
    all_vars: BTreeMap<Symbol, LocalVarEntry>,
    // The specification included from other schemas, after renaming and type instantiation.
    included_spec: Spec,
}

/// A declaration of a struct.
#[derive(Debug, Clone)]
struct StructEntry {
    loc: Loc,
    module_id: ModuleId,
    struct_id: StructId,
    is_resource: bool,
    type_params: Vec<(Symbol, Type)>,
    fields: Option<BTreeMap<Symbol, (usize, Type)>>,
}

/// A declaration of a function.
#[derive(Debug, Clone)]
struct FunEntry {
    loc: Loc,
    module_id: ModuleId,
    fun_id: FunId,
    is_public: bool,
    type_params: Vec<(Symbol, Type)>,
    params: Vec<(Symbol, Type)>,
    result_type: Type,
}

/// ## General

impl<'env> Translator<'env> {
    /// Creates a translator.
    pub fn new(env: &'env mut GlobalEnv) -> Self {
        let mut translator = Translator {
            env,
            spec_fun_table: BTreeMap::new(),
            spec_var_table: BTreeMap::new(),
            spec_schema_table: BTreeMap::new(),
            struct_table: BTreeMap::new(),
            reverse_struct_table: BTreeMap::new(),
            fun_table: BTreeMap::new(),
        };
        translator.declare_builtins();
        translator
    }

    /// Shortcut for translating a Move AST location into ours.
    pub fn to_loc(&self, loc: &move_ir_types::location::Loc) -> Loc {
        self.env.to_loc(loc)
    }

    /// Reports a type checking error.
    fn error(&self, at: &Loc, msg: &str) {
        self.env.error(at, msg)
    }

    /// Defines a spec function.
    fn define_spec_fun(
        &mut self,
        loc: &Loc,
        name: QualifiedSymbol,
        module_id: ModuleId,
        fun_id: SpecFunId,
        type_params: Vec<Type>,
        arg_types: Vec<Type>,
        result_type: Type,
    ) {
        let entry = SpecFunEntry {
            loc: loc.clone(),
            oper: Operation::Function(module_id, fun_id),
            type_params,
            arg_types,
            result_type,
        };
        // TODO: check whether overloads are distinguishable
        self.spec_fun_table
            .entry(name)
            .or_insert_with(|| vec![])
            .push(entry);
    }

    /// Defines a spec variable.
    fn define_spec_var(
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
    fn define_spec_schema(
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
    }

    /// Defines a struct type.
    fn define_struct(
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
        // Duplicate declarations have been checked by the move compiler.
        assert!(self.struct_table.insert(name.clone(), entry).is_none());
        self.reverse_struct_table
            .insert((module_id, struct_id), name);
    }

    /// Defines a function.
    fn define_fun(
        &mut self,
        loc: Loc,
        name: QualifiedSymbol,
        module_id: ModuleId,
        fun_id: FunId,
        is_public: bool,
        type_params: Vec<(Symbol, Type)>,
        params: Vec<(Symbol, Type)>,
        result_type: Type,
    ) {
        let entry = FunEntry {
            loc,
            module_id,
            fun_id,
            is_public,
            type_params,
            params,
            result_type,
        };
        // Duplicate declarations have been checked by the move compiler.
        assert!(self.fun_table.insert(name, entry).is_none());
    }

    /// Looks up a type (struct), reporting an error if it is not found.
    fn lookup_type(&self, loc: &Loc, name: &QualifiedSymbol) -> Type {
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
}

/// # Builtins

impl<'env> Translator<'env> {
    /// Declares builtins in the translator. This adds functions and operators
    /// to the translator which will be treated the same as user defined specification functions
    /// during translation.
    fn declare_builtins(&mut self) {
        let loc = self.env.internal_loc();
        let bool_t = &Type::new_prim(PrimitiveType::Bool);
        let num_t = &Type::new_prim(PrimitiveType::Num);
        let range_t = &Type::new_prim(PrimitiveType::Range);
        let address_t = &Type::new_prim(PrimitiveType::Address);
        let param_t = &Type::TypeParameter(0);
        let add_builtin = |trans: &mut Translator, name: QualifiedSymbol, entry: SpecFunEntry| {
            trans
                .spec_fun_table
                .entry(name)
                .or_insert_with(|| vec![])
                .push(entry);
        };

        {
            // Binary operators.
            let mut declare_bin =
                |op: PA::BinOp_, oper: Operation, param_type: &Type, result_type: &Type| {
                    add_builtin(
                        self,
                        self.bin_op_symbol(&op),
                        SpecFunEntry {
                            loc: loc.clone(),
                            oper,
                            type_params: vec![],
                            arg_types: vec![param_type.clone(), param_type.clone()],
                            result_type: result_type.clone(),
                        },
                    );
                };
            use PA::BinOp_::*;
            declare_bin(Add, Operation::Add, num_t, num_t);
            declare_bin(Sub, Operation::Sub, num_t, num_t);
            declare_bin(Mul, Operation::Mul, num_t, num_t);
            declare_bin(Mod, Operation::Mod, num_t, num_t);
            declare_bin(Div, Operation::Div, num_t, num_t);
            declare_bin(BitOr, Operation::BitOr, num_t, num_t);
            declare_bin(BitAnd, Operation::BitAnd, num_t, num_t);
            declare_bin(Xor, Operation::Xor, num_t, num_t);
            declare_bin(Shl, Operation::Shl, num_t, num_t);
            declare_bin(Shr, Operation::Shr, num_t, num_t);

            declare_bin(Range, Operation::Range, num_t, range_t);

            declare_bin(Implies, Operation::Implies, bool_t, bool_t);
            declare_bin(And, Operation::And, bool_t, bool_t);
            declare_bin(Or, Operation::Or, bool_t, bool_t);

            declare_bin(Lt, Operation::Lt, num_t, bool_t);
            declare_bin(Le, Operation::Le, num_t, bool_t);
            declare_bin(Gt, Operation::Gt, num_t, bool_t);
            declare_bin(Ge, Operation::Ge, num_t, bool_t);

            // Eq and Neq have special treatment because they are generic.
            add_builtin(
                self,
                self.bin_op_symbol(&PA::BinOp_::Eq),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::Eq,
                    type_params: vec![param_t.clone()],
                    arg_types: vec![param_t.clone(), param_t.clone()],
                    result_type: bool_t.clone(),
                },
            );
            add_builtin(
                self,
                self.bin_op_symbol(&PA::BinOp_::Neq),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::Neq,
                    type_params: vec![param_t.clone()],
                    arg_types: vec![param_t.clone(), param_t.clone()],
                    result_type: bool_t.clone(),
                },
            );
        }

        {
            // Unary operators.
            add_builtin(
                self,
                self.unary_op_symbol(&PA::UnaryOp_::Not),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::Not,
                    type_params: vec![],
                    arg_types: vec![bool_t.clone()],
                    result_type: bool_t.clone(),
                },
            );
        }

        {
            // Builtin functions.
            let vector_t = &Type::Vector(Box::new(param_t.clone()));
            let pred_t = &Type::Fun(vec![param_t.clone()], Box::new(bool_t.clone()));
            let pred_num_t = &Type::Fun(vec![num_t.clone()], Box::new(bool_t.clone()));

            // Transaction metadata
            add_builtin(
                self,
                self.builtin_fun_symbol("sender"),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::Sender,
                    type_params: vec![],
                    arg_types: vec![],
                    result_type: address_t.clone(),
                },
            );

            // constants (max_u8(), etc.)
            add_builtin(
                self,
                self.builtin_fun_symbol("max_u8"),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::MaxU8,
                    type_params: vec![],
                    arg_types: vec![],
                    result_type: num_t.clone(),
                },
            );

            add_builtin(
                self,
                self.builtin_fun_symbol("max_u64"),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::MaxU64,
                    type_params: vec![],
                    arg_types: vec![],
                    result_type: num_t.clone(),
                },
            );

            add_builtin(
                self,
                self.builtin_fun_symbol("max_u128"),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::MaxU128,
                    type_params: vec![],
                    arg_types: vec![],
                    result_type: num_t.clone(),
                },
            );

            // Vectors
            add_builtin(
                self,
                self.builtin_fun_symbol("len"),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::Len,
                    type_params: vec![param_t.clone()],
                    arg_types: vec![vector_t.clone()],
                    result_type: num_t.clone(),
                },
            );
            add_builtin(
                self,
                self.builtin_fun_symbol("all"),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::All,
                    type_params: vec![param_t.clone()],
                    arg_types: vec![vector_t.clone(), pred_t.clone()],
                    result_type: bool_t.clone(),
                },
            );
            add_builtin(
                self,
                self.builtin_fun_symbol("any"),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::Any,
                    type_params: vec![param_t.clone()],
                    arg_types: vec![vector_t.clone(), pred_t.clone()],
                    result_type: bool_t.clone(),
                },
            );
            add_builtin(
                self,
                self.builtin_fun_symbol("update"),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::Update,
                    type_params: vec![param_t.clone()],
                    arg_types: vec![vector_t.clone(), num_t.clone(), param_t.clone()],
                    result_type: vector_t.clone(),
                },
            );

            // Ranges
            add_builtin(
                self,
                self.builtin_fun_symbol("all"),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::All,
                    type_params: vec![],
                    arg_types: vec![range_t.clone(), pred_num_t.clone()],
                    result_type: bool_t.clone(),
                },
            );
            add_builtin(
                self,
                self.builtin_fun_symbol("any"),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::Any,
                    type_params: vec![],
                    arg_types: vec![range_t.clone(), pred_num_t.clone()],
                    result_type: bool_t.clone(),
                },
            );

            // Resources.
            add_builtin(
                self,
                self.builtin_fun_symbol("global"),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::Global,
                    type_params: vec![param_t.clone()],
                    arg_types: vec![address_t.clone()],
                    result_type: param_t.clone(),
                },
            );
            add_builtin(
                self,
                self.builtin_fun_symbol("exists"),
                SpecFunEntry {
                    loc: loc.clone(),
                    oper: Operation::Exists,
                    type_params: vec![param_t.clone()],
                    arg_types: vec![address_t.clone()],
                    result_type: bool_t.clone(),
                },
            );

            // Old
            add_builtin(
                self,
                self.builtin_fun_symbol("old"),
                SpecFunEntry {
                    loc,
                    oper: Operation::Old,
                    type_params: vec![param_t.clone()],
                    arg_types: vec![param_t.clone()],
                    result_type: param_t.clone(),
                },
            );
        }
    }

    /// Returns the qualified symbol for the binary operator.
    fn bin_op_symbol(&self, op: &PA::BinOp_) -> QualifiedSymbol {
        QualifiedSymbol {
            module_name: self.builtin_module(),
            symbol: self.env.symbol_pool().make(&op.symbol()),
        }
    }

    /// Returns the qualified symbol for the unary operator.
    fn unary_op_symbol(&self, op: &PA::UnaryOp_) -> QualifiedSymbol {
        QualifiedSymbol {
            module_name: self.builtin_module(),
            symbol: self.env.symbol_pool().make(&op.symbol()),
        }
    }

    /// Returns the qualified symbol for a builtin function.
    fn builtin_fun_symbol(&self, name: &str) -> QualifiedSymbol {
        QualifiedSymbol {
            module_name: self.builtin_module(),
            symbol: self.env.symbol_pool().make(name),
        }
    }

    /// Returns the symbol for the builtin function `old`.
    fn old_symbol(&self) -> Symbol {
        self.env.symbol_pool().make("old")
    }

    /// Returns the name for the pseudo builtin module.
    pub fn builtin_module(&self) -> ModuleName {
        ModuleName::new(BigUint::default(), self.env.symbol_pool().make("$$"))
    }
}

// =================================================================================================
/// # Module Translation

/// A module translator.
#[derive(Debug)]
pub struct ModuleTranslator<'env, 'translator> {
    parent: &'translator mut Translator<'env>,
    /// Counter for NodeId in this module.
    node_counter: usize,
    /// A map from node id to associated location.
    loc_map: BTreeMap<NodeId, Loc>,
    /// A map from node id to associated type.
    type_map: BTreeMap<NodeId, Type>,
    /// A map from node id to associated instantiation of type parameters.
    pub instantiation_map: BTreeMap<NodeId, Vec<Type>>,
    /// Id of the currently build module.
    module_id: ModuleId,
    /// Name of the currently build module.
    module_name: ModuleName,
    /// Translated specification functions.
    spec_funs: Vec<SpecFunDecl>,
    /// During the definition analysis, the index into `spec_funs` we are currently
    /// handling
    spec_fun_index: usize,
    /// Translated specification variables.
    spec_vars: Vec<SpecVarDecl>,
    /// Translated function specifications.
    fun_specs: BTreeMap<Symbol, Spec>,
    /// Translated struct specifications.
    struct_specs: BTreeMap<Symbol, Spec>,
    /// Translated module spec
    module_spec: Spec,
}

/// # Entry Points

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    pub fn new(
        parent: &'translator mut Translator<'env>,
        module_id: ModuleId,
        module_name: ModuleName,
    ) -> Self {
        Self {
            parent,
            node_counter: 0,
            loc_map: BTreeMap::new(),
            type_map: BTreeMap::new(),
            instantiation_map: BTreeMap::new(),
            module_id,
            module_name,
            spec_funs: vec![],
            spec_fun_index: 0,
            spec_vars: vec![],
            fun_specs: BTreeMap::new(),
            struct_specs: BTreeMap::new(),
            module_spec: Spec::default(),
        }
    }

    /// Translates the given module definition from the move compiler's expansion phase,
    /// combined with a compiled module (bytecode) and a source map, and enters it into
    /// this global environment. Any type check or others errors encountered will be collected
    /// in the environment for later processing. Dependencies of this module are guaranteed to
    /// have been analyzed and being already part of the environment.
    ///
    /// Translation happens in three phases:
    ///
    /// 1. In the *declaration analysis*, we collect all information about structs, functions,
    ///    spec functions, spec vars, and schemas in a module. We do not yet analyze function
    ///    bodies, conditions, and invariants, which we can only analyze after we know all
    ///    global declarations (declaration of globals is order independent, and they can have
    ///    cyclic references).
    /// 2. In the *definition analysis*, we visit the definitions we have skipped in step (1),
    ///    specifically analyzing and type checking expressions and schema inclusions.
    /// 3. In the *population phase*, we populate the global environment with the information
    ///    from this module.
    pub fn translate(
        &mut self,
        loc: Loc,
        module_def: EA::ModuleDefinition,
        compiled_module: CompiledModule,
        source_map: SourceMap<MoveIrLoc>,
        function_infos: UniqueMap<FunctionName, FunctionInfo>,
    ) {
        self.decl_ana(&module_def);
        self.def_ana(&module_def, function_infos);
        self.populate_env_from_result(loc, compiled_module, source_map);
    }
}

/// # Basic Helpers

/// A value which we pass in to spec block analyzers, describing the resolved target of the spec
/// block.
#[derive(Debug)]
enum SpecBlockContext<'a> {
    Module,
    Struct(QualifiedSymbol),
    Function(QualifiedSymbol),
    FunctionCode(QualifiedSymbol, &'a SpecInfo),
    Schema(QualifiedSymbol),
}

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    /// Shortcut for accessing the symbol pool.
    fn symbol_pool(&self) -> &SymbolPool {
        self.parent.env.symbol_pool()
    }

    /// Creates a new node id.
    fn new_node_id(&mut self) -> NodeId {
        let node_id = NodeId::new(self.node_counter);
        self.node_counter += 1;
        node_id
    }

    /// Creates a new node id and assigns type and location to it.
    fn new_node_id_with_type_loc(&mut self, ty: &Type, loc: &Loc) -> NodeId {
        let id = self.new_node_id();
        self.loc_map.insert(id, loc.clone());
        self.type_map.insert(id, ty.clone());
        id
    }

    /// Qualifies the given symbol by the current module.
    fn qualified_by_module(&self, sym: Symbol) -> QualifiedSymbol {
        QualifiedSymbol {
            module_name: self.module_name.clone(),
            symbol: sym,
        }
    }

    /// Qualifies the given name by the current module.
    fn qualified_by_module_from_name(&self, name: &Name) -> QualifiedSymbol {
        let sym = self.symbol_pool().make(&name.value);
        self.qualified_by_module(sym)
    }

    /// Converts a ModuleAccess into its parts, an optional ModuleName and base name.
    fn module_access_to_parts(&self, access: &EA::ModuleAccess) -> (Option<ModuleName>, Symbol) {
        match &access.value {
            EA::ModuleAccess_::Name(n) => (None, self.symbol_pool().make(n.value.as_str())),
            EA::ModuleAccess_::ModuleAccess(m, n) => {
                let module_name = ModuleName::from_str(
                    &m.0.value.address.to_string(),
                    self.symbol_pool().make(m.0.value.name.0.value.as_str()),
                );
                (Some(module_name), self.symbol_pool().make(n.value.as_str()))
            }
        }
    }

    /// Converts a ModuleAccess into a qualified symbol which can be used for lookup of
    /// types or functions.
    fn module_access_to_qualified(&self, access: &EA::ModuleAccess) -> QualifiedSymbol {
        let (module_name_opt, symbol) = self.module_access_to_parts(access);
        let module_name = module_name_opt.unwrap_or_else(|| self.module_name.clone());
        QualifiedSymbol {
            module_name,
            symbol,
        }
    }

    /// Creates a SpecBlockContext from the given SpecBlockTarget. The context is used during
    /// definition analysis when visiting a schema block member (condition, invariant, etc.).
    /// This returns None if the SpecBlockTarget cannnot be resolved; error reporting happens
    /// at caller side.
    fn get_spec_block_context<'pa>(
        &self,
        target: &'pa PA::SpecBlockTarget,
    ) -> Option<SpecBlockContext<'pa>> {
        match &target.value {
            PA::SpecBlockTarget_::Code => None,
            PA::SpecBlockTarget_::Function(name) => {
                let qsym = self.qualified_by_module_from_name(&name.0);
                if self.parent.fun_table.contains_key(&qsym) {
                    Some(SpecBlockContext::Function(qsym))
                } else {
                    None
                }
            }
            PA::SpecBlockTarget_::Structure(name) => {
                let qsym = self.qualified_by_module_from_name(&name.0);
                if self.parent.struct_table.contains_key(&qsym) {
                    Some(SpecBlockContext::Struct(qsym))
                } else {
                    None
                }
            }
            PA::SpecBlockTarget_::Schema(name, _) => {
                let qsym = self.qualified_by_module_from_name(&name);
                if self.parent.spec_schema_table.contains_key(&qsym) {
                    Some(SpecBlockContext::Schema(qsym))
                } else {
                    None
                }
            }
            PA::SpecBlockTarget_::Module => Some(SpecBlockContext::Module),
        }
    }
}

/// # Declaration Analysis

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    fn decl_ana(&mut self, module_def: &EA::ModuleDefinition) {
        for (name, struct_def) in &module_def.structs {
            self.decl_ana_struct(&name, struct_def);
        }
        for (name, fun_def) in &module_def.functions {
            self.decl_ana_fun(&name, fun_def);
        }
        for spec in &module_def.specs {
            self.decl_ana_spec_block(spec);
        }
    }

    fn decl_ana_struct(&mut self, name: &PA::StructName, def: &EA::StructDefinition) {
        let qsym = self.qualified_by_module_from_name(&name.0);
        let struct_id = StructId::new(qsym.symbol);
        let mut et = ExpTranslator::new(self);
        let type_params = et.analyze_and_add_type_params(&def.type_parameters);
        et.parent.parent.define_struct(
            et.to_loc(&def.loc),
            qsym,
            et.parent.module_id,
            struct_id,
            def.resource_opt.is_some(),
            type_params,
            None, // will be filled in during definition analysis
        );
    }

    fn decl_ana_fun(&mut self, name: &PA::FunctionName, def: &EA::Function) {
        let qsym = self.qualified_by_module_from_name(&name.0);
        let fun_id = FunId::new(qsym.symbol);
        let mut et = ExpTranslator::new(self);
        et.enter_scope();
        let type_params = et.analyze_and_add_type_params(&def.signature.type_parameters);
        et.enter_scope();
        let params = et.analyze_and_add_params(&def.signature.parameters);
        let result_type = et.translate_type(&def.signature.return_type);
        let is_public = matches!(def.visibility, PA::FunctionVisibility::Public(..));
        et.parent.parent.define_fun(
            et.to_loc(&def.loc),
            qsym,
            et.parent.module_id,
            fun_id,
            is_public,
            type_params,
            params,
            result_type,
        );
    }

    fn decl_ana_spec_block(&mut self, block: &EA::SpecBlock) {
        use EA::SpecBlockMember_::*;
        // Process any spec block members which introduce global declarations.
        for member in &block.value.members {
            let loc = self.parent.env.to_loc(&member.loc);
            match &member.value {
                Function {
                    name, signature, ..
                } => self.decl_ana_spec_fun(&loc, name, signature),
                Variable {
                    is_global: true,
                    name,
                    type_,
                    type_parameters,
                } => self.decl_ana_global_var(&loc, name, type_parameters, type_),
                _ => {}
            }
        }
        // If this is a schema spec block, process its declaration.
        if let PA::SpecBlockTarget_::Schema(name, type_params) = &block.value.target.value {
            self.decl_ana_schema(&block, &name, &type_params);
        }
    }

    fn decl_ana_spec_fun(
        &mut self,
        loc: &Loc,
        name: &PA::FunctionName,
        signature: &EA::FunctionSignature,
    ) {
        let name = self.symbol_pool().make(&name.0.value);
        let (type_params, params, result_type) = {
            let et = &mut ExpTranslator::new(self);
            let type_params = et.analyze_and_add_type_params(&signature.type_parameters);
            et.enter_scope();
            let params = et.analyze_and_add_params(&signature.parameters);
            let result_type = et.translate_type(&signature.return_type);
            et.finalize_types();
            (type_params, params, result_type)
        };

        // Add the function to the symbol table.
        let fun_id = SpecFunId::new(self.spec_funs.len());
        self.parent.define_spec_fun(
            loc,
            self.qualified_by_module(name),
            self.module_id,
            fun_id,
            type_params.iter().map(|(_, ty)| ty.clone()).collect(),
            params.iter().map(|(_, ty)| ty.clone()).collect(),
            result_type.clone(),
        );
        // Add a prototype of the SpecFunDecl to the module translator. This
        // will for now have an empty body which we fill in during a 2nd pass.
        let fun_decl = SpecFunDecl {
            loc: loc.clone(),
            name,
            type_params,
            params,
            result_type,
            used_spec_vars: BTreeSet::new(),
            is_pure: true,
            body: None,
        };
        self.spec_funs.push(fun_decl);
    }

    fn decl_ana_global_var(
        &mut self,
        loc: &Loc,
        name: &Name,
        type_params: &[(Name, PA::Kind)],
        type_: &EA::Type,
    ) {
        let name = self.symbol_pool().make(name.value.as_str());
        let (type_params, type_) = {
            let et = &mut ExpTranslator::new(self);
            let type_params = et.analyze_and_add_type_params(type_params);
            let type_ = et.translate_type(type_);
            (type_params, type_)
        };
        if type_.is_reference() {
            self.parent.error(
                loc,
                &format!(
                    "`{}` cannot have reference type",
                    name.display(self.symbol_pool())
                ),
            )
        }
        // Add the variable to the symbol table.
        let var_id = SpecVarId::new(self.spec_vars.len());
        self.parent.define_spec_var(
            loc,
            self.qualified_by_module(name),
            self.module_id,
            var_id,
            project_2nd(&type_params),
            type_.clone(),
        );
        // Add the variable to the module translator.
        let var_decl = SpecVarDecl {
            loc: loc.clone(),
            name,
            type_params,
            type_,
        };
        self.spec_vars.push(var_decl);
    }

    fn decl_ana_schema(
        &mut self,
        block: &EA::SpecBlock,
        name: &Name,
        type_params: &[(Name, PA::Kind)],
    ) {
        let qsym = self.qualified_by_module_from_name(name);
        let mut et = ExpTranslator::new(self);
        et.enter_scope();
        let type_params = et.analyze_and_add_type_params(&type_params);
        // Extract local variables.
        let mut vars = vec![];
        for member in &block.value.members {
            if let EA::SpecBlockMember_::Variable {
                is_global: false,
                name,
                type_,
                type_parameters,
            } = &member.value
            {
                if !type_parameters.is_empty() {
                    et.error(
                        &et.to_loc(&member.loc),
                        "schema variable cannot have type parameters",
                    );
                }
                let name = et.symbol_pool().make(&name.value);
                let type_ = et.translate_type(type_);
                vars.push((name, type_));
            }
        }
        // Add schema declaration prototype to the symbol table.
        let loc = et.to_loc(&block.loc);
        self.parent
            .define_spec_schema(&loc, qsym, self.module_id, type_params, vars);
    }
}

/// # Definition Analysis

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    fn def_ana(
        &mut self,
        module_def: &EA::ModuleDefinition,
        function_infos: UniqueMap<FunctionName, FunctionInfo>,
    ) {
        // Analyze all structs.
        for (name, def) in &module_def.structs {
            self.def_ana_struct(&name, def);
        }

        // Analyze all schemas. This must be done before other things because schemas need to be
        // ready for inclusion. We also must do this recursively, so use a visited set to detect
        // cycles.
        {
            let schema_defs: BTreeMap<QualifiedSymbol, &EA::SpecBlock> = module_def
                .specs
                .iter()
                .filter_map(|block| {
                    if let PA::SpecBlockTarget_::Schema(name, ..) = &block.value.target.value {
                        let qsym = self.qualified_by_module_from_name(name);
                        Some((qsym, block))
                    } else {
                        None
                    }
                })
                .collect();
            let mut visited = BTreeSet::new();
            let mut visiting = vec![];
            for (name, block) in schema_defs.iter() {
                self.def_ana_schema(
                    &schema_defs,
                    &mut visited,
                    &mut visiting,
                    name.clone(),
                    block,
                );
            }
        }

        // Analyze all module level spec blocks (except schemas)
        for spec in &module_def.specs {
            if matches!(spec.value.target.value, PA::SpecBlockTarget_::Schema(..)) {
                continue;
            }
            match self.get_spec_block_context(&spec.value.target) {
                Some(context) => self.def_ana_spec_block(&context, spec),
                None => {
                    let loc = self.parent.env.to_loc(&spec.value.target.loc);
                    self.parent.error(&loc, "unresolved spec target");
                }
            }
        }

        // Analyze in-function spec blocks.
        for (name, fun_def) in &module_def.functions {
            let fun_spec_info = &function_infos.get(&name).unwrap().spec_info;
            let qsym = self.qualified_by_module_from_name(&name.0);
            for (spec_id, spec_block) in fun_def.specs.iter() {
                for member in &spec_block.value.members {
                    let loc = &self.parent.env.to_loc(&member.loc);
                    match &member.value {
                        EA::SpecBlockMember_::Condition { kind, exp } => {
                            let context = SpecBlockContext::FunctionCode(
                                qsym.clone(),
                                &fun_spec_info[spec_id],
                            );
                            if let Some((kind, exp)) =
                                self.extract_condition_kind(&context, kind, exp)
                            {
                                self.def_ana_condition(loc, &context, kind, exp);
                            }
                        }
                        _ => {
                            self.parent.error(&loc, "item not allowed");
                        }
                    }
                }
            }
        }

        // Perform post analyzes of spec var usage in spec functions.
        self.compute_spec_var_usage();

        // Perform post reduction of module invariants.
        self.reduce_module_invariants();
    }
}

/// ## Struct Definition Analysis

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    fn def_ana_struct(&mut self, name: &PA::StructName, def: &EA::StructDefinition) {
        let qsym = self.qualified_by_module_from_name(&name.0);
        let type_params = self
            .parent
            .struct_table
            .get(&qsym)
            .expect("struct invalid")
            .type_params
            .clone();
        let mut et = ExpTranslator::new(self);
        let loc = et.to_loc(&name.0.loc);
        for (name, ty) in type_params {
            et.define_type_param(&loc, name, ty);
        }
        let fields = match &def.fields {
            EA::StructFields::Defined(fields) => {
                let mut field_map = BTreeMap::new();
                for (ref field_name, (idx, ty)) in fields.iter() {
                    let field_sym = et.symbol_pool().make(&field_name.0.value);
                    let field_ty = et.translate_type(&ty);
                    field_map.insert(field_sym, (*idx, field_ty));
                }
                Some(field_map)
            }
            EA::StructFields::Native(_) => None,
        };
        self.parent
            .struct_table
            .get_mut(&qsym)
            .expect("struct invalid")
            .fields = fields;
    }
}

/// ## Spec Block Definition Analysis

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    fn def_ana_spec_block(&mut self, context: &SpecBlockContext<'_>, block: &EA::SpecBlock) {
        use EA::SpecBlockMember_::*;
        for member in &block.value.members {
            let loc = &self.parent.env.to_loc(&member.loc);
            match &member.value {
                Condition { kind, exp } => {
                    if let Some((kind, exp)) = self.extract_condition_kind(context, kind, exp) {
                        self.def_ana_condition(loc, context, kind, exp)
                    }
                }
                Function {
                    signature, body, ..
                } => self.def_ana_spec_fun(signature, body),
                Include { exp } => {
                    self.def_ana_schema_inclusion_outside_schema(loc, context, None, exp)
                }
                Apply {
                    exp,
                    patterns,
                    exclusion_patterns,
                } => self.def_ana_schema_apply(loc, context, exp, patterns, exclusion_patterns),
                Pragma { properties } => self.def_ana_pragma(loc, context, properties),
                Variable { .. } => { /* nothing to do right now */ }
            }
        }
    }
}

/// ## Pragma Definition Analysis

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    /// Definition analysis for a pragma.
    fn def_ana_pragma(
        &mut self,
        _loc: &Loc,
        context: &SpecBlockContext,
        properties: &[PA::PragmaProperty],
    ) {
        // For now we pass properties just on. We may want to check against a set of known
        // property names and types.
        for prop in properties {
            let prop_name = self.symbol_pool().make(&prop.value.name.value);
            let value = if let Some(pv) = &prop.value.value {
                let mut et = ExpTranslator::new(self);
                if let Some((v, _)) = et.translate_value(pv) {
                    v
                } else {
                    // Error reported
                    continue;
                }
            } else {
                Value::Bool(true)
            };
            self.update_spec(context, move |spec| {
                spec.properties.insert(prop_name, value);
            });
        }
    }
}

/// ## General Helpers for Definition Analysis

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    /// Updates the Spec of a given context via an update function.
    fn update_spec<F>(&mut self, context: &SpecBlockContext, update: F)
    where
        F: FnOnce(&mut Spec),
    {
        use SpecBlockContext::*;
        match context {
            Function(name) => update(
                &mut self
                    .fun_specs
                    .entry(name.symbol)
                    .or_insert_with(Spec::default),
            ),
            FunctionCode(name, spec_info) => update(
                &mut self
                    .fun_specs
                    .entry(name.symbol)
                    .or_insert_with(Spec::default)
                    .on_impl
                    .entry(spec_info.offset)
                    .or_insert_with(Spec::default),
            ),
            Schema(name) => update(
                &mut self
                    .parent
                    .spec_schema_table
                    .get_mut(name)
                    .expect("schema defined")
                    .spec,
            ),
            Struct(name) => update(
                &mut self
                    .struct_specs
                    .entry(name.symbol)
                    .or_insert_with(Spec::default),
            ),
            Module => update(&mut self.module_spec),
        }
    }

    /// Sets up an expression translator for the given spec block context. If kind
    /// is given, includes all the symbols which can be consumed by the condition,
    /// otherwise only defines type parameters.
    fn exp_translator_for_context<'module_translator>(
        &'module_translator mut self,
        loc: &Loc,
        context: &SpecBlockContext,
        kind_opt: Option<&ConditionKind>,
    ) -> ExpTranslator<'env, 'translator, 'module_translator> {
        use SpecBlockContext::*;
        let allow_old = if let Some(kind) = kind_opt {
            kind.allows_old()
        } else {
            false
        };
        match context {
            Function(name) => {
                let entry = &self
                    .parent
                    .fun_table
                    .get(name)
                    .expect("invalid spec block context")
                    .clone();
                let mut et = ExpTranslator::new_with_old(self, allow_old);
                for (n, ty) in &entry.type_params {
                    et.define_type_param(loc, *n, ty.clone());
                }
                if let Some(kind) = kind_opt {
                    et.enter_scope();
                    for (n, ty) in &entry.params {
                        et.define_local(loc, *n, ty.clone(), None);
                    }
                    // Define the placeholders for the result values of a function if this is an
                    // Ensures condition.
                    if matches!(kind, ConditionKind::Ensures) {
                        et.enter_scope();
                        if let Type::Tuple(ts) = &entry.result_type {
                            for (i, ty) in ts.iter().enumerate() {
                                let name = et.symbol_pool().make(&format!("result_{}", i + 1));
                                et.define_local(loc, name, ty.clone(), Some(Operation::Result(i)));
                            }
                        } else {
                            let name = et.symbol_pool().make("result");
                            et.define_local(
                                loc,
                                name,
                                entry.result_type.clone(),
                                Some(Operation::Result(0)),
                            );
                        }
                    }
                }
                et
            }
            FunctionCode(name, spec_info) => {
                let entry = &self
                    .parent
                    .fun_table
                    .get(name)
                    .expect("invalid spec block context")
                    .clone();
                let mut et = ExpTranslator::new_with_old(self, allow_old);
                for (n, ty) in &entry.type_params {
                    et.define_type_param(loc, *n, ty.clone());
                }
                if kind_opt.is_some() {
                    et.enter_scope();
                    for (n, info) in &spec_info.used_locals {
                        let sym = et.symbol_pool().make(n.0.value.as_str());
                        let ty = et.translate_hlir_single_type(&info.type_);
                        if ty == Type::Error {
                            et.error(
                                loc,
                                "[internal] error in translating hlir type to prover type",
                            );
                        }
                        et.define_local(loc, sym, ty, Some(Operation::Local(sym)));
                    }
                }
                et
            }
            Struct(name) => {
                let entry = &self
                    .parent
                    .struct_table
                    .get(name)
                    .expect("invalid spec block context")
                    .clone();

                let mut et = ExpTranslator::new_with_old(self, allow_old);
                for (n, ty) in &entry.type_params {
                    et.define_type_param(loc, *n, ty.clone());
                }
                if kind_opt.is_some() {
                    if let Some(fields) = &entry.fields {
                        et.enter_scope();
                        for (n, (_, ty)) in fields {
                            et.define_local(
                                loc,
                                *n,
                                ty.clone(),
                                Some(Operation::Select(
                                    entry.module_id,
                                    entry.struct_id,
                                    FieldId::new(*n),
                                )),
                            );
                        }
                    }
                }
                et
            }
            Module => ExpTranslator::new(self),
            Schema(name) => {
                let entry = self
                    .parent
                    .spec_schema_table
                    .get(name)
                    .expect("schema defined");
                // Unfortunately need to clone elements from the entry because we need mut borrow
                // of self for expression translator.
                let type_params = entry.type_params.clone();
                let all_vars = entry.all_vars.clone();
                let mut et = ExpTranslator::new_with_old(self, allow_old);
                for (n, ty) in type_params {
                    et.define_type_param(loc, n, ty);
                }
                if kind_opt.is_some() {
                    et.enter_scope();
                    for (n, entry) in all_vars {
                        et.define_local(loc, n, entry.type_, None);
                    }
                }
                et
            }
        }
    }
}

/// ## Condition Definition Analysis

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    /// Check whether the condition is allowed in the given context. Return true if so, otherwise
    /// report an error and return false.
    fn check_condition_is_valid(
        &mut self,
        context: &SpecBlockContext,
        loc: &Loc,
        kind: &ConditionKind,
        error_msg: &str,
    ) -> bool {
        use SpecBlockContext::*;
        let ok = match context {
            Module => kind.allowed_on_module(),
            Struct(_) => kind.allowed_on_struct(),
            Function(name) => {
                let entry = self.parent.fun_table.get(name).expect("function defined");
                if entry.is_public {
                    kind.allowed_on_public_fun_decl()
                } else {
                    kind.allowed_on_private_fun_decl()
                }
            }
            FunctionCode(_, _) => kind.allowed_on_fun_impl(),
            Schema(_) => true,
        };
        if !ok {
            self.parent.error(loc, &format!("`{}` {}", kind, error_msg));
        }
        ok
    }

    /// Add the given conditions to the context, after checking whether they are valid in the
    /// context. Reports errors for invalid conditions.
    fn add_conditions_to_context(
        &mut self,
        context: &SpecBlockContext,
        loc: &Loc,
        conditions: Vec<Condition>,
        error_msg: &str,
    ) {
        for cond in conditions {
            // If this is an invariant on a function decl, transform it into a pair of
            // requires and ensures.
            let derived_conds = match cond {
                Condition {
                    loc,
                    kind: kind @ ConditionKind::Invariant,
                    exp,
                }
                | Condition {
                    loc,
                    kind: kind @ ConditionKind::InvariantModule,
                    exp,
                } if matches!(context, SpecBlockContext::Function(..)) => {
                    let requires_kind = if kind == ConditionKind::InvariantModule {
                        ConditionKind::RequiresModule
                    } else {
                        ConditionKind::Requires
                    };
                    vec![
                        Condition {
                            loc: loc.clone(),
                            kind: requires_kind,
                            exp: exp.clone(),
                        },
                        Condition {
                            loc,
                            kind: ConditionKind::Ensures,
                            exp,
                        },
                    ]
                }
                _ => vec![cond],
            };
            for derived_cond in derived_conds {
                if self.check_condition_is_valid(context, loc, &derived_cond.kind, error_msg) {
                    self.update_spec(context, |spec| spec.conditions.push(derived_cond));
                }
            }
        }
    }

    /// Definition analysis for a condition.
    fn def_ana_condition(
        &mut self,
        loc: &Loc,
        context: &SpecBlockContext,
        kind: ConditionKind,
        exp: &EA::Exp,
    ) {
        if kind == ConditionKind::Decreases {
            self.parent
                .error(loc, "decreases specification not supported currently");
            return;
        }
        let expected_type = self.expected_type_for_condition(&kind);
        let mut et = self.exp_translator_for_context(loc, context, Some(&kind));
        let translated = et.translate_exp(exp, &expected_type);
        et.finalize_types();
        self.add_conditions_to_context(
            context,
            loc,
            vec![Condition {
                loc: loc.clone(),
                kind,
                exp: translated,
            }],
            "not allowed in this context",
        );
    }

    /// Compute the expected type for the expression in a condition.
    fn expected_type_for_condition(&mut self, kind: &ConditionKind) -> Type {
        if let Some((mid, vid, ty_args)) = kind.get_spec_var_target() {
            if mid == self.module_id {
                self.spec_vars[vid.as_usize()]
                    .type_
                    .clone()
                    .instantiate(&ty_args)
            } else {
                let module_env = self.parent.env.get_module(mid);
                module_env
                    .get_spec_var(vid)
                    .type_
                    .clone()
                    .instantiate(&ty_args)
            }
        } else {
            BOOL_TYPE.clone()
        }
    }

    /// Extracts a condition kind based on the parsed kind and the associated expression. This
    /// identifies a spec var assignment expression and moves the var to the SpecConditionKind enum
    /// we use in our AST, returning the rhs expression of the assignment; otherwise it returns
    /// the passed expression.
    fn extract_condition_kind<'a>(
        &mut self,
        context: &SpecBlockContext,
        kind: &PA::SpecConditionKind,
        exp: &'a EA::Exp,
    ) -> Option<(ConditionKind, &'a EA::Exp)> {
        use ConditionKind::*;
        use PA::SpecConditionKind as PK;
        let loc = self.parent.env.to_loc(&exp.loc);
        match kind {
            PK::Assert => Some((Assert, exp)),
            PK::Assume => Some((Assume, exp)),
            PK::Decreases => Some((Decreases, exp)),
            PK::Ensures => Some((Ensures, exp)),
            PK::Requires => Some((Requires, exp)),
            PK::AbortsIf => Some((AbortsIf, exp)),
            PK::RequiresModule => Some((RequiresModule, exp)),
            PK::Invariant => Some((Invariant, exp)),
            PK::InvariantModule => Some((InvariantModule, exp)),
            PK::InvariantUpdate => {
                if let Some((mid, vid, tys, exp1)) = self.extract_assignment(context, exp) {
                    Some((VarUpdate(mid, vid, tys), exp1))
                } else {
                    Some((InvariantUpdate, exp))
                }
            }
            PK::InvariantPack => {
                if let Some((mid, vid, tys, exp1)) = self.extract_assignment(context, exp) {
                    Some((VarPack(mid, vid, tys), exp1))
                } else {
                    self.parent
                        .error(&loc, "expected assignment to spec variable");
                    None
                }
            }
            PK::InvariantUnpack => {
                if let Some((mid, vid, tys, exp1)) = self.extract_assignment(context, exp) {
                    Some((VarUnpack(mid, vid, tys), exp1))
                } else {
                    self.parent
                        .error(&loc, "expected assignment to spec variable");
                    None
                }
            }
        }
    }

    /// Extracts an assignment from an expression, returning the assigned spec var and
    /// rhs expression.
    fn extract_assignment<'a>(
        &mut self,
        context: &SpecBlockContext,
        exp: &'a EA::Exp,
    ) -> Option<(ModuleId, SpecVarId, Vec<Type>, &'a EA::Exp)> {
        if let EA::Exp_::Assign(list, rhs) = &exp.value {
            let var_loc = self.parent.to_loc(&list.loc);
            if list.value.len() != 1 {
                self.parent.error(
                    &var_loc,
                    "[current restriction] tuples not supported in assignment",
                );
                return None;
            }
            if let EA::LValue_::Var(maccess, tys_opt) = &list.value[0].value {
                let var_name = self.module_access_to_qualified(maccess);
                let mut et = self.exp_translator_for_context(&var_loc, context, None);
                let tys = tys_opt
                    .as_ref()
                    .map(|tys| et.translate_types(tys))
                    .unwrap_or_else(|| vec![]);
                if let Some(spec_var) = et.parent.parent.spec_var_table.get(&var_name) {
                    Some((spec_var.module_id, spec_var.var_id, tys, rhs.as_ref()))
                } else {
                    et.error(
                        &var_loc,
                        &format!(
                            "spec global `{}` undeclared",
                            var_name.display(et.symbol_pool())
                        ),
                    );
                    None
                }
            } else {
                self.parent.error(
                    &var_loc,
                    "[current restriction] unpack not supported in assignment",
                );
                None
            }
        } else {
            None
        }
    }
}

/// ## Spec Function Definition Analysis

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    /// Definition analysis for a specification helper function.
    fn def_ana_spec_fun(&mut self, _signature: &EA::FunctionSignature, body: &EA::FunctionBody) {
        if let EA::FunctionBody_::Defined(seq) = &body.value {
            let entry = &self.spec_funs[self.spec_fun_index];
            let type_params = entry.type_params.clone();
            let params = entry.params.clone();
            let result_type = entry.result_type.clone();
            let mut et = ExpTranslator::new(self);
            let loc = et.to_loc(&body.loc);
            for (n, ty) in type_params {
                et.define_type_param(&loc, n, ty);
            }
            et.enter_scope();
            for (n, ty) in params {
                et.define_local(&loc, n, ty, None);
            }
            let translated = et.translate_seq(&loc, seq, &result_type);
            et.finalize_types();
            self.spec_funs[self.spec_fun_index].body = Some(translated);
        }
        self.spec_fun_index += 1;
    }
}

/// ## Schema Definition Analysis

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    /// Definition analysis for a schema. This proceeds in two steps: first we ensure recursively
    /// that all included schemas are analyzed, checking for cycles. Then we actually analyze this
    /// schema's content.
    fn def_ana_schema(
        &mut self,
        schema_defs: &BTreeMap<QualifiedSymbol, &EA::SpecBlock>,
        visited: &mut BTreeSet<QualifiedSymbol>,
        visiting: &mut Vec<QualifiedSymbol>,
        name: QualifiedSymbol,
        block: &EA::SpecBlock,
    ) {
        if !visited.insert(name.clone()) {
            // Already analyzed.
            return;
        }
        visiting.push(name.clone());

        // First recursively visit all schema includes and ensure they are analyzed.
        for included_name in self
            .iter_schema_includes(&block.value.members)
            .map(|(_, exp)| {
                let mut res = vec![];
                extract_schema_access(exp, &mut res);
                res
            })
            .flatten()
        {
            let included_loc = self.parent.env.to_loc(&included_name.loc);
            let included_name = self.module_access_to_qualified(included_name);
            if included_name.module_name == self.module_name {
                // A schema in the module we are currently analyzing. We need to check
                // for cycles before recursively analyzing it.
                if visiting.contains(&included_name) {
                    self.parent.error(
                        &included_loc,
                        &format!(
                            "cyclic schema dependency: {} -> {}",
                            visiting
                                .iter()
                                .map(|name| format!("{}", name.display_simple(self.symbol_pool())))
                                .join(" -> "),
                            included_name.display_simple(self.symbol_pool())
                        ),
                    )
                } else if let Some(included_block) = schema_defs.get(&included_name) {
                    // Recursively analyze it, if its defined. If not, we report an undeclared
                    // error in 2nd phase.
                    self.def_ana_schema(
                        schema_defs,
                        visited,
                        visiting,
                        included_name,
                        included_block,
                    );
                }
            }
        }

        // Now actually analyze this schema.
        self.def_ana_schema_content(name, block);

        // Remove from visiting list
        visiting.pop();
    }

    /// Analysis of schema after it is ensured that all included schemas are fully analyzed.
    fn def_ana_schema_content(&mut self, name: QualifiedSymbol, block: &EA::SpecBlock) {
        let loc = self.parent.env.to_loc(&block.loc);
        let entry = self
            .parent
            .spec_schema_table
            .get(&name)
            .expect("schema defined");

        // Process all schema includes. We need to do this before we type check expressions to have
        // all variables from includes in the environment.
        let type_params = entry.type_params.clone();
        let mut all_vars: BTreeMap<Symbol, LocalVarEntry> = entry
            .vars
            .iter()
            .map(|(n, ty)| {
                (
                    *n,
                    LocalVarEntry {
                        loc: loc.clone(),
                        type_: ty.clone(),
                        operation: None,
                    },
                )
            })
            .collect();
        let mut included_spec = Spec::default();
        for (_, included_exp) in self.iter_schema_includes(&block.value.members) {
            self.def_ana_schema_exp(
                &type_params,
                &mut all_vars,
                &mut included_spec,
                true,
                included_exp,
            );
        }
        // Store the results back to the schema entry.
        {
            let entry = self
                .parent
                .spec_schema_table
                .get_mut(&name)
                .expect("schema defined");
            entry.all_vars = all_vars;
            entry.included_spec = included_spec;
        }

        // Now process all conditions and invariants.
        for member in &block.value.members {
            let member_loc = self.parent.to_loc(&member.loc);
            match &member.value {
                EA::SpecBlockMember_::Variable {
                    is_global: false, ..
                } => { /* handled during decl analysis */ }
                EA::SpecBlockMember_::Include { .. } => { /* handled above */ }
                EA::SpecBlockMember_::Condition { kind, exp } => {
                    let context = SpecBlockContext::Schema(name.clone());
                    if let Some((kind, exp)) = self.extract_condition_kind(&context, kind, exp) {
                        self.def_ana_condition(&member_loc, &context, kind, exp);
                    } else {
                        // Error reported.
                    }
                }
                _ => {
                    self.parent.error(&member_loc, "item not allowed in schema");
                }
            };
        }
    }

    /// Extracts all schema inclusions from a list of spec block members.
    fn iter_schema_includes<'a>(
        &self,
        members: &'a [EA::SpecBlockMember],
    ) -> impl Iterator<Item = (&'a MoveIrLoc, &'a EA::Exp)> {
        members.iter().filter_map(|m| {
            if let EA::SpecBlockMember_::Include { exp } = &m.value {
                Some((&m.loc, exp))
            } else {
                None
            }
        })
    }

    /// Analyzes a schema expression. Depending on whether `allow_new_vars` is true, this will
    /// add new variables to `vars` and match types of existing ones. All conditions
    /// from the schema are rewritten for the inclusion context and added to the provided spec.
    ///
    /// We accept a very restricted set of Move expressions for schemas:
    ///
    /// - `P ==> SchemaExp`: all conditions in the schema will be prefixed with `P ==> ..`.
    ///   Conditions which are not based on boolean expressions (as VarUpdate et. al) will
    ///   be rejected.
    /// - `if (P) SchemaExp else SchemaExp`: this is treated similar as one include for
    ///   `P ==> SchemaExp` and one for `!P ==> SchemaExp`.
    /// - `SchemaExp1 && SchemaExp2`: this is treated as two includes for the both expressions.
    /// - `SchemaExp1 || SchemaExp2`: this could be treated as
    ///   `exists b: bool :: if (b) SchemaExp1 else SchemaExp2` (but as we do not have the
    ///   existential quantifier yet in the spec language, it is actually not supported..)
    ///
    /// The implementation works via a recursive function which accumulates a path condition
    /// leading to a Move "pack" expression which is interpreted as a schema reference.
    fn def_ana_schema_exp(
        &mut self,
        context_type_params: &[(Symbol, Type)],
        vars: &mut BTreeMap<Symbol, LocalVarEntry>,
        spec: &mut Spec,
        allow_new_vars: bool,
        exp: &EA::Exp,
    ) {
        self.def_ana_schema_exp_oper(context_type_params, vars, spec, allow_new_vars, None, exp)
    }

    /// Analyzes operations in schema expressions. This extends the path condition as needed
    /// and continues recursively.
    fn def_ana_schema_exp_oper(
        &mut self,
        context_type_params: &[(Symbol, Type)],
        vars: &mut BTreeMap<Symbol, LocalVarEntry>,
        spec: &mut Spec,
        allow_new_vars: bool,
        path_cond: Option<Exp>,
        exp: &EA::Exp,
    ) {
        let loc = self.parent.to_loc(&exp.loc);
        match &exp.value {
            EA::Exp_::BinopExp(
                lhs,
                Spanned {
                    value: BinOp_::Implies,
                    ..
                },
                rhs,
            ) => {
                let lhs_exp = self
                    .exp_translator_for_schema(&loc, context_type_params, vars)
                    .translate_exp(lhs, &BOOL_TYPE);
                let path_cond = self.extend_path_condition(&loc, path_cond, lhs_exp);
                self.def_ana_schema_exp_oper(
                    context_type_params,
                    vars,
                    spec,
                    allow_new_vars,
                    path_cond,
                    rhs,
                );
            }
            EA::Exp_::BinopExp(
                lhs,
                Spanned {
                    value: BinOp_::And, ..
                },
                rhs,
            ) => {
                self.def_ana_schema_exp_oper(
                    context_type_params,
                    vars,
                    spec,
                    allow_new_vars,
                    path_cond.clone(),
                    lhs,
                );
                self.def_ana_schema_exp_oper(
                    context_type_params,
                    vars,
                    spec,
                    allow_new_vars,
                    path_cond,
                    rhs,
                );
            }
            EA::Exp_::IfElse(c, t, e) => {
                let c_exp = self
                    .exp_translator_for_schema(&loc, context_type_params, vars)
                    .translate_exp(c, &BOOL_TYPE);
                let t_path_cond =
                    self.extend_path_condition(&loc, path_cond.clone(), c_exp.clone());
                self.def_ana_schema_exp_oper(
                    context_type_params,
                    vars,
                    spec,
                    allow_new_vars,
                    t_path_cond,
                    t,
                );
                let node_id = self.new_node_id_with_type_loc(&BOOL_TYPE, &loc);
                let not_c_exp = Exp::Call(node_id, Operation::Not, vec![c_exp]);
                let e_path_cond = self.extend_path_condition(&loc, path_cond, not_c_exp);
                self.def_ana_schema_exp_oper(
                    context_type_params,
                    vars,
                    spec,
                    allow_new_vars,
                    e_path_cond,
                    e,
                );
            }
            EA::Exp_::Name(maccess, type_args_opt) => self.def_ana_schema_exp_leaf(
                context_type_params,
                vars,
                spec,
                allow_new_vars,
                path_cond,
                &loc,
                maccess,
                type_args_opt,
                None,
            ),
            EA::Exp_::Pack(maccess, type_args_opt, fields) => self.def_ana_schema_exp_leaf(
                context_type_params,
                vars,
                spec,
                allow_new_vars,
                path_cond,
                &loc,
                maccess,
                type_args_opt,
                Some(fields),
            ),
            _ => self
                .parent
                .error(&loc, "expression construct not supported for schemas"),
        }
    }

    /// Analyzes a schema leaf expression.
    fn def_ana_schema_exp_leaf(
        &mut self,
        context_type_params: &[(Symbol, Type)],
        vars: &mut BTreeMap<Symbol, LocalVarEntry>,
        spec: &mut Spec,
        allow_new_vars: bool,
        path_cond: Option<Exp>,
        loc: &Loc,
        maccess: &EA::ModuleAccess,
        type_args_opt: &Option<Vec<EA::Type>>,
        args_opt: Option<&EA::Fields<EA::Exp>>,
    ) {
        let schema_name = self.module_access_to_qualified(maccess);

        // We need to temporarily detach the schema entry from the parent table because of
        // borrowing problems, as we need to traverse it while at the same time mutate self.
        let schema_entry = if let Some(e) = self.parent.spec_schema_table.remove(&schema_name) {
            e
        } else {
            self.parent.error(
                loc,
                &format!(
                    "schema `{}` undeclared",
                    schema_name.display(self.symbol_pool())
                ),
            );
            return;
        };

        // Translate type arguments
        let mut et = self.exp_translator_for_schema(&loc, context_type_params, vars);
        let type_arguments = &et.translate_types_opt(type_args_opt);
        if schema_entry.type_params.len() != type_arguments.len() {
            self.parent.error(
                loc,
                &format!(
                    "wrong number of type arguments (expected {}, got {})",
                    schema_entry.type_params.len(),
                    type_arguments.len()
                ),
            );
            // Don't forget to put schema back.
            self.parent
                .spec_schema_table
                .insert(schema_name, schema_entry);
            return;
        }

        // Translate schema arguments.
        let mut argument_map: BTreeMap<Symbol, Exp> = args_opt
            .map(|args| {
                args.iter()
                    .map(|(ref schema_var, (_, exp))| {
                        let pool = et.symbol_pool();
                        let schema_sym = pool.make(&schema_var.0.value);
                        let schema_type = if let Some(LocalVarEntry { type_, .. }) =
                            schema_entry.all_vars.get(&schema_sym)
                        {
                            type_.instantiate(type_arguments)
                        } else {
                            et.error(
                                &et.to_loc(&schema_var.0.loc),
                                &format!("`{}` not declared in schema", schema_sym.display(pool)),
                            );
                            Type::Error
                        };
                        // Check the expression in the argument list.
                        // Note we currently only use the vars defined so far in this context. Variables
                        // which are introduced by schemas after the inclusion of this one are not in scope.
                        let exp = et.translate_exp(exp, &schema_type);
                        et.finalize_types();
                        (schema_sym, exp)
                    })
                    .collect()
            })
            .unwrap_or_else(BTreeMap::new);

        // Go over all variables in the schema which are not in the argument map and either match
        // them against existing one or declare new, if allowed.
        for (name, LocalVarEntry { type_, .. }) in &schema_entry.all_vars {
            if argument_map.contains_key(name) {
                continue;
            }
            let ty = type_.instantiate(type_arguments);
            let pool = et.symbol_pool();
            if let Some(entry) = vars.get(name) {
                // Name already exists in inclusion context, check its type.
                et.check_type(
                    loc,
                    &ty,
                    &entry.type_,
                    &format!(
                        "for `{}` included from schema",
                        name.display(et.symbol_pool())
                    ),
                );
                // Put into argument map.
                let node_id = et.new_node_id_with_type_loc(&entry.type_, loc);
                let exp = if let Some(oper) = &entry.operation {
                    Exp::Call(node_id, oper.clone(), vec![])
                } else {
                    Exp::LocalVar(node_id, *name)
                };
                argument_map.insert(*name, exp);
            } else if allow_new_vars {
                // Name does not yet exists in inclusion context, but is allowed to be introduced.
                // This happens if we include a schema in another schema.
                vars.insert(
                    *name,
                    LocalVarEntry {
                        loc: loc.clone(),
                        type_: ty.clone(),
                        operation: None,
                    },
                );
            } else {
                et.error(
                    loc,
                    &format!(
                        "`{}` cannot be matched to an existing name in inclusion context",
                        name.display(pool)
                    ),
                );
            }
        }
        // Done with expression translator; ensure all types are inferred correctly.
        et.finalize_types();

        // Go over all conditions in the schema, rewrite them, and add to the inclusion conditions.
        for Condition { loc, kind, exp } in schema_entry
            .spec
            .conditions
            .iter()
            .chain(schema_entry.included_spec.conditions.iter())
        {
            let mut rewriter =
                ExpRewriter::new(self, schema_entry.module_id, &argument_map, type_arguments);
            let mut exp = rewriter.rewrite(exp);
            if let Some(cond) = &path_cond {
                // There is a path condition to be added. This is only possible for proper
                // boolean conditions.
                if kind.get_spec_var_target().is_some() {
                    self.parent
                        .error(loc, &format!("`{}` cannot be included conditionally", kind));
                } else {
                    let path_cond_loc = self.loc_map.get(&cond.node_id()).unwrap_or(loc).clone();
                    let node_id = self.new_node_id_with_type_loc(&BOOL_TYPE, &path_cond_loc);
                    // In case of AbortsIf, the path condition is combined with the predicate using
                    // &&, otherwise ==>.
                    exp = Exp::Call(
                        node_id,
                        if kind == &ConditionKind::AbortsIf {
                            Operation::And
                        } else {
                            Operation::Implies
                        },
                        vec![cond.clone(), exp],
                    );
                }
            }
            spec.conditions.push(Condition {
                loc: loc.clone(),
                kind: kind.clone(),
                exp,
            });
        }

        // Put schema entry back.
        self.parent
            .spec_schema_table
            .insert(schema_name, schema_entry);
    }

    /// Creates an expression translator for use in schema expression. This defines the context
    /// type parameters and the variables.
    fn exp_translator_for_schema<'module_translator>(
        &'module_translator mut self,
        loc: &Loc,
        context_type_params: &[(Symbol, Type)],
        vars: &mut BTreeMap<Symbol, LocalVarEntry>,
    ) -> ExpTranslator<'env, 'translator, 'module_translator> {
        let mut et = ExpTranslator::new(self);
        for (n, ty) in context_type_params {
            et.define_type_param(loc, *n, ty.clone())
        }
        et.enter_scope();
        for (n, entry) in vars.iter() {
            et.define_local(&entry.loc, *n, entry.type_.clone(), entry.operation.clone());
        }
        et
    }

    /// Extends a path condition for schema expression analysis.
    fn extend_path_condition(
        &mut self,
        loc: &Loc,
        path_cond: Option<Exp>,
        exp: Exp,
    ) -> Option<Exp> {
        if let Some(cond) = path_cond {
            let node_id = self.new_node_id_with_type_loc(&BOOL_TYPE, loc);
            Some(Exp::Call(node_id, Operation::And, vec![cond, exp]))
        } else {
            Some(exp)
        }
    }

    /// Analyze schema inclusion in the spec block for a function, struct or module. This
    /// instantiates the schema and adds all conditions and invariants it contains to the context.
    ///
    /// The `alt_context_type_params` allows to use different type parameter names as would
    /// otherwise be inferred from the SchemaBlockContext. This is used for the apply weaving
    /// operator which allows to use different type parameter names than the function declarations
    /// to which it is applied to.
    fn def_ana_schema_inclusion_outside_schema(
        &mut self,
        loc: &Loc,
        context: &SpecBlockContext,
        alt_context_type_params: Option<&[(Symbol, Type)]>,
        exp: &EA::Exp,
    ) {
        // Compute the type parameters and variables this spec block uses. We do this by constructing
        // an expression translator and immediately extracting  from it. Depending on whether in
        // function or struct context, we use a condition kind which defines the maximum
        // of available symbols. We need to potentially revise this to only declare variables which
        // have a proper use in a condition/invariant, depending on what is actually included in
        // the block.
        let (mut vars, context_type_params) = match context {
            SpecBlockContext::Function(..) | SpecBlockContext::FunctionCode(..) => {
                let et =
                    self.exp_translator_for_context(loc, context, Some(&ConditionKind::Ensures));
                (et.extract_var_map(), et.extract_type_params())
            }
            SpecBlockContext::Struct(..) => {
                let et = self.exp_translator_for_context(
                    loc,
                    context,
                    Some(&ConditionKind::InvariantUpdate),
                );
                (et.extract_var_map(), et.extract_type_params())
            }
            SpecBlockContext::Module => (BTreeMap::new(), vec![]),
            SpecBlockContext::Schema { .. } => panic!("unexpected schema context"),
        };
        let mut spec = Spec::default();

        // Analyze the schema inclusion. This will instantiate conditions for
        // this block.
        self.def_ana_schema_exp(
            if let Some(type_params) = alt_context_type_params {
                type_params
            } else {
                &context_type_params
            },
            &mut vars,
            &mut spec,
            false,
            exp,
        );

        // Write the conditions to the context item.
        // TODO: merge pragma properties as well?
        self.add_conditions_to_context(
            context,
            loc,
            spec.conditions,
            "(included from schema) not allowed in this context",
        );
    }

    /// Analyzes a schema apply weaving operator.
    fn def_ana_schema_apply(
        &mut self,
        loc: &Loc,
        context: &SpecBlockContext,
        exp: &EA::Exp,
        patterns: &[PA::SpecApplyPattern],
        exclusion_patterns: &[PA::SpecApplyPattern],
    ) {
        if !matches!(context, SpecBlockContext::Module) {
            self.parent.error(
                loc,
                "the `apply` schema weaving operator can only be used inside a `spec module` block",
            );
            return;
        }
        for fun_name in self.parent.fun_table.keys().cloned().collect_vec() {
            // Note we need the vector clone above to avoid borrowing self for the
            // whole loop.
            let entry = self.parent.fun_table.get(&fun_name).unwrap();
            if entry.module_id != self.module_id {
                // Not a function from this module
                continue;
            }
            let is_public = entry.is_public;
            let type_arg_count = entry.type_params.len();
            let is_excluded = exclusion_patterns.iter().any(|p| {
                self.apply_pattern_matches(fun_name.symbol, is_public, type_arg_count, true, p)
            });
            if is_excluded {
                // Explicitly excluded from matching.
                continue;
            }
            if let Some(matched) = patterns.iter().find(|p| {
                self.apply_pattern_matches(fun_name.symbol, is_public, type_arg_count, false, p)
            }) {
                // This is a match, so apply this schema to this function.
                let type_params = {
                    let mut et = ExpTranslator::new(self);
                    et.analyze_and_add_type_params(&matched.value.type_parameters);
                    et.extract_type_params()
                };
                self.def_ana_schema_inclusion_outside_schema(
                    loc,
                    &SpecBlockContext::Function(fun_name),
                    Some(&type_params),
                    exp,
                );
            }
        }
    }

    /// Returns true if the pattern matches the function of name, type arity, and
    /// visibility.
    ///
    /// The `ignore_type_args` parameter is used for exclusion matches. In exclusion matches we
    /// do not want to include type args because its to easy for a user to get this wrong, so
    /// we match based only on visibility and name pattern. On the other hand, we want a user
    /// in inclusion matches to use a pattern like `*<X>` to match any generic function with
    /// one type argument.
    fn apply_pattern_matches(
        &self,
        name: Symbol,
        is_public: bool,
        type_arg_count: usize,
        ignore_type_args: bool,
        pattern: &PA::SpecApplyPattern,
    ) -> bool {
        if !ignore_type_args && pattern.value.type_parameters.len() != type_arg_count {
            return false;
        }
        if let Some(v) = &pattern.value.visibility {
            match v {
                PA::FunctionVisibility::Public(..) => {
                    if !is_public {
                        return false;
                    }
                }
                PA::FunctionVisibility::Internal => {
                    if is_public {
                        return false;
                    }
                }
            }
        }
        let rex = Regex::new(&format!(
            "^{}$",
            pattern
                .value
                .name_pattern
                .iter()
                .map(|p| match &p.value {
                    PA::SpecApplyFragment_::Wildcard => ".*".to_string(),
                    PA::SpecApplyFragment_::NamePart(n) => n.value.clone(),
                })
                .join("")
        ))
        .expect("regex valid");
        rex.is_match(self.symbol_pool().string(name).as_str())
    }
}

/// ## Spec Var Usage Analysis

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    /// Compute spec var usage of spec funs.
    fn compute_spec_var_usage(&mut self) {
        let mut visited = BTreeSet::new();
        for idx in 0..self.spec_funs.len() {
            self.compute_spec_var_usage_for_fun(&mut visited, idx);
        }
        // Check for purity requirements. All data invariants must be pure expressions and
        // not depend on global state.
        let check_pure = |mid: ModuleId, fid: SpecFunId| {
            if mid.to_usize() < self.parent.env.get_module_count() {
                // This is calling a function from another module we already have
                // translated.
                let module_env = self.parent.env.get_module(mid);
                module_env.get_spec_fun(fid).is_pure
            } else {
                // This is calling a function from the module we are currently translating.
                self.spec_funs[fid.as_usize()].is_pure
            }
        };
        for struct_spec in self.struct_specs.values() {
            for cond in &struct_spec.conditions {
                if cond.kind == ConditionKind::Invariant && !cond.exp.is_pure(&check_pure) {
                    self.parent.error(
                        &cond.loc,
                        "data invariants cannot depend on global state \
                        (directly or indirectly uses a global spec var or resource storage).",
                    );
                }
            }
        }
    }

    /// Compute spec var usage for a given spec fun, defined via its index into the spec_funs
    /// vector of the currently translated module. This recursively computes the values for
    /// functions called from this one; the visited set is there to break cycles.
    fn compute_spec_var_usage_for_fun(&mut self, visited: &mut BTreeSet<usize>, fun_idx: usize) {
        if !visited.insert(fun_idx) {
            return;
        }

        // Detach the current SpecFunDecl body so we can traverse it while at the same time mutating
        // the full self. Rust requires us to do so (at least the author doesn't know better yet),
        // but moving it should be not too expensive.
        let body = if self.spec_funs[fun_idx].body.is_some() {
            std::mem::replace(&mut self.spec_funs[fun_idx].body, None).unwrap()
        } else {
            // Native function: assume it is impure. We need a modifier to declare otherwise
            self.spec_funs[fun_idx].is_pure = false;
            return;
        };

        let mut used_spec_vars = BTreeSet::new();
        let mut is_pure = true;
        body.visit(&mut |e: &Exp| {
            match e {
                Exp::SpecVar(_, mid, vid) => {
                    used_spec_vars.insert((*mid, *vid));
                    is_pure = false;
                }
                Exp::Call(_, Operation::Function(mid, fid), _) => {
                    if mid.to_usize() < self.parent.env.get_module_count() {
                        // This is calling a function from another module we already have
                        // translated.
                        let module_env = self.parent.env.get_module(*mid);
                        let fun_decl = module_env.get_spec_fun(*fid);
                        used_spec_vars.extend(&fun_decl.used_spec_vars);
                        is_pure = is_pure && fun_decl.is_pure
                    } else {
                        // This is calling a function from the module we are currently translating.
                        // Need to recursively ensure we have computed used_spec_vars because of
                        // arbitrary call graphs, including cyclic.
                        self.compute_spec_var_usage_for_fun(visited, fid.as_usize());
                        let fun_decl = &self.spec_funs[fid.as_usize()];
                        used_spec_vars.extend(&fun_decl.used_spec_vars);
                        is_pure = is_pure && fun_decl.is_pure;
                    }
                }
                Exp::Call(_, Operation::Sender, _)
                | Exp::Call(_, Operation::Global, _)
                | Exp::Call(_, Operation::Exists, _) => is_pure = false,
                _ => {}
            }
        });

        // Store result back.
        let fun_decl = &mut self.spec_funs[fun_idx];
        fun_decl.body = Some(body);
        fun_decl.used_spec_vars = used_spec_vars;
        fun_decl.is_pure = is_pure;
    }
}

/// ## Module Invariant Reduction

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    /// Reduce module invariants by making them requires/ensures on each function.
    fn reduce_module_invariants(&mut self) {
        for mut cond in self.module_spec.conditions.iter().cloned().collect_vec() {
            assert_eq!(cond.kind, ConditionKind::Invariant);
            // An Invariant on module level becomes an InvariantModule on function level
            // (which is then further reduced to a pair of RequiresModule and Ensures).
            // Only public functions receive it.
            cond.kind = ConditionKind::InvariantModule;
            for qname in self
                .parent
                .fun_table
                .keys()
                .filter(|qn| qn.module_name == self.module_name)
                .cloned()
                .collect_vec()
            {
                let entry = self.parent.fun_table.get(&qname).unwrap();
                if entry.is_public {
                    let context = SpecBlockContext::Function(qname);
                    // The below should not generate an error because of the above assert.
                    self.add_conditions_to_context(
                        &context,
                        &cond.loc.clone(),
                        vec![cond.clone()],
                        "[internal] (included via module level invariant) not allowed in this context",
                    )
                }
            }
        }
    }
}

/// # Environment Population

impl<'env, 'translator> ModuleTranslator<'env, 'translator> {
    fn populate_env_from_result(
        &mut self,
        loc: Loc,
        module: CompiledModule,
        source_map: SourceMap<MoveIrLoc>,
    ) {
        let struct_data: BTreeMap<StructId, StructData> = (0..module.struct_defs().len())
            .filter_map(|idx| {
                let def_idx = StructDefinitionIndex(idx as u16);
                let handle_idx = module.struct_def_at(def_idx).struct_handle;
                let handle = module.struct_handle_at(handle_idx);
                let view = StructHandleView::new(&module, handle);
                let name = self.symbol_pool().make(view.name().as_str());
                if let Some(entry) = self
                    .parent
                    .struct_table
                    .get(&self.qualified_by_module(name))
                {
                    let struct_spec = self
                        .struct_specs
                        .remove(&name)
                        .unwrap_or_else(Spec::default);
                    // Ensure that all invariant kinds
                    Some((
                        StructId::new(name),
                        self.parent.env.create_struct_data(
                            &module,
                            def_idx,
                            name,
                            entry.loc.clone(),
                            struct_spec,
                        ),
                    ))
                } else {
                    self.parent.error(
                        &self.parent.env.internal_loc(),
                        &format!("[internal] bytecode does not match AST: `{}` in bytecode but not in AST", name.display(self.symbol_pool())));
                    None
                }
            })
            .collect();
        let function_data: BTreeMap<FunId, FunctionData> = (0..module.function_defs().len())
            .filter_map(|idx| {
                let def_idx = FunctionDefinitionIndex(idx as u16);
                let handle_idx = module.function_def_at(def_idx).function;
                let handle = module.function_handle_at(handle_idx);
                let view = FunctionHandleView::new(&module, handle);
                let mut name_str = view.name().as_str();
                // Script functions have different names in AST and bytecode; adjust.
                if name_str == SCRIPT_BYTECODE_FUN_NAME {
                    name_str = SCRIPT_AST_FUN_NAME;
                }
                let name = self.symbol_pool().make(name_str);
                let fun_spec = self.fun_specs.remove(&name).unwrap_or_else(Spec::default);
                if let Some(entry) = self.parent.fun_table.get(&self.qualified_by_module(name)) {
                    let arg_names = project_1st(&entry.params);
                    let type_arg_names = project_1st(&entry.type_params);
                    Some((FunId::new(name), self.parent.env.create_function_data(
                        &module,
                        def_idx,
                        name,
                        entry.loc.clone(),
                        arg_names,
                        type_arg_names,
                        fun_spec,
                    )))
                } else {
                    let funs = self.parent.fun_table.iter().map(|(k, _)| {
                        format!("{}", k.display_full(self.symbol_pool()))
                    }).join(", ");
                    self.parent.error(
                        &self.parent.env.internal_loc(),
                        &format!("[internal] bytecode does not match AST: `{}` in bytecode but not in AST (available in AST: {})", name.display(self.symbol_pool()), funs));
                    None
                }
            })
            .collect();
        self.parent.env.add(
            loc,
            module,
            source_map,
            struct_data,
            function_data,
            std::mem::take(&mut self.spec_vars),
            std::mem::take(&mut self.spec_funs),
            std::mem::take(&mut self.module_spec),
            std::mem::take(&mut self.loc_map),
            std::mem::take(&mut self.type_map),
            std::mem::take(&mut self.instantiation_map),
        );
    }
}

// =================================================================================================
/// # Expression and Type Translation

#[derive(Debug)]
pub struct ExpTranslator<'env, 'translator, 'module_translator> {
    parent: &'module_translator mut ModuleTranslator<'env, 'translator>,
    /// A symbol table for type parameters.
    type_params_table: BTreeMap<Symbol, Type>,
    /// A scoped symbol table for local names. The first element in the list contains the most
    /// inner scope.
    local_table: LinkedList<BTreeMap<Symbol, LocalVarEntry>>,
    /// When compiling a condition, the result type of the function the condition is associated
    /// with.
    result_type: Option<Type>,
    /// Status for the `old(...)` expression form.
    old_status: OldExpStatus,
    /// The currently build type substitution.
    subs: Substitution,
    /// A counter for generating type variables.
    type_var_counter: u16,
    /// A marker to indicate the node_counter start state.
    node_counter_start: usize,
}

#[derive(Debug, Clone)]
struct LocalVarEntry {
    loc: Loc,
    type_: Type,
    // If this local is associated with an operation, this is set.
    operation: Option<Operation>,
}

#[derive(Debug, PartialEq)]
enum OldExpStatus {
    NotSupported,
    OutsideOld,
    InsideOld,
}

/// ## General

impl<'env, 'translator, 'module_translator> ExpTranslator<'env, 'translator, 'module_translator> {
    fn new(parent: &'module_translator mut ModuleTranslator<'env, 'translator>) -> Self {
        let node_counter_start = parent.node_counter;
        Self {
            parent,
            type_params_table: BTreeMap::new(),
            local_table: LinkedList::new(),
            result_type: None,
            old_status: OldExpStatus::NotSupported,
            subs: Substitution::new(),
            type_var_counter: 0,
            node_counter_start,
        }
    }

    fn new_with_old(
        parent: &'module_translator mut ModuleTranslator<'env, 'translator>,
        allow_old: bool,
    ) -> Self {
        let mut et = ExpTranslator::new(parent);
        if allow_old {
            et.old_status = OldExpStatus::OutsideOld;
        } else {
            et.old_status = OldExpStatus::NotSupported;
        };
        et
    }

    /// Extract a map from names to types from the scopes of this translator.
    fn extract_var_map(&self) -> BTreeMap<Symbol, LocalVarEntry> {
        let mut vars: BTreeMap<Symbol, LocalVarEntry> = BTreeMap::new();
        for s in &self.local_table {
            vars.extend(s.clone());
        }
        vars
    }

    // Extract type parameters from this translator.
    fn extract_type_params(&self) -> Vec<(Symbol, Type)> {
        self.type_params_table
            .iter()
            .map(|(n, ty)| (*n, ty.clone()))
            .collect_vec()
    }

    /// Shortcut for accessing symbol pool.
    fn symbol_pool(&self) -> &SymbolPool {
        self.parent.parent.env.symbol_pool()
    }

    /// Shortcut for translating a Move AST location into ours.
    fn to_loc(&self, loc: &move_ir_types::location::Loc) -> Loc {
        self.parent.parent.env.to_loc(loc)
    }

    /// Shortcut for reporting an error.
    fn error(&self, loc: &Loc, msg: &str) {
        self.parent.parent.error(loc, msg);
    }

    /// Creates a fresh type variable.
    fn fresh_type_var(&mut self) -> Type {
        let var = Type::Var(self.type_var_counter);
        self.type_var_counter += 1;
        var
    }

    /// Creates a new node id and assigns type and location to it.
    fn new_node_id_with_type_loc(&mut self, ty: &Type, loc: &Loc) -> NodeId {
        self.parent.new_node_id_with_type_loc(ty, loc)
    }

    /// Sets instantiation for the given node id.
    fn set_instantiation(&mut self, node_id: NodeId, instantiation: Vec<Type>) {
        self.parent.instantiation_map.insert(node_id, instantiation);
    }

    /// Finalizes types in this translator, producing errors if some could not be inferred
    /// and remained incomplete.
    fn finalize_types(&mut self) {
        if self.parent.parent.env.has_errors() {
            // Don't do that check if we already reported errors, as this would produce
            // useless followup errors.
            return;
        }
        for i in self.node_counter_start..self.parent.node_counter {
            let node_id = NodeId::new(i);
            if let Some(ty) = self.parent.type_map.get(&node_id) {
                let ty = self.finalize_type(node_id, ty);
                self.parent.type_map.insert(node_id.clone(), ty);
            }
            if let Some(inst) = self.parent.instantiation_map.get(&node_id) {
                let inst = inst
                    .iter()
                    .map(|ty| self.finalize_type(node_id, ty))
                    .collect_vec();
                self.parent.instantiation_map.insert(node_id.clone(), inst);
            }
        }
    }

    /// Finalize the the given type, producing an error if it is not complete.
    fn finalize_type(&self, node_id: NodeId, ty: &Type) -> Type {
        let ty = self.subs.specialize(ty);
        if ty.is_incomplete() {
            // This type could not be fully inferred.
            let loc = if let Some(loc) = self.parent.loc_map.get(&node_id) {
                loc.clone()
            } else {
                self.parent.parent.env.unknown_loc()
            };
            self.error(
                &loc,
                &format!(
                    "unable to infer type: `{}`",
                    ty.display(&self.type_display_context())
                ),
            );
        }
        ty
    }

    /// Constructs a type display context used to visualize types in error messages.
    fn type_display_context(&self) -> TypeDisplayContext<'_> {
        TypeDisplayContext::WithoutEnv {
            symbol_pool: self.symbol_pool(),
            reverse_struct_table: &self.parent.parent.reverse_struct_table,
        }
    }

    /// Creates an error expression.
    fn new_error_exp(&mut self) -> Exp {
        let id =
            self.new_node_id_with_type_loc(&Type::Error, &self.parent.parent.env.internal_loc());
        Exp::Error(id)
    }

    /// Enters a new scope in the locals table.
    fn enter_scope(&mut self) {
        self.local_table.push_front(BTreeMap::new());
    }

    /// Exits the most inner scope of the locals table.
    fn exit_scope(&mut self) {
        self.local_table.pop_front();
    }

    /// Defines a type parameter.
    fn define_type_param(&mut self, loc: &Loc, name: Symbol, ty: Type) {
        if self.type_params_table.insert(name, ty).is_some() {
            let param_name = name.display(self.symbol_pool());
            self.parent
                .parent
                .error(loc, &format!("duplicate declaration of `{}`", param_name));
        }
    }

    /// Defines a local in the most inner scope. This produces an error
    /// if the name already exists. The invariant option is used for names
    /// which appear as locals in expressions, but are actually implicit selections
    /// of fields of an underlying struct.
    fn define_local(&mut self, loc: &Loc, name: Symbol, type_: Type, operation: Option<Operation>) {
        let entry = LocalVarEntry {
            loc: loc.clone(),
            type_,
            operation,
        };
        if let Some(old) = self
            .local_table
            .front_mut()
            .expect("symbol table empty")
            .insert(name, entry)
        {
            let display = name.display(self.symbol_pool());
            self.error(loc, &format!("duplicate declaration of `{}`", display));
            self.error(&old.loc, &format!("previous declaration of `{}`", display));
        }
    }

    /// Lookup a local in this translator.
    fn lookup_local(&mut self, name: Symbol) -> Option<&LocalVarEntry> {
        for scope in &self.local_table {
            if let Some(entry) = scope.get(&name) {
                return Some(entry);
            }
        }
        None
    }

    /// Analyzes the sequence of type parameters as they are provided via the source AST and enters
    /// them into the environment. Returns a vector for representing them in the target AST.
    fn analyze_and_add_type_params(
        &mut self,
        type_params: &[(Name, PA::Kind)],
    ) -> Vec<(Symbol, Type)> {
        type_params
            .iter()
            .enumerate()
            .map(|(i, (n, _))| {
                let ty = Type::TypeParameter(i as u16);
                let sym = self.symbol_pool().make(n.value.as_str());
                self.define_type_param(&self.to_loc(&n.loc), sym, ty.clone());
                (sym, ty)
            })
            .collect_vec()
    }

    /// Analyzes the sequence of function parameters as they are provided via the source AST and
    /// enters them into the environment. Returns a vector for representing them in the target AST.
    fn analyze_and_add_params(&mut self, params: &[(PA::Var, EA::Type)]) -> Vec<(Symbol, Type)> {
        params
            .iter()
            .map(|(v, ty)| {
                let ty = self.translate_type(ty);
                let sym = self.symbol_pool().make(v.0.value.as_str());
                self.define_local(&self.to_loc(&v.0.loc), sym, ty.clone(), None);
                (sym, ty)
            })
            .collect_vec()
    }

    /// Displays a call target for error messages.
    fn display_call_target(&mut self, module: &Option<ModuleName>, name: Symbol) -> String {
        if let Some(m) = module {
            if m != &self.parent.parent.builtin_module() {
                // Only print the module name if it is not the pseudo builtin module.
                return format!(
                    "{}",
                    QualifiedSymbol {
                        module_name: m.clone(),
                        symbol: name,
                    }
                    .display(self.symbol_pool())
                );
            }
        }
        format!("{}", name.display(self.symbol_pool()))
    }

    /// Displays a call target candidate for error messages.
    fn display_call_cand(
        &mut self,
        module: &Option<ModuleName>,
        name: Symbol,
        entry: &SpecFunEntry,
    ) -> String {
        let target = self.display_call_target(module, name);
        let type_display_context = self.type_display_context();
        format!(
            "{}({}): {}",
            target,
            entry
                .arg_types
                .iter()
                .map(|ty| ty.display(&type_display_context))
                .join(", "),
            entry.result_type.display(&type_display_context)
        )
    }
}

/// ## Type Translation

impl<'env, 'translator, 'module_translator> ExpTranslator<'env, 'translator, 'module_translator> {
    /// Translates an hlir type into a target AST type.
    fn translate_hlir_single_type(&mut self, ty: &SingleType) -> Type {
        use move_lang::hlir::ast::SingleType_::*;
        match &ty.value {
            Ref(is_mut, ty) => {
                let ty = self.translate_hlir_base_type(&*ty);
                if ty == Type::Error {
                    Type::Error
                } else {
                    Type::Reference(*is_mut, Box::new(ty))
                }
            }
            Base(ty) => self.translate_hlir_base_type(&*ty),
        }
    }

    fn translate_hlir_base_type(&mut self, ty: &BaseType) -> Type {
        use move_lang::{
            hlir::ast::{BaseType_::*, TypeName_::*},
            naming::ast::BuiltinTypeName_::*,
        };
        match &ty.value {
            Param(TParam {
                user_specified_name,
                ..
            }) => {
                let sym = self.symbol_pool().make(user_specified_name.value.as_str());
                self.type_params_table[&sym].clone()
            }
            Apply(_, type_name, args) => {
                let loc = self.to_loc(&type_name.loc);
                match &type_name.value {
                    Builtin(builtin_type_name) => match &builtin_type_name.value {
                        Address => Type::new_prim(PrimitiveType::Address),
                        U8 => Type::new_prim(PrimitiveType::U8),
                        U64 => Type::new_prim(PrimitiveType::U64),
                        U128 => Type::new_prim(PrimitiveType::U128),
                        Vector => Type::Vector(Box::new(self.translate_hlir_base_type(&args[0]))),
                        Bool => Type::new_prim(PrimitiveType::Bool),
                    },
                    ModuleType(m, n) => {
                        let module_name = ModuleName::from_str(
                            &m.0.value.address.to_string(),
                            self.symbol_pool().make(m.0.value.name.0.value.as_str()),
                        );
                        let symbol = self.symbol_pool().make(n.0.value.as_str());
                        let qsym = QualifiedSymbol {
                            module_name,
                            symbol,
                        };
                        let rty = self.parent.parent.lookup_type(&loc, &qsym);
                        if !args.is_empty() {
                            // Replace type instantiation.
                            if let Type::Struct(mid, sid, _) = &rty {
                                let arg_types = self.translate_hlir_base_types(args);
                                if arg_types.iter().any(|x| *x == Type::Error) {
                                    Type::Error
                                } else {
                                    Type::Struct(*mid, *sid, arg_types)
                                }
                            } else {
                                Type::Error
                            }
                        } else {
                            rty
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn translate_hlir_base_types(&mut self, tys: &[BaseType]) -> Vec<Type> {
        tys.iter()
            .map(|t| self.translate_hlir_base_type(t))
            .collect()
    }

    /// Translates a source AST type into a target AST type.
    fn translate_type(&mut self, ty: &EA::Type) -> Type {
        use EA::Type_::*;
        match &ty.value {
            Apply(access, args) => {
                if let EA::ModuleAccess_::Name(n) = &access.value {
                    // Attempt to resolve as primitive type.
                    match n.value.as_str() {
                        "bool" => return Type::new_prim(PrimitiveType::Bool),
                        "u8" => return Type::new_prim(PrimitiveType::U8),
                        "u64" => return Type::new_prim(PrimitiveType::U64),
                        "u128" => return Type::new_prim(PrimitiveType::U128),
                        "num" => return Type::new_prim(PrimitiveType::Num),
                        "range" => return Type::new_prim(PrimitiveType::Range),
                        "address" => return Type::new_prim(PrimitiveType::Address),
                        "vector" => {
                            if args.len() != 1 {
                                self.error(
                                    &self.to_loc(&ty.loc),
                                    "expected one type argument for `vector`",
                                );
                                return Type::Error;
                            } else {
                                return Type::Vector(Box::new(self.translate_type(&args[0])));
                            }
                        }
                        _ => {}
                    }
                    // Attempt to resolve as a type parameter.
                    let sym = self.symbol_pool().make(n.value.as_str());
                    if let Some(ty) = self.type_params_table.get(&sym) {
                        return ty.clone();
                    }
                }
                let loc = self.to_loc(&access.loc);
                let sym = self.parent.module_access_to_qualified(access);
                let rty = self.parent.parent.lookup_type(&loc, &sym);
                if !args.is_empty() {
                    // Replace type instantiation.
                    if let Type::Struct(mid, sid, params) = &rty {
                        if params.len() != args.len() {
                            self.error(&loc, "type argument count mismatch");
                            Type::Error
                        } else {
                            Type::Struct(*mid, *sid, self.translate_types(args))
                        }
                    } else {
                        self.error(&loc, "type cannot have type arguments");
                        Type::Error
                    }
                } else {
                    rty
                }
            }
            Ref(is_mut, ty) => Type::Reference(*is_mut, Box::new(self.translate_type(&*ty))),
            Fun(args, result) => Type::Fun(
                self.translate_types(&args),
                Box::new(self.translate_type(&*result)),
            ),
            Unit => Type::Tuple(vec![]),
            Multiple(vst) => Type::Tuple(self.translate_types(vst)),
            UnresolvedError => Type::Error,
        }
    }

    /// Translates a slice of single types.
    fn translate_types(&mut self, tys: &[EA::Type]) -> Vec<Type> {
        tys.iter().map(|t| self.translate_type(t)).collect()
    }

    /// Translates option a slice of single types.
    fn translate_types_opt(&mut self, tys_opt: &Option<Vec<EA::Type>>) -> Vec<Type> {
        tys_opt
            .as_deref()
            .map(|tys| self.translate_types(tys))
            .unwrap_or_else(|| vec![])
    }
}

/// ## Expression Translation

impl<'env, 'translator, 'module_translator> ExpTranslator<'env, 'translator, 'module_translator> {
    /// Translates an expression, with given expected type, which might be a type variable.
    fn translate_exp(&mut self, exp: &EA::Exp, expected_type: &Type) -> Exp {
        let loc = self.to_loc(&exp.loc);
        let make_value = |et: &mut ExpTranslator, val: Value, ty: Type| {
            let rty = et.check_type(&loc, &ty, expected_type, "in expression");
            let id = et.new_node_id_with_type_loc(&rty, &loc);
            Exp::Value(id, val)
        };
        match &exp.value {
            EA::Exp_::Value(v) => {
                if let Some((v, ty)) = self.translate_value(v) {
                    make_value(self, v, ty)
                } else {
                    self.new_error_exp()
                }
            }
            EA::Exp_::InferredNum(x) => {
                // We don't really need to infer type, because all ints are exchangeable.
                make_value(
                    self,
                    Value::Number(BigUint::from_u128(*x).unwrap()),
                    Type::new_prim(PrimitiveType::U128),
                )
            }
            EA::Exp_::Name(maccess, type_params) => {
                self.translate_name(&loc, maccess, type_params.as_deref(), expected_type)
            }
            EA::Exp_::Call(maccess, type_params, args) => {
                // Need to make a &[&Exp] out of args.
                let args = args.value.iter().map(|e| e).collect_vec();
                self.translate_fun_call(
                    expected_type,
                    &loc,
                    &maccess,
                    type_params.as_deref(),
                    &args,
                )
            }
            EA::Exp_::Pack(maccess, generics, fields) => {
                self.translate_pack(&loc, maccess, generics, fields, expected_type)
            }
            EA::Exp_::IfElse(cond, then, else_) => {
                let then = self.translate_exp(&*then, expected_type);
                let else_ = self.translate_exp(&*else_, expected_type);
                let cond = self.translate_exp(&*cond, &Type::new_prim(PrimitiveType::Bool));
                let id = self.new_node_id_with_type_loc(expected_type, &loc);
                Exp::IfElse(id, Box::new(cond), Box::new(then), Box::new(else_))
            }
            EA::Exp_::Block(seq) => self.translate_seq(&loc, seq, expected_type),
            EA::Exp_::Lambda(bindings, exp) => {
                self.translate_lambda(&loc, bindings, exp, expected_type)
            }
            EA::Exp_::BinopExp(l, op, r) => {
                let args = vec![l.as_ref(), r.as_ref()];
                let QualifiedSymbol {
                    module_name,
                    symbol,
                } = self.parent.parent.bin_op_symbol(&op.value);
                self.translate_call(&loc, &Some(module_name), symbol, None, &args, expected_type)
            }
            EA::Exp_::UnaryExp(op, exp) => {
                let args = vec![exp.as_ref()];
                let QualifiedSymbol {
                    module_name,
                    symbol,
                } = self.parent.parent.unary_op_symbol(&op.value);
                self.translate_call(&loc, &Some(module_name), symbol, None, &args, expected_type)
            }
            EA::Exp_::ExpDotted(dotted) => self.translate_dotted(dotted, expected_type),
            EA::Exp_::Index(target, index) => {
                self.translate_index(&loc, target, index, expected_type)
            }
            EA::Exp_::ExpList(exps) => {
                let mut types = vec![];
                let exps = exps
                    .iter()
                    .map(|exp| {
                        let (ty, exp) = self.translate_exp_free(exp);
                        types.push(ty);
                        exp
                    })
                    .collect_vec();
                let ty = self.check_type(
                    &loc,
                    &Type::Tuple(types),
                    expected_type,
                    "in expression list",
                );
                let id = self.new_node_id_with_type_loc(&ty, &loc);
                Exp::Call(id, Operation::Tuple, exps)
            }
            EA::Exp_::Unit => {
                let ty = self.check_type(
                    &loc,
                    &Type::Tuple(vec![]),
                    expected_type,
                    "in unit expression",
                );
                let id = self.new_node_id_with_type_loc(&ty, &loc);
                Exp::Call(id, Operation::Tuple, vec![])
            }
            EA::Exp_::Assign(..) => {
                self.error(&loc, "assignment only allowed in spec var updates");
                self.new_error_exp()
            }
            _ => {
                self.error(&loc, "expression construct not supported in specifications");
                self.new_error_exp()
            }
        }
    }

    fn translate_value(&mut self, v: &PA::Value) -> Option<(Value, Type)> {
        let loc = self.to_loc(&v.loc);
        match &v.value {
            PA::Value_::Address(addr) => {
                let addr_str = &format!("{}", addr);
                if &addr_str[..2] == "0x" {
                    let digits_only = &addr_str[2..];
                    Some((
                        Value::Address(
                            BigUint::from_str_radix(digits_only, 16).expect("valid address"),
                        ),
                        Type::new_prim(PrimitiveType::Address),
                    ))
                } else {
                    self.error(&loc, "address string does not begin with '0x'");
                    None
                }
            }
            PA::Value_::U8(x) => Some((
                Value::Number(BigUint::from_u8(*x).unwrap()),
                Type::new_prim(PrimitiveType::U8),
            )),
            PA::Value_::U64(x) => Some((
                Value::Number(BigUint::from_u64(*x).unwrap()),
                Type::new_prim(PrimitiveType::U64),
            )),
            PA::Value_::U128(x) => Some((
                Value::Number(BigUint::from_u128(*x).unwrap()),
                Type::new_prim(PrimitiveType::U128),
            )),
            PA::Value_::Bool(x) => Some((Value::Bool(*x), Type::new_prim(PrimitiveType::Bool))),
            PA::Value_::Bytearray(x) => {
                let ty = Type::Vector(Box::new(Type::new_prim(PrimitiveType::U8)));
                Some((Value::ByteArray(x.clone()), ty))
            }
        }
    }

    fn translate_fun_call(
        &mut self,
        expected_type: &Type,
        loc: &Loc,
        maccess: &Spanned<EA::ModuleAccess_>,
        generics: Option<&[EA::Type]>,
        args: &[&EA::Exp],
    ) -> Exp {
        // First check whether this is an Invoke on a function value.
        if let EA::ModuleAccess_::Name(n) = &maccess.value {
            let sym = self.symbol_pool().make(&n.value);
            if let Some(entry) = self.lookup_local(sym) {
                // Check whether the local has the expected function type.
                let sym_ty = entry.type_.clone();
                let (arg_types, args) = self.translate_exp_list(args);
                let fun_t = Type::Fun(arg_types, Box::new(expected_type.clone()));
                let sym_ty = self.check_type(&loc, &sym_ty, &fun_t, "in expression");
                let local_id = self.new_node_id_with_type_loc(&sym_ty, &self.to_loc(&n.loc));
                let local_var = Exp::LocalVar(local_id, sym);
                let id = self.new_node_id_with_type_loc(expected_type, &loc);
                return Exp::Invoke(id, Box::new(local_var), args);
            }
        }
        // Next treat this as a call to a global function.
        let (module_name, name) = self.parent.module_access_to_parts(maccess);
        let is_old = module_name.is_none() && name == self.parent.parent.old_symbol();
        if is_old {
            match self.old_status {
                OldExpStatus::NotSupported => {
                    self.error(&loc, "`old(..)` expression not allowed in this context");
                }
                OldExpStatus::InsideOld => {
                    self.error(&loc, "`old(..old(..)..)` not allowed");
                }
                OldExpStatus::OutsideOld => {
                    self.old_status = OldExpStatus::InsideOld;
                }
            }
        }
        let result = self.translate_call(&loc, &module_name, name, generics, args, expected_type);
        if is_old && self.old_status == OldExpStatus::InsideOld {
            self.old_status = OldExpStatus::OutsideOld;
        }
        result
    }

    /// Translates an expression without any known type expectation. This creates a fresh type
    /// variable and passes this in as expected type, then returns a pair of this type and the
    /// translated expression.
    fn translate_exp_free(&mut self, exp: &EA::Exp) -> (Type, Exp) {
        let tvar = self.fresh_type_var();
        let exp = self.translate_exp(exp, &tvar);
        (tvar, exp)
    }

    /// Translates a sequence expression.
    fn translate_seq(&mut self, loc: &Loc, seq: &EA::Sequence, expected_type: &Type) -> Exp {
        let n = seq.len();
        if n == 0 {
            self.error(loc, "block sequence cannot be empty");
            return self.new_error_exp();
        }
        // Process all items before the last one, which must be bindings, and accumulate
        // declarations for them.
        let mut decls = vec![];
        let seq = seq.iter().collect_vec();
        for item in &seq[0..seq.len() - 1] {
            match &item.value {
                EA::SequenceItem_::Bind(list, exp) => {
                    let (t, e) = self.translate_exp_free(exp);
                    if list.value.len() != 1 {
                        self.error(
                            &self.to_loc(&list.loc),
                            "[current restriction] tuples not supported in let",
                        );
                        return Exp::Error(self.parent.new_node_id());
                    }
                    let bind_loc = self.to_loc(&list.value[0].loc);
                    match &list.value[0].value {
                        EA::LValue_::Var(maccess, _) => {
                            let name = match &maccess.value {
                                EA::ModuleAccess_::Name(n) => n,
                                EA::ModuleAccess_::ModuleAccess(_, n) => n,
                            };
                            // Declare the variable in the local environment. Currently we mimic
                            // Rust/ML semantics here, allowing to shadow with each let.
                            self.enter_scope();
                            let name = self.symbol_pool().make(&name.value);
                            self.define_local(&bind_loc, name, t.clone(), None);
                            let id = self.new_node_id_with_type_loc(&t, &bind_loc);
                            decls.push(LocalVarDecl {
                                id,
                                name,
                                binding: Some(e),
                            });
                        }
                        EA::LValue_::Unpack(..) => {
                            self.error(
                                &bind_loc,
                                "[current restriction] unpack not supported in let",
                            );
                            return Exp::Error(self.parent.new_node_id());
                        }
                    }
                }
                _ => self.error(
                    &self.to_loc(&item.loc),
                    "only binding `let p = e; ...` allowed here",
                ),
            }
        }

        // Process the last element, which must be an Exp item.
        let last = match &seq[n - 1].value {
            EA::SequenceItem_::Seq(e) => self.translate_exp(e, expected_type),
            _ => {
                self.error(
                    &self.to_loc(&seq[n - 1].loc),
                    "expected an expression as the last element of the block",
                );
                self.new_error_exp()
            }
        };

        // Exit the scopes for variable bindings
        for _ in 0..decls.len() {
            self.exit_scope();
        }

        let id = self.new_node_id_with_type_loc(expected_type, loc);
        Exp::Block(id, decls, Box::new(last))
    }

    /// Translates a name. Reports an error if the name is not found.
    fn translate_name(
        &mut self,
        loc: &Loc,
        maccess: &EA::ModuleAccess,
        type_args: Option<&[EA::Type]>,
        expected_type: &Type,
    ) -> Exp {
        let spec_var_sym = match &maccess.value {
            EA::ModuleAccess_::ModuleAccess(..) => self.parent.module_access_to_qualified(maccess),
            EA::ModuleAccess_::Name(name) => {
                // First try to resolve simple name as local.
                let sym = self.symbol_pool().make(name.value.as_str());
                if let Some(entry) = self.lookup_local(sym) {
                    let oper_opt = entry.operation.clone();
                    let ty = entry.type_.clone();
                    let ty = self.check_type(loc, &ty, expected_type, "in name expression");
                    let id = self.new_node_id_with_type_loc(&ty, loc);
                    if let Some(oper) = oper_opt {
                        return Exp::Call(id, oper, vec![]);
                    } else {
                        return Exp::LocalVar(id, sym);
                    }
                }
                // If not found, treat as spec var.
                self.parent.qualified_by_module(sym)
            }
        };
        if let Some(entry) = self.parent.parent.spec_var_table.get(&spec_var_sym) {
            let type_args = type_args.unwrap_or(&[]);
            if entry.type_params.len() != type_args.len() {
                self.error(
                    loc,
                    &format!(
                        "generic count mismatch (expected {} but found {})",
                        entry.type_params.len(),
                        type_args.len()
                    ),
                );
                return self.new_error_exp();
            }
            let ty = entry.type_.clone();
            let module_id = entry.module_id;
            let var_id = entry.var_id;
            let instantiation = self.translate_types(type_args);
            let ty = ty.instantiate(&instantiation);
            let ty = self.check_type(loc, &ty, expected_type, "in spec var expression");
            let id = self.new_node_id_with_type_loc(&ty, loc);
            // Remember the instantiation as an attribute on the expression node.
            self.set_instantiation(id, instantiation);
            return Exp::SpecVar(id, module_id, var_id);
        }

        self.error(
            loc,
            &format!("undeclared `{}`", spec_var_sym.display(self.symbol_pool())),
        );
        self.new_error_exp()
    }

    /// Translate an Index expression.
    fn translate_index(
        &mut self,
        loc: &Loc,
        target: &EA::Exp,
        index: &EA::Exp,
        expected_type: &Type,
    ) -> Exp {
        // We must concretize the type of index to decide whether this is a slice
        // or not. This is not compatible with full type inference, so we may
        // try to actually represent slicing explicitly in the syntax to fix this.
        // Alternatively, we could leave it to the backend to figure (after full
        // type inference) whether this is slice or index.
        let elem_ty = self.fresh_type_var();
        let vector_ty = Type::Vector(Box::new(elem_ty.clone()));
        let vector_exp = self.translate_exp(target, &vector_ty);
        let (index_ty, ie) = self.translate_exp_free(index);
        let index_ty = self.subs.specialize(&index_ty);
        let (result_t, oper) = if let Type::Primitive(PrimitiveType::Range) = &index_ty {
            (vector_ty, Operation::Slice)
        } else {
            // If this is not (known to be) a range, assume its an index.
            self.check_type(
                &loc,
                &index_ty,
                &Type::new_prim(PrimitiveType::Num),
                "in index expression",
            );
            (elem_ty, Operation::Index)
        };
        let result_t = self.check_type(loc, &result_t, expected_type, "in index expression");
        let id = self.new_node_id_with_type_loc(&result_t, &loc);
        Exp::Call(id, oper, vec![vector_exp, ie])
    }

    /// Translate a Dotted expression.
    fn translate_dotted(&mut self, dotted: &EA::ExpDotted, expected_type: &Type) -> Exp {
        // Similar as with Index, we must concretize the type of the expression on which
        // field selection is performed, violating full type inference rules, so we can actually
        // check and retrieve the field. To avoid this, we would need to introduce the concept
        // of a type constraint to type unification, where the constraint would be
        // 'type var X where X has field F'. This makes unification significant more complex,
        // so lets see how far we get without this.
        match &dotted.value {
            EA::ExpDotted_::Exp(e) => self.translate_exp(e, expected_type),
            EA::ExpDotted_::Dot(e, n) => {
                let loc = self.to_loc(&dotted.loc);
                let field_name = self.symbol_pool().make(&n.value);
                let ty = self.fresh_type_var();
                let exp = self.translate_dotted(e.as_ref(), &ty);
                // Try to concretize the type and check its a struct.
                let struct_ty = self.subs.specialize(&ty);
                if let Type::Struct(mid, sid, targs) = &struct_ty {
                    // Lookup the StructEntry in the translator. It must be defined for valid
                    // Type::Struct instances.
                    let struct_name = self
                        .parent
                        .parent
                        .reverse_struct_table
                        .get(&(*mid, *sid))
                        .expect("invalid Type::Struct");
                    let entry = self
                        .parent
                        .parent
                        .struct_table
                        .get(struct_name)
                        .expect("invalid Type::Struct")
                        .clone();
                    // Lookup the field in the struct.
                    if let Some(fields) = &entry.fields {
                        if let Some((_, field_ty)) = fields.get(&field_name) {
                            // We must instantiate the field type by the provided type args.
                            let field_ty = field_ty.instantiate(targs);
                            self.check_type(&loc, &field_ty, expected_type, "in field selection");
                            let id = self.new_node_id_with_type_loc(&field_ty, &loc);
                            Exp::Call(
                                id,
                                Operation::Select(
                                    entry.module_id,
                                    entry.struct_id,
                                    FieldId::new(field_name),
                                ),
                                vec![exp],
                            )
                        } else {
                            self.error(
                                &loc,
                                &format!(
                                    "field `{}` not declared in struct `{}`",
                                    field_name.display(self.symbol_pool()),
                                    struct_name.display(self.symbol_pool())
                                ),
                            );
                            self.new_error_exp()
                        }
                    } else {
                        self.error(
                            &loc,
                            &format!(
                                "struct `{}` is native and does not support field selection",
                                struct_name.display(self.symbol_pool())
                            ),
                        );
                        self.new_error_exp()
                    }
                } else {
                    self.error(
                        &loc,
                        &format!(
                            "type `{}` cannot be resolved as a struct",
                            struct_ty.display(&self.type_display_context()),
                        ),
                    );
                    self.new_error_exp()
                }
            }
        }
    }

    /// Translates a call, performing overload resolution. Reports an error if the function cannot be found.
    /// This is used to resolve both calls to user spec functions and builtin operators.
    fn translate_call(
        &mut self,
        loc: &Loc,
        module: &Option<ModuleName>,
        name: Symbol,
        generics: Option<&[EA::Type]>,
        args: &[&EA::Exp],
        expected_type: &Type,
    ) -> Exp {
        // Translate generic arguments, if any.
        let generics = generics.as_ref().map(|ts| self.translate_types(&ts));
        // Translate arguments.
        let (arg_types, args) = self.translate_exp_list(args);
        // Lookup candidates.
        let cand_modules = if let Some(m) = module {
            vec![m.clone()]
        } else {
            // For an unqualified name, resolve it both in this and in the builtin pseudo module.
            vec![
                self.parent.module_name.clone(),
                self.parent.parent.builtin_module(),
            ]
        };
        let mut cands = vec![];
        for module_name in cand_modules {
            let full_name = QualifiedSymbol {
                module_name,
                symbol: name,
            };
            if let Some(list) = self.parent.parent.spec_fun_table.get(&full_name) {
                cands.extend_from_slice(list);
            }
        }
        if cands.is_empty() {
            let display = self.display_call_target(module, name);
            self.error(loc, &format!("no function named `{}` found", display));
            return self.new_error_exp();
        }
        // Partition candidates in those which matched and which have been outruled.
        let mut outruled = vec![];
        let mut matching = vec![];
        for cand in &cands {
            if cand.arg_types.len() != args.len() {
                outruled.push((
                    cand,
                    format!(
                        "argument count mismatch (expected {} but found {})",
                        cand.arg_types.len(),
                        args.len()
                    ),
                ));
                continue;
            }
            let (instantiation, diag) =
                self.make_instantiation(cand.type_params.len(), generics.clone());
            if let Some(msg) = diag {
                outruled.push((cand, msg));
                continue;
            }
            // Clone the current substitution, then unify arguments against parameter types.
            let mut subs = self.subs.clone();
            let mut success = true;
            for (i, arg_ty) in arg_types.iter().enumerate() {
                let instantiated = cand.arg_types[i].instantiate(&instantiation);
                if let Err(err) = subs.unify(&self.type_display_context(), arg_ty, &instantiated) {
                    outruled.push((cand, format!("{} for argument {}", err.message, i + 1)));
                    success = false;
                    break;
                }
            }
            if success {
                matching.push((cand, subs, instantiation));
            }
        }
        // Deliver results, reporting errors if there are no or ambiguous matches.
        match matching.len() {
            0 => {
                let display = self.display_call_target(module, name);
                let notes = outruled
                    .iter()
                    .map(|(cand, msg)| {
                        format!(
                            "outruled candidate `{}` ({})",
                            self.display_call_cand(module, name, cand),
                            msg
                        )
                    })
                    .collect_vec();
                self.parent.parent.env.error_with_notes(
                    loc,
                    &format!("no matching declaration of `{}`", display),
                    notes,
                );
                self.new_error_exp()
            }
            1 => {
                let (cand, subs, instantiation) = matching.remove(0);
                // Commit the candidate substitution to this expression translator.
                self.subs = subs;
                // Check result type against expected type.
                let ty = self.check_type(
                    loc,
                    &cand.result_type.instantiate(&instantiation),
                    expected_type,
                    "in expression",
                );
                // Construct result.
                let id = self.new_node_id_with_type_loc(&ty, loc);
                self.set_instantiation(id, instantiation);
                Exp::Call(id, cand.oper.clone(), args)
            }
            _ => {
                let display = self.display_call_target(module, name);
                let notes = matching
                    .iter()
                    .map(|(cand, _, _)| {
                        format!(
                            "matching candidate `{}`",
                            self.display_call_cand(module, name, cand)
                        )
                    })
                    .collect_vec();
                self.parent.parent.env.error_with_notes(
                    loc,
                    &format!("ambiguous application of `{}`", display),
                    notes,
                );
                self.new_error_exp()
            }
        }
    }

    /// Translate a list of expressions and deliver them together with their types.
    fn translate_exp_list(&mut self, exps: &[&EA::Exp]) -> (Vec<Type>, Vec<Exp>) {
        let mut types = vec![];
        let exps = exps
            .iter()
            .map(|e| {
                let (t, e) = self.translate_exp_free(e);
                types.push(t);
                e
            })
            .collect_vec();
        (types, exps)
    }

    /// Creates a type instantiation based on provided actual type parameters.
    fn make_instantiation(
        &mut self,
        param_count: usize,
        actuals: Option<Vec<Type>>,
    ) -> (Vec<Type>, Option<String>) {
        if let Some(types) = actuals {
            let n = types.len();
            if n != param_count {
                (
                    types,
                    Some(format!(
                        "generic count mismatch (expected {} but found {})",
                        param_count, n,
                    )),
                )
            } else {
                (types, None)
            }
        } else {
            // Create fresh type variables.
            (
                (0..param_count)
                    .map(|_| self.fresh_type_var())
                    .collect_vec(),
                None,
            )
        }
    }

    fn translate_pack(
        &mut self,
        loc: &Loc,
        maccess: &EA::ModuleAccess,
        generics: &Option<Vec<EA::Type>>,
        fields: &EA::Fields<EA::Exp>,
        expected_type: &Type,
    ) -> Exp {
        let struct_name = self.parent.module_access_to_qualified(maccess);
        let struct_name_loc = self.to_loc(&maccess.loc);
        let generics = generics.as_ref().map(|ts| self.translate_types(&ts));
        if let Some(entry) = self.parent.parent.struct_table.get(&struct_name) {
            let entry = entry.clone();
            let (instantiation, diag) = self.make_instantiation(entry.type_params.len(), generics);
            if let Some(msg) = diag {
                self.error(loc, &msg);
                return self.new_error_exp();
            }
            if let Some(field_decls) = &entry.fields {
                let mut fields_not_convered: BTreeSet<Symbol> = BTreeSet::new();
                fields_not_convered.extend(field_decls.keys());
                let mut args = BTreeMap::new();
                for (ref name, (_, exp)) in fields.iter() {
                    let field_name = self.symbol_pool().make(&name.0.value);
                    if let Some((idx, field_ty)) = field_decls.get(&field_name) {
                        let exp = self.translate_exp(exp, &field_ty.instantiate(&instantiation));
                        fields_not_convered.remove(&field_name);
                        args.insert(idx, exp);
                    } else {
                        self.error(
                            &self.to_loc(&name.0.loc),
                            &format!(
                                "field `{}` not declared in struct `{}`",
                                field_name.display(self.symbol_pool()),
                                struct_name.display(self.symbol_pool())
                            ),
                        );
                    }
                }
                if !fields_not_convered.is_empty() {
                    self.error(
                        loc,
                        &format!(
                            "missing fields {}",
                            fields_not_convered
                                .iter()
                                .map(|n| format!("`{}`", n.display(self.symbol_pool())))
                                .join(", ")
                        ),
                    );
                    self.new_error_exp()
                } else {
                    let struct_ty = Type::Struct(entry.module_id, entry.struct_id, instantiation);
                    let struct_ty =
                        self.check_type(loc, &struct_ty, expected_type, "in pack expression");
                    let args = args
                        .into_iter()
                        .sorted_by_key(|(i, _)| *i)
                        .map(|(_, e)| e)
                        .collect_vec();
                    let id = self.new_node_id_with_type_loc(&struct_ty, loc);
                    Exp::Call(id, Operation::Pack(entry.module_id, entry.struct_id), args)
                }
            } else {
                self.error(
                    &struct_name_loc,
                    &format!(
                        "native struct `{}` cannot be packed",
                        struct_name.display(self.symbol_pool())
                    ),
                );
                self.new_error_exp()
            }
        } else {
            self.error(
                &struct_name_loc,
                &format!(
                    "undeclared struct `{}`",
                    struct_name.display(self.symbol_pool())
                ),
            );
            self.new_error_exp()
        }
    }

    fn translate_lambda(
        &mut self,
        loc: &Loc,
        bindings: &EA::LValueList,
        body: &EA::Exp,
        expected_type: &Type,
    ) -> Exp {
        // Enter the lambda variables into a new local scope and collect their declarations.
        self.enter_scope();
        let mut decls = vec![];
        let mut arg_types = vec![];
        for bind in &bindings.value {
            let loc = self.to_loc(&bind.loc);
            match &bind.value {
                EA::LValue_::Var(
                    Spanned {
                        value: EA::ModuleAccess_::Name(n),
                        ..
                    },
                    _,
                ) => {
                    let name = self.symbol_pool().make(&n.value);
                    let ty = self.fresh_type_var();
                    let id = self.new_node_id_with_type_loc(&ty, &loc);
                    self.define_local(&loc, name, ty.clone(), None);
                    arg_types.push(ty);
                    decls.push(LocalVarDecl {
                        id,
                        name,
                        binding: None,
                    });
                }
                EA::LValue_::Unpack(..) | EA::LValue_::Var(..) => {
                    self.error(&loc, "[current restriction] tuples not supported in lambda")
                }
            }
        }
        // Translate the body.
        let ty = self.fresh_type_var();
        let rbody = self.translate_exp(body, &ty);
        // Check types and construct result.
        let rty = self.check_type(
            loc,
            &Type::Fun(arg_types, Box::new(ty)),
            expected_type,
            "in lambda body",
        );
        let id = self.new_node_id_with_type_loc(&rty, loc);
        Exp::Lambda(id, decls, Box::new(rbody))
    }

    fn check_type(&mut self, loc: &Loc, ty: &Type, expected: &Type, context_msg: &str) -> Type {
        // Because of Rust borrow semantics, we must temporarily detach the substitution from
        // the translator. This is because we also need to inherently borrow self via the
        // type_display_context which is passed into unification.
        let mut subs = std::mem::replace(&mut self.subs, Substitution::new());
        let result = match subs.unify(&self.type_display_context(), ty, expected) {
            Ok(t) => t,
            Err(err) => {
                self.error(&loc, &format!("{} {}", err.message, context_msg));
                Type::Error
            }
        };
        std::mem::replace(&mut self.subs, subs);
        result
    }
}

/// ## Expression Rewriting

/// Rewriter for expressions, allowing to substitute locals by expressions as well as instantiate
/// types. Currently used to rewrite conditions and invariants included from schemas.
struct ExpRewriter<'env, 'translator, 'rewriter> {
    parent: &'rewriter mut ModuleTranslator<'env, 'translator>,
    argument_map: &'rewriter BTreeMap<Symbol, Exp>,
    type_args: &'rewriter [Type],
    shadowed: VecDeque<BTreeSet<Symbol>>,
    originating_module: ModuleId,
}

impl<'env, 'translator, 'rewriter> ExpRewriter<'env, 'translator, 'rewriter> {
    fn new(
        parent: &'rewriter mut ModuleTranslator<'env, 'translator>,
        originating_module: ModuleId,
        argument_map: &'rewriter BTreeMap<Symbol, Exp>,
        type_args: &'rewriter [Type],
    ) -> Self {
        ExpRewriter {
            parent,
            argument_map,
            type_args,
            shadowed: VecDeque::new(),
            originating_module,
        }
    }

    fn replace_local(&mut self, node_id: NodeId, sym: Symbol) -> Exp {
        for vars in &self.shadowed {
            if vars.contains(&sym) {
                let node_id = self.rewrite_attrs(node_id);
                return Exp::LocalVar(node_id, sym);
            }
        }
        if let Some(exp) = self.argument_map.get(&sym) {
            exp.clone()
        } else {
            let node_id = self.rewrite_attrs(node_id);
            Exp::LocalVar(node_id, sym)
        }
    }

    fn rewrite(&mut self, exp: &Exp) -> Exp {
        use Exp::*;
        match exp {
            LocalVar(id, sym) => self.replace_local(*id, *sym),
            Call(id, oper, args) => Call(
                self.rewrite_attrs(*id),
                oper.clone(),
                self.rewrite_vec(args),
            ),
            Invoke(id, target, args) => Invoke(
                self.rewrite_attrs(*id),
                Box::new(self.rewrite(target)),
                self.rewrite_vec(args),
            ),
            Lambda(id, vars, body) => {
                self.shadowed
                    .push_front(vars.iter().map(|decl| decl.name).collect());
                let res = Lambda(
                    self.rewrite_attrs(*id),
                    vars.clone(),
                    Box::new(self.rewrite(body)),
                );
                self.shadowed.pop_front();
                res
            }
            Block(id, vars, body) => {
                self.shadowed
                    .push_front(vars.iter().map(|decl| decl.name).collect());
                let res = Block(
                    self.rewrite_attrs(*id),
                    vars.clone(),
                    Box::new(self.rewrite(body)),
                );
                self.shadowed.pop_front();
                res
            }
            IfElse(id, cond, then, else_) => IfElse(
                self.rewrite_attrs(*id),
                Box::new(self.rewrite(cond)),
                Box::new(self.rewrite(then)),
                Box::new(self.rewrite(else_)),
            ),
            Error(..) | Value(..) | SpecVar(..) => exp.clone(),
        }
    }

    fn rewrite_vec(&mut self, exps: &[Exp]) -> Vec<Exp> {
        // For some reason, we don't get the lifetime right when we use a map. Figure out
        // why and remove this explicit treatment.
        let mut res = vec![];
        for exp in exps {
            res.push(self.rewrite(exp));
        }
        res
    }

    fn rewrite_attrs(&mut self, node_id: NodeId) -> NodeId {
        // Create a new node id and copy attributes over, after instantiation with type args.
        let (loc, ty, instantiation_opt) = if self.parent.module_id == self.originating_module {
            let loc = self
                .parent
                .loc_map
                .get(&node_id)
                .expect("loc defined")
                .clone();
            let ty = self
                .parent
                .type_map
                .get(&node_id)
                .expect("type defined")
                .instantiate(self.type_args);
            let instantiation_opt = self.parent.instantiation_map.get(&node_id).map(|tys| {
                tys.iter()
                    .map(|ty| ty.instantiate(self.type_args))
                    .collect_vec()
            });
            (loc, ty, instantiation_opt)
        } else {
            let module_env = self.parent.parent.env.get_module(self.originating_module);
            let loc = module_env.get_node_loc(node_id);
            let ty = module_env.get_node_type(node_id);
            let instantiation = module_env.get_node_instantiation(node_id);
            (
                loc,
                ty,
                if instantiation.is_empty() {
                    None
                } else {
                    Some(instantiation)
                },
            )
        };
        let new_node_id = self.parent.new_node_id();
        self.parent.loc_map.insert(new_node_id, loc);
        self.parent.type_map.insert(new_node_id, ty);
        if let Some(instantiation) = instantiation_opt {
            self.parent
                .instantiation_map
                .insert(new_node_id, instantiation);
        }
        new_node_id
    }
}

/// Extract all accesses of a schema from a schema expression.
fn extract_schema_access<'a>(exp: &'a EA::Exp, res: &mut Vec<&'a EA::ModuleAccess>) {
    match &exp.value {
        EA::Exp_::Name(maccess, _) => res.push(maccess),
        EA::Exp_::BinopExp(_, _, rhs) => extract_schema_access(rhs, res),
        EA::Exp_::IfElse(_, t, e) => {
            extract_schema_access(t, res);
            extract_schema_access(e, res);
        }
        _ => {}
    }
}
