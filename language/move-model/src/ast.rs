// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Contains AST definitions for the specification language fragments of the Move language.
//! Note that in this crate, specs are represented in AST form, whereas code is represented
//! as bytecodes. Therefore we do not need an AST for the Move code itself.

use num::{BigInt, BigUint, Num};

use crate::{
    model::{FieldId, Loc, ModuleId, NodeId, SpecFunId, SpecVarId, StructId},
    symbol::{Symbol, SymbolPool},
    ty::Type,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fmt::{Error, Formatter},
};
use vm::file_format::CodeOffset;

use crate::{
    model::{FunId, GlobalEnv, GlobalId, QualifiedId, SchemaId, TypeParameter},
    ty::TypeDisplayContext,
};
use itertools::Itertools;
use once_cell::sync::Lazy;

// =================================================================================================
/// # Declarations

#[derive(Debug)]
pub struct SpecVarDecl {
    pub loc: Loc,
    pub name: Symbol,
    pub type_params: Vec<(Symbol, Type)>,
    pub type_: Type,
}

#[derive(Clone, Debug)]
pub struct SpecFunDecl {
    pub loc: Loc,
    pub name: Symbol,
    pub type_params: Vec<(Symbol, Type)>,
    pub params: Vec<(Symbol, Type)>,
    pub context_params: Option<Vec<(Symbol, bool)>>,
    pub result_type: Type,
    pub used_spec_vars: BTreeSet<QualifiedId<SpecVarId>>,
    pub used_memory: BTreeSet<QualifiedId<StructId>>,
    pub uninterpreted: bool,
    pub is_move_fun: bool,
    pub is_native: bool,
    pub body: Option<Exp>,
}

// =================================================================================================
/// # Conditions

#[derive(Debug, PartialEq, Clone)]
pub enum ConditionKind {
    Assert,
    Assume,
    Decreases,
    AbortsIf,
    AbortsWith,
    SucceedsIf,
    Modifies,
    Emits,
    Ensures,
    Requires,
    RequiresModule,
    Invariant,
    InvariantModule,
    InvariantUpdate,
    VarUpdate(ModuleId, SpecVarId, Vec<Type>),
    VarPack(ModuleId, SpecVarId, Vec<Type>),
    VarUnpack(ModuleId, SpecVarId, Vec<Type>),
}

impl ConditionKind {
    /// If this is an assignment to a spec var, return it.
    pub fn get_spec_var_target(&self) -> Option<(ModuleId, SpecVarId, Vec<Type>)> {
        use ConditionKind::*;
        if let VarUpdate(mid, vid, tys) | VarPack(mid, vid, tys) | VarUnpack(mid, vid, tys) = self {
            Some((*mid, *vid, tys.clone()))
        } else {
            None
        }
    }

    /// Returns true of this condition allows the `old(..)` expression.
    pub fn allows_old(&self) -> bool {
        use ConditionKind::*;
        matches!(self, Emits | Ensures | InvariantUpdate | VarUpdate(..))
    }

    /// Returns true if this condition is allowed on a public function declaration.
    pub fn allowed_on_public_fun_decl(&self) -> bool {
        use ConditionKind::*;
        matches!(
            self,
            Requires
                | RequiresModule
                | AbortsIf
                | AbortsWith
                | SucceedsIf
                | Emits
                | Ensures
                | Modifies
        )
    }

    /// Returns true if this condition is allowed on a private function declaration.
    pub fn allowed_on_private_fun_decl(&self) -> bool {
        use ConditionKind::*;
        matches!(
            self,
            Requires
                | RequiresModule
                | AbortsIf
                | AbortsWith
                | SucceedsIf
                | Emits
                | Ensures
                | Modifies
        )
    }

    /// Returns true if this condition is allowed in a function body.
    pub fn allowed_on_fun_impl(&self) -> bool {
        use ConditionKind::*;
        matches!(self, Assert | Assume | Decreases)
    }

    /// Returns true if this condition is allowed on a struct.
    pub fn allowed_on_struct(&self) -> bool {
        use ConditionKind::*;
        matches!(self, Invariant | VarPack(..) | VarUnpack(..))
    }

    /// Returns true if this condition is allowed on a module.
    pub fn allowed_on_module(&self) -> bool {
        use ConditionKind::*;
        matches!(self, Invariant | InvariantUpdate)
    }
}

impl std::fmt::Display for ConditionKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ConditionKind::*;
        match self {
            Assert => write!(f, "assert"),
            Assume => write!(f, "assume"),
            Decreases => write!(f, "decreases"),
            AbortsIf => write!(f, "aborts_if"),
            AbortsWith => write!(f, "aborts_with"),
            SucceedsIf => write!(f, "succeeds_if"),
            Modifies => write!(f, "modifies"),
            Emits => write!(f, "emits"),
            Ensures => write!(f, "ensures"),
            Requires => write!(f, "requires"),
            RequiresModule => write!(f, "requires module"),
            Invariant => write!(f, "invariant"),
            InvariantModule => write!(f, "invariant module"),
            InvariantUpdate => write!(f, "invariant update"),
            VarUpdate(..) => write!(f, "invariant update assign"),
            VarPack(..) => write!(f, "invariant pack assign"),
            VarUnpack(..) => write!(f, "invariant unpack assign"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum QuantKind {
    Forall,
    Exists,
}

impl std::fmt::Display for QuantKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use QuantKind::*;
        match self {
            Forall => write!(f, "forall"),
            Exists => write!(f, "exists"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub loc: Loc,
    pub kind: ConditionKind,
    pub properties: PropertyBag,
    pub exp: Exp,
    pub additional_exps: Vec<Exp>,
}

impl Condition {
    /// Return all expressions in the condition, the primary one and the additional ones.
    pub fn all_exps(&self) -> impl Iterator<Item = &Exp> {
        std::iter::once(&self.exp).chain(self.additional_exps.iter())
    }
}

// =================================================================================================
/// # Specifications

/// A set of properties stemming from pragmas.
pub type PropertyBag = BTreeMap<Symbol, PropertyValue>;

/// The value of a property.
#[derive(Debug, Clone)]
pub enum PropertyValue {
    Value(Value),
    Symbol(Symbol),
    QualifiedSymbol(QualifiedSymbol),
}

/// Specification and properties associated with a language item.
#[derive(Debug, Clone, Default)]
pub struct Spec {
    // The set of conditions associated with this item.
    pub conditions: Vec<Condition>,
    // Any pragma properties associated with this item.
    pub properties: PropertyBag,
    // If this is a function, specs associated with individual code points.
    pub on_impl: BTreeMap<CodeOffset, Spec>,
}

impl Spec {
    pub fn has_conditions(&self) -> bool {
        !self.conditions.is_empty()
    }

    pub fn filter<P>(&self, pred: P) -> impl Iterator<Item = &Condition>
    where
        P: FnMut(&&Condition) -> bool,
    {
        self.conditions.iter().filter(pred)
    }

    pub fn filter_kind(&self, kind: ConditionKind) -> impl Iterator<Item = &Condition> {
        self.filter(move |c| c.kind == kind)
    }

    pub fn any<P>(&self, pred: P) -> bool
    where
        P: FnMut(&Condition) -> bool,
    {
        self.conditions.iter().any(pred)
    }

    pub fn any_kind(&self, kind: ConditionKind) -> bool {
        self.any(move |c| c.kind == kind)
    }
}

/// Information about a specification block in the source. This is used for documentation
/// generation. In the object model, the original locations and documentation of spec blocks
/// is reduced to conditions on a `Spec`, with expansion of schemas. This data structure
/// allows us to discover the original spec blocks and their content.
#[derive(Debug, Clone)]
pub struct SpecBlockInfo {
    /// The location of the entire spec block.
    pub loc: Loc,
    /// The target of the spec block.
    pub target: SpecBlockTarget,
    /// The locations of all members of the spec block.
    pub member_locs: Vec<Loc>,
}

/// Describes the target of a spec block.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpecBlockTarget {
    Module,
    Struct(ModuleId, StructId),
    Function(ModuleId, FunId),
    FunctionCode(ModuleId, FunId, usize),
    Schema(ModuleId, SchemaId, Vec<TypeParameter>),
}

/// Describes a global invariant.
#[derive(Debug, Clone)]
pub struct GlobalInvariant {
    pub id: GlobalId,
    pub loc: Loc,
    pub kind: ConditionKind,
    pub mem_usage: BTreeSet<QualifiedId<StructId>>,
    pub spec_var_usage: BTreeSet<QualifiedId<SpecVarId>>,
    pub declaring_module: ModuleId,
    pub properties: PropertyBag,
    pub cond: Exp,
}

// =================================================================================================
/// # Expressions

/// The type of expressions.
///
/// Expression layout follows the following design principles:
///
/// - We try to keep the number of expression variants minimal, for easier treatment in
///   generic traversals. Builtin and user functions are abstracted into a general
///   `Call(.., operation, args)` construct.
/// - Each expression has a unique node id assigned. This id allows to build attribute tables
///   for additional information, like expression type and source location. The id is globally
///   unique.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Exp {
    /// Represents an invalid expression. This is used as a stub for algorithms which
    /// generate expressions but can fail with multiple errors, like a translator from
    /// some other source into expressions. Consumers of expressions should assume this
    /// variant is not present and can panic when seeing it.
    Invalid(NodeId),
    /// Represents a value.
    Value(NodeId, Value),
    /// Represents a reference to a local variable.
    LocalVar(NodeId, Symbol),
    /// Represents a reference to a global specification (ghost) variable.
    SpecVar(NodeId, ModuleId, SpecVarId, Option<MemoryLabel>),
    /// Represents a call to an operation. The `Operation` enum covers all builtin functions
    /// (including operators, constants, ...) as well as user functions.
    Call(NodeId, Operation, Vec<Exp>),
    /// Represents an invocation of a function value, as a lambda.
    Invoke(NodeId, Box<Exp>, Vec<Exp>),
    /// Represents a lambda.
    Lambda(NodeId, Vec<LocalVarDecl>, Box<Exp>),
    /// Represents a quantified formula over multiple variables and ranges.
    Quant(NodeId, QuantKind, Vec<(LocalVarDecl, Exp)>, Box<Exp>),
    /// Represents a block which contains a set of variable bindings and an expression
    /// for which those are defined.
    Block(NodeId, Vec<LocalVarDecl>, Box<Exp>),
    /// Represents a conditional.
    IfElse(NodeId, Box<Exp>, Box<Exp>, Box<Exp>),
}

impl Exp {
    pub fn node_id(&self) -> NodeId {
        use Exp::*;
        match self {
            Invalid(node_id)
            | Value(node_id, ..)
            | LocalVar(node_id, ..)
            | SpecVar(node_id, ..)
            | Call(node_id, ..)
            | Invoke(node_id, ..)
            | Lambda(node_id, ..)
            | Quant(node_id, ..)
            | Block(node_id, ..)
            | IfElse(node_id, ..) => *node_id,
        }
    }

    pub fn call_args(&self) -> &[Exp] {
        match self {
            Exp::Call(_, _, args) => args,
            _ => panic!("function must be called on Exp::Call(...)"),
        }
    }

    pub fn node_ids(&self) -> Vec<NodeId> {
        let mut ids = vec![];
        self.visit(&mut |e| {
            ids.push(e.node_id());
        });
        ids
    }

    /// Returns the locals used in this expression.
    pub fn locals(&self) -> BTreeSet<Symbol> {
        let mut locals = BTreeSet::new();
        let mut shadowed = vec![]; // Should be multiset but don't have this
        let mut visitor = |up: bool, e: &Exp| {
            use Exp::*;
            let decls = match e {
                Lambda(_, decls, _) | Block(_, decls, _) => decls.as_slice(),
                _ => &[],
            };
            if !up {
                shadowed.extend(decls.iter().map(|d| d.name));
            } else {
                for sym in decls.iter().map(|d| d.name) {
                    if let Some(pos) = shadowed.iter().position(|s| *s == sym) {
                        // Remove one instance of this symbol. The same symbol can appear
                        // multiple times in `shadowed`.
                        shadowed.remove(pos);
                    }
                }
                if let LocalVar(_, sym) = e {
                    if !shadowed.contains(sym) {
                        locals.insert(*sym);
                    }
                }
            }
        };
        self.visit_pre_post(&mut visitor);
        locals
    }

    /// Visits expression, calling visitor on each sub-expression, depth first.
    pub fn visit<F>(&self, visitor: &mut F)
    where
        F: FnMut(&Exp),
    {
        self.visit_pre_post(&mut |up, e| {
            if up {
                visitor(e);
            }
        });
    }

    /// Visits expression, calling visitor on each sub-expression. `visitor(false, ..)` will
    /// be called before descending into expression, and `visitor(true, ..)` after. Notice
    /// we use one function instead of two so a lambda can be passed which encapsulates mutable
    /// references.
    pub fn visit_pre_post<F>(&self, visitor: &mut F)
    where
        F: FnMut(bool, &Exp),
    {
        use Exp::*;
        visitor(false, self);
        match self {
            Call(_, _, args) => {
                for exp in args {
                    exp.visit_pre_post(visitor);
                }
            }
            Invoke(_, target, args) => {
                target.visit_pre_post(visitor);
                for exp in args {
                    exp.visit_pre_post(visitor);
                }
            }
            Lambda(_, _, body) => body.visit_pre_post(visitor),
            Quant(_, _, ranges, body) => {
                for (_, range) in ranges {
                    range.visit_pre_post(visitor);
                }
                body.visit_pre_post(visitor);
            }
            Block(_, decls, body) => {
                for decl in decls {
                    if let Some(def) = &decl.binding {
                        def.visit_pre_post(visitor);
                    }
                }
                body.visit_pre_post(visitor)
            }
            IfElse(_, c, t, e) => {
                c.visit_pre_post(visitor);
                t.visit_pre_post(visitor);
                e.visit_pre_post(visitor);
            }
            _ => {}
        }
        visitor(true, self);
    }

    /// Returns the set of module ids used by this expression.
    pub fn module_usage(&self, usage: &mut BTreeSet<ModuleId>) {
        self.visit(&mut |e| match e {
            Exp::Call(_, oper, _) => {
                use Operation::*;
                match oper {
                    Function(mid, ..) | Pack(mid, ..) | Select(mid, ..) | UpdateField(mid, ..) => {
                        usage.insert(*mid);
                    }
                    _ => {}
                }
            }
            Exp::SpecVar(_, mid, ..) => {
                usage.insert(*mid);
            }
            _ => {}
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Function(ModuleId, SpecFunId, Option<Vec<MemoryLabel>>),
    Pack(ModuleId, StructId),
    Tuple,
    Select(ModuleId, StructId, FieldId),
    UpdateField(ModuleId, StructId, FieldId),
    Local(Symbol),
    Result(usize),
    Index,
    Slice,

    // Binary operators
    Range,
    Add,
    Sub,
    Mul,
    Mod,
    Div,
    BitOr,
    BitAnd,
    Xor,
    Shl,
    Shr,
    Implies,
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,

    // Unary operators
    Not,

    // Builtin functions
    Len,
    TypeValue,
    TypeDomain,
    Global(Option<MemoryLabel>),
    Exists(Option<MemoryLabel>),
    Old,
    Trace,
    Empty,
    Single,
    Update,
    Concat,
    MaxU8,
    MaxU64,
    MaxU128,
    AbortFlag,
    AbortCode,

    // Operation with no effect
    NoOp,
}

/// A label used for referring to a specific memory in Global, Exists, and SpecVar expressions.
pub type MemoryLabel = GlobalId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalVarDecl {
    pub id: NodeId,
    pub name: Symbol,
    pub binding: Option<Exp>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value {
    Address(BigUint),
    Number(BigInt),
    Bool(bool),
    ByteArray(Vec<u8>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Value::Address(address) => write!(f, "{:x}", address),
            Value::Number(int) => write!(f, "{}", int),
            Value::Bool(b) => write!(f, "{}", b),
            // TODO(tzakian): Figure out a better story for byte array displays
            Value::ByteArray(bytes) => write!(f, "{:?}", bytes),
        }
    }
}

// =================================================================================================
/// # Purity of Expressions

impl Operation {
    /// Determines whether this operation is pure (does not depend on global state)
    pub fn is_pure<F>(&self, check_pure: &F) -> bool
    where
        F: Fn(ModuleId, SpecFunId) -> bool,
    {
        use Operation::*;
        match self {
            Exists(_) | Global(_) => false,
            Function(mid, fid, _) => check_pure(*mid, *fid),
            _ => true,
        }
    }
}

impl Exp {
    /// Determines whether this expression is pure (does not depend on global state)
    pub fn is_pure<F>(&self, check_pure: &F) -> bool
    where
        F: Fn(ModuleId, SpecFunId) -> bool,
    {
        use Exp::*;
        let mut is_pure = true;
        self.visit(&mut |exp: &Exp| match exp {
            Call(_, oper, _) => {
                is_pure = is_pure && oper.is_pure(check_pure);
            }
            SpecVar(..) => is_pure = false,
            _ => {}
        });
        is_pure
    }
}

// =================================================================================================
/// # Names

/// Represents a module name, consisting of address and name.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ModuleName(BigUint, Symbol);

impl ModuleName {
    pub fn new(addr: BigUint, name: Symbol) -> ModuleName {
        ModuleName(addr, name)
    }

    pub fn from_str(mut addr: &str, name: Symbol) -> ModuleName {
        if addr.starts_with("0x") {
            addr = &addr[2..];
        }
        let bi = BigUint::from_str_radix(addr, 16).expect("valid hex");
        ModuleName(bi, name)
    }

    pub fn addr(&self) -> &BigUint {
        &self.0
    }

    pub fn name(&self) -> Symbol {
        self.1
    }

    /// Determine whether this is a script. The move-lang infrastructure uses MAX_ADDR
    /// for pseudo modules created from scripts, so use this address to check.
    pub fn is_script(&self) -> bool {
        static MAX_ADDR: Lazy<BigUint> = Lazy::new(|| {
            BigUint::from_str_radix("ffffffffffffffffffffffffffffffff", 16).expect("valid hex")
        });
        self.0 == *MAX_ADDR
    }
}

impl ModuleName {
    /// Creates a value implementing the Display trait which shows this name,
    /// excluding address.
    pub fn display<'a>(&'a self, pool: &'a SymbolPool) -> ModuleNameDisplay<'a> {
        ModuleNameDisplay {
            name: self,
            pool,
            with_address: false,
        }
    }

    /// Creates a value implementing the Display trait which shows this name,
    /// including address.
    pub fn display_full<'a>(&'a self, pool: &'a SymbolPool) -> ModuleNameDisplay<'a> {
        ModuleNameDisplay {
            name: self,
            pool,
            with_address: true,
        }
    }
}

/// A helper to support module names in formatting.
pub struct ModuleNameDisplay<'a> {
    name: &'a ModuleName,
    pool: &'a SymbolPool,
    with_address: bool,
}

impl<'a> fmt::Display for ModuleNameDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.with_address && !self.name.is_script() {
            write!(f, "0x{}::", self.name.0.to_str_radix(16))?;
        }
        write!(f, "{}", self.name.1.display(self.pool))?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct QualifiedSymbol {
    pub module_name: ModuleName,
    pub symbol: Symbol,
}

impl QualifiedSymbol {
    /// Creates a value implementing the Display trait which shows this symbol,
    /// including module name but excluding address.
    pub fn display<'a>(&'a self, pool: &'a SymbolPool) -> QualifiedSymbolDisplay<'a> {
        QualifiedSymbolDisplay {
            sym: self,
            pool,
            with_module: true,
            with_address: false,
        }
    }

    /// Creates a value implementing the Display trait which shows this qualified symbol,
    /// excluding module name.
    pub fn display_simple<'a>(&'a self, pool: &'a SymbolPool) -> QualifiedSymbolDisplay<'a> {
        QualifiedSymbolDisplay {
            sym: self,
            pool,
            with_module: false,
            with_address: false,
        }
    }

    /// Creates a value implementing the Display trait which shows this symbol,
    /// including module name with address.
    pub fn display_full<'a>(&'a self, pool: &'a SymbolPool) -> QualifiedSymbolDisplay<'a> {
        QualifiedSymbolDisplay {
            sym: self,
            pool,
            with_module: true,
            with_address: true,
        }
    }
}

/// A helper to support qualified symbols in formatting.
pub struct QualifiedSymbolDisplay<'a> {
    sym: &'a QualifiedSymbol,
    pool: &'a SymbolPool,
    with_module: bool,
    with_address: bool,
}

impl<'a> fmt::Display for QualifiedSymbolDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if self.with_module {
            write!(
                f,
                "{}::",
                if self.with_address {
                    self.sym.module_name.display_full(self.pool)
                } else {
                    self.sym.module_name.display(self.pool)
                }
            )?;
        }
        write!(f, "{}", self.sym.symbol.display(self.pool))?;
        Ok(())
    }
}

impl Exp {
    /// Creates a display of an expression which can be used in formatting.
    pub fn display<'a>(&'a self, env: &'a GlobalEnv) -> ExpDisplay<'a> {
        ExpDisplay { env, exp: self }
    }
}

/// Helper type for expression display.
pub struct ExpDisplay<'a> {
    env: &'a GlobalEnv,
    exp: &'a Exp,
}

impl<'a> fmt::Display for ExpDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use Exp::*;
        match self.exp {
            Invalid(_) => write!(f, "*invalid*"),
            Value(_, v) => write!(f, "{}", v),
            LocalVar(_, name) => write!(f, "{}", name.display(self.env.symbol_pool())),
            SpecVar(_, mid, vid, label) => {
                let module_env = self.env.get_module(*mid);
                let spec_var = module_env.get_spec_var(*vid);
                write!(
                    f,
                    "{}::{}",
                    module_env.get_name().display(self.env.symbol_pool()),
                    spec_var.name.display(self.env.symbol_pool())
                )?;
                if let Some(label) = label {
                    write!(f, "[{}]", label)?;
                }
                Ok(())
            }
            Call(node_id, oper, args) => write!(
                f,
                "{}({})",
                oper.display(self.env, *node_id),
                args.iter()
                    .map(|e| e.display(self.env).to_string())
                    .join(", ")
            ),
            _ => {
                // TODO(wrwg): implement remaining expression forms.
                f.write_str("<can't display>")
            }
        }
    }
}

impl Operation {
    /// Creates a display of an operation which can be used in formatting.
    pub fn display<'a>(&'a self, env: &'a GlobalEnv, node_id: NodeId) -> OperationDisplay<'a> {
        OperationDisplay {
            env,
            oper: self,
            node_id,
        }
    }
}

/// Helper type for operation display.
pub struct OperationDisplay<'a> {
    env: &'a GlobalEnv,
    node_id: NodeId,
    oper: &'a Operation,
}

impl<'a> fmt::Display for OperationDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use Operation::*;
        match self.oper {
            Function(mid, fid, labels_opt) => {
                write!(f, "{}", self.fun_str(mid, fid))?;
                if let Some(labels) = labels_opt {
                    write!(
                        f,
                        "[{}]",
                        labels.iter().map(|l| format!("{}", l)).join(", ")
                    )?;
                }
                Ok(())
            }
            Global(label_opt) => {
                write!(f, "global<{}>", self.resource_access_str(self.node_id))?;
                if let Some(label) = label_opt {
                    write!(f, "[{}]", label)?
                }
                Ok(())
            }
            Exists(label_opt) => {
                write!(f, "exists<{}>", self.resource_access_str(self.node_id))?;
                if let Some(label) = label_opt {
                    write!(f, "[{}]", label)?
                }
                Ok(())
            }
            Pack(mid, sid) => write!(f, "pack {}", self.struct_str(mid, sid)),
            Select(mid, sid, fid) => {
                write!(f, "select {}", self.field_str(mid, sid, fid))
            }
            UpdateField(mid, sid, fid) => {
                write!(f, "update {}", self.field_str(mid, sid, fid))
            }
            Local(s) => write!(f, "{}", s.display(self.env.symbol_pool())),
            Result(t) => write!(f, "result{}", t),
            _ => write!(f, "{:?}", self.oper),
        }
    }
}

impl<'a> OperationDisplay<'a> {
    fn fun_str(&self, mid: &ModuleId, fid: &SpecFunId) -> String {
        let module_env = self.env.get_module(*mid);
        let fun = module_env.get_spec_fun(*fid);
        format!(
            "{}::{}",
            module_env.get_name().display(self.env.symbol_pool()),
            fun.name.display(self.env.symbol_pool()),
        )
    }

    fn struct_str(&self, mid: &ModuleId, sid: &StructId) -> String {
        let module_env = self.env.get_module(*mid);
        let struct_env = module_env.get_struct(*sid);
        format!(
            "{}::{}",
            module_env.get_name().display(self.env.symbol_pool()),
            struct_env.get_name().display(self.env.symbol_pool()),
        )
    }

    fn field_str(&self, mid: &ModuleId, sid: &StructId, fid: &FieldId) -> String {
        let struct_env = self.env.get_module(*mid).into_struct(*sid);
        let field_name = struct_env.get_field(*fid).get_name();
        format!(
            "{}.{}",
            self.struct_str(mid, sid),
            field_name.display(self.env.symbol_pool())
        )
    }

    fn resource_access_str(&self, node_id: NodeId) -> String {
        let ty = &self.env.get_node_instantiation(node_id)[0];
        let (mid, sid, targs) = ty.require_struct();
        let tctx = TypeDisplayContext::WithEnv {
            env: self.env,
            type_param_names: None,
        };
        let targs_str = if targs.is_empty() {
            "".to_string()
        } else {
            format!("<{}>", targs.iter().map(|t| t.display(&tctx)).join(", "))
        };
        format!("{}{}", self.struct_str(&mid, &sid), targs_str)
    }
}

impl fmt::Display for MemoryLabel {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "@{}", self.as_usize())
    }
}
