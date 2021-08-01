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
use move_binary_format::file_format::CodeOffset;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fmt::{Error, Formatter},
};

use crate::{
    exp_rewriter::ExpRewriterFunctions,
    model::{
        EnvDisplay, FunId, FunctionVisibility, GlobalEnv, GlobalId, QualifiedInstId, SchemaId,
        TypeParameter,
    },
    ty::TypeDisplayContext,
};
use internment::LocalIntern;
use itertools::Itertools;
use once_cell::sync::Lazy;
use std::{borrow::Borrow, fmt::Debug, hash::Hash, ops::Deref};

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
    pub used_spec_vars: BTreeSet<QualifiedInstId<SpecVarId>>,
    pub used_memory: BTreeSet<QualifiedInstId<StructId>>,
    pub uninterpreted: bool,
    pub is_move_fun: bool,
    pub is_native: bool,
    pub body: Option<Exp>,
}

// =================================================================================================
/// # Conditions

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ConditionKind {
    LetPost(Symbol),
    LetPre(Symbol),
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
    StructInvariant,
    FunctionInvariant,
    LoopInvariant,
    GlobalInvariant(Vec<Type>),
    GlobalInvariantUpdate(Vec<Type>),
    SchemaInvariant(Vec<Type>),
    Axiom(Vec<Type>),
}

impl ConditionKind {
    /// Returns true of this condition allows the `old(..)` expression.
    pub fn allows_old(&self) -> bool {
        use ConditionKind::*;
        matches!(
            self,
            Assert | Assume | Emits | Ensures | GlobalInvariantUpdate(..) | LetPost(..)
        )
    }

    /// Returns true if this condition is allowed on a function declaration.
    pub fn allowed_on_fun_decl(&self, _visibility: FunctionVisibility) -> bool {
        use ConditionKind::*;
        matches!(
            self,
            Requires
                | AbortsIf
                | AbortsWith
                | SucceedsIf
                | Emits
                | Ensures
                | Modifies
                | FunctionInvariant
                | LetPost(..)
                | LetPre(..)
        )
    }

    /// Returns true if this condition is allowed in a function body.
    pub fn allowed_on_fun_impl(&self) -> bool {
        use ConditionKind::*;
        matches!(
            self,
            Assert | Assume | Decreases | LoopInvariant | LetPost(..) | LetPre(..)
        )
    }

    /// Returns true if this condition is allowed on a struct.
    pub fn allowed_on_struct(&self) -> bool {
        use ConditionKind::*;
        matches!(self, StructInvariant)
    }

    /// Returns true if this condition is allowed on a module.
    pub fn allowed_on_module(&self) -> bool {
        use ConditionKind::*;
        matches!(
            self,
            GlobalInvariant(..) | GlobalInvariantUpdate(..) | Axiom(..)
        )
    }
}

impl std::fmt::Display for ConditionKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        fn display_ty_locals(f: &mut Formatter<'_>, ty_locals: &[Type]) -> std::fmt::Result {
            if !ty_locals.is_empty() {
                write!(f, "<")?;
                for (i, ty) in ty_locals.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", ty)?;
                }
                write!(f, ">")?;
            }
            Ok(())
        }

        use ConditionKind::*;
        match self {
            LetPost(sym) => write!(f, "let({:?})", sym),
            LetPre(sym) => write!(f, "let old({:?})", sym),
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
            StructInvariant | FunctionInvariant | LoopInvariant => write!(f, "invariant"),
            GlobalInvariant(ty_locals) => {
                write!(f, "invariant")?;
                display_ty_locals(f, ty_locals)
            }
            GlobalInvariantUpdate(ty_locals) => {
                write!(f, "invariant")?;
                display_ty_locals(f, ty_locals)?;
                write!(f, " update")
            }
            SchemaInvariant(ty_locals) => {
                write!(f, "invariant")?;
                display_ty_locals(f, ty_locals)
            }
            Axiom(ty_locals) => {
                write!(f, "axiom")?;
                display_ty_locals(f, ty_locals)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
pub enum QuantKind {
    Forall,
    Exists,
    Choose,
    ChooseMin,
}

impl QuantKind {
    /// Returns true of this is a choice like Some or Min.
    pub fn is_choice(self) -> bool {
        matches!(self, QuantKind::Choose | QuantKind::ChooseMin)
    }
}

impl std::fmt::Display for QuantKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use QuantKind::*;
        match self {
            Forall => write!(f, "forall"),
            Exists => write!(f, "exists"),
            Choose => write!(f, "choose"),
            ChooseMin => write!(f, "choose min"),
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
    // The location of this specification, if available.
    pub loc: Option<Loc>,
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

    pub fn filter_kind_axiom(&self) -> impl Iterator<Item = &Condition> {
        self.filter(move |c| matches!(c.kind, ConditionKind::Axiom(..)))
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
    pub mem_usage: BTreeSet<QualifiedInstId<StructId>>,
    pub spec_var_usage: BTreeSet<QualifiedInstId<SpecVarId>>,
    pub declaring_module: ModuleId,
    pub properties: PropertyBag,
    pub cond: Exp,
}

// =================================================================================================
/// # Expressions

/// A type alias for temporaries. Those are locals used in bytecode.
pub type TempIndex = usize;

/// The type of expression data.
///
/// Expression layout follows the following design principles:
///
/// - We try to keep the number of expression variants minimal, for easier treatment in
///   generic traversals. Builtin and user functions are abstracted into a general
///   `Call(.., operation, args)` construct.
/// - Each expression has a unique node id assigned. This id allows to build attribute tables
///   for additional information, like expression type and source location. The id is globally
///   unique.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpData {
    /// Represents an invalid expression. This is used as a stub for algorithms which
    /// generate expressions but can fail with multiple errors, like a translator from
    /// some other source into expressions. Consumers of expressions should assume this
    /// variant is not present and can panic when seeing it.
    Invalid(NodeId),
    /// Represents a value.
    Value(NodeId, Value),
    /// Represents a reference to a local variable introduced by a specification construct,
    /// e.g. a quantifier.
    LocalVar(NodeId, Symbol),
    /// Represents a reference to a temporary used in bytecode.
    Temporary(NodeId, TempIndex),
    /// Represents a reference to a global specification (ghost) variable.
    SpecVar(NodeId, ModuleId, SpecVarId, Option<MemoryLabel>),
    /// Represents a call to an operation. The `Operation` enum covers all builtin functions
    /// (including operators, constants, ...) as well as user functions.
    Call(NodeId, Operation, Vec<Exp>),
    /// Represents an invocation of a function value, as a lambda.
    Invoke(NodeId, Exp, Vec<Exp>),
    /// Represents a lambda.
    Lambda(NodeId, Vec<LocalVarDecl>, Exp),
    /// Represents a quantified formula over multiple variables and ranges.
    Quant(
        NodeId,
        QuantKind,
        /// Ranges
        Vec<(LocalVarDecl, Exp)>,
        /// Triggers
        Vec<Vec<Exp>>,
        /// Optional `where` clause
        Option<Exp>,
        // Body
        Exp,
    ),
    /// Represents a block which contains a set of variable bindings and an expression
    /// for which those are defined.
    Block(NodeId, Vec<LocalVarDecl>, Exp),
    /// Represents a conditional.
    IfElse(NodeId, Exp, Exp, Exp),
}

/// An internalized expression. We do use a wrapper around the underlying internement implementation
/// variant to ensure a unique API (LocalIntern and ArcIntern e.g. differ in the presence of
/// the Copy trait, and by wrapping we effectively remove the Copy from LocalIntern).
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Exp {
    data: LocalIntern<ExpData>,
}

impl AsRef<ExpData> for Exp {
    fn as_ref(&self) -> &ExpData {
        self.data.as_ref()
    }
}

impl Borrow<ExpData> for Exp {
    fn borrow(&self) -> &ExpData {
        self.as_ref()
    }
}

impl Deref for Exp {
    type Target = ExpData;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Debug for Exp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

impl From<ExpData> for Exp {
    fn from(data: ExpData) -> Self {
        Exp {
            data: LocalIntern::new(data),
        }
    }
}

impl From<Exp> for ExpData {
    /// Takes an expression and returns expression data.
    fn from(exp: Exp) -> ExpData {
        exp.as_ref().to_owned()
    }
}

impl ExpData {
    /// Version of `into` which does not require type annotations.
    pub fn into_exp(self) -> Exp {
        self.into()
    }

    pub fn ptr_eq(e1: &Exp, e2: &Exp) -> bool {
        // For the internement based implementations, we can just test equality. Other
        // representations may need different measures.
        e1 == e2
    }

    pub fn node_id(&self) -> NodeId {
        use ExpData::*;
        match self {
            Invalid(node_id)
            | Value(node_id, ..)
            | LocalVar(node_id, ..)
            | Temporary(node_id, ..)
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
            ExpData::Call(_, _, args) => args,
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

    /// Returns the free local variables, inclusive their types, used in this expression.
    /// Result is ordered by occurrence.
    pub fn free_vars(&self, env: &GlobalEnv) -> Vec<(Symbol, Type)> {
        let mut vars = vec![];
        let mut shadowed = vec![]; // Should be multiset but don't have this
        let mut visitor = |up: bool, e: &ExpData| {
            use ExpData::*;
            let decls = match e {
                Lambda(_, decls, _) | Block(_, decls, _) => {
                    decls.iter().map(|d| d.name).collect_vec()
                }
                Quant(_, _, decls, ..) => decls.iter().map(|(d, _)| d.name).collect_vec(),
                _ => vec![],
            };
            if !up {
                shadowed.extend(decls.iter());
            } else {
                for sym in decls {
                    if let Some(pos) = shadowed.iter().position(|s| *s == sym) {
                        // Remove one instance of this symbol. The same symbol can appear
                        // multiple times in `shadowed`.
                        shadowed.remove(pos);
                    }
                }
                if let LocalVar(id, sym) = e {
                    if !shadowed.contains(sym) && !vars.iter().any(|(s, _)| s == sym) {
                        vars.push((*sym, env.get_node_type(*id)));
                    }
                }
            }
        };
        self.visit_pre_post(&mut visitor);
        vars
    }

    /// Returns the used memory of this expression.
    pub fn used_memory(
        &self,
        env: &GlobalEnv,
    ) -> BTreeSet<(QualifiedInstId<StructId>, Option<MemoryLabel>)> {
        let mut result = BTreeSet::new();
        let mut visitor = |e: &ExpData| {
            use ExpData::*;
            use Operation::*;
            match e {
                Call(id, Exists(label), _) | Call(id, Global(label), _) => {
                    let inst = &env.get_node_instantiation(*id);
                    let (mid, sid, sinst) = inst[0].require_struct();
                    result.insert((mid.qualified_inst(sid, sinst.to_owned()), label.to_owned()));
                }
                Call(id, Function(mid, fid, labels), _) => {
                    let inst = &env.get_node_instantiation(*id);
                    let module = env.get_module(*mid);
                    let fun = module.get_spec_fun(*fid);
                    for (i, mem) in fun.used_memory.iter().enumerate() {
                        result.insert((
                            mem.to_owned().instantiate(inst),
                            labels.as_ref().map(|l| l[i]),
                        ));
                    }
                }
                _ => {}
            }
        };
        self.visit(&mut visitor);
        result
    }

    /// Returns the temporaries used in this expression. Result is ordered by occurrence.
    pub fn temporaries(&self, env: &GlobalEnv) -> Vec<(TempIndex, Type)> {
        let mut temps = vec![];
        let mut visitor = |e: &ExpData| {
            if let ExpData::Temporary(id, idx) = e {
                if !temps.iter().any(|(i, _)| i == idx) {
                    temps.push((*idx, env.get_node_type(*id)));
                }
            }
        };
        self.visit(&mut visitor);
        temps
    }

    /// Visits expression, calling visitor on each sub-expression, depth first.
    pub fn visit<F>(&self, visitor: &mut F)
    where
        F: FnMut(&ExpData),
    {
        self.visit_pre_post(&mut |up, e| {
            if up {
                visitor(e);
            }
        });
    }

    pub fn any<P>(&self, predicate: &mut P) -> bool
    where
        P: FnMut(&ExpData) -> bool,
    {
        let mut found = false;
        self.visit(&mut |e| {
            if !found {
                // This still continues to visit after a match is found, may want to
                // optimize if it becomes an issue.
                found = predicate(e)
            }
        });
        found
    }

    /// Visits expression, calling visitor on each sub-expression. `visitor(false, ..)` will
    /// be called before descending into expression, and `visitor(true, ..)` after. Notice
    /// we use one function instead of two so a lambda can be passed which encapsulates mutable
    /// references.
    pub fn visit_pre_post<F>(&self, visitor: &mut F)
    where
        F: FnMut(bool, &ExpData),
    {
        use ExpData::*;
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
            Quant(_, _, ranges, triggers, condition, body) => {
                for (decl, range) in ranges {
                    if let Some(binding) = &decl.binding {
                        binding.visit_pre_post(visitor);
                    }
                    range.visit_pre_post(visitor);
                }
                for trigger in triggers {
                    for e in trigger {
                        e.visit_pre_post(visitor);
                    }
                }
                if let Some(exp) = condition {
                    exp.visit_pre_post(visitor);
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

    /// Rewrites this expression and sub-expression based on the rewriter function. The
    /// function returns `Ok(e)` if the expression is rewritten, and passes back ownership
    /// using `Err(e)` if the expression stays unchanged. This function stops traversing
    ///on `Ok(e)` and descents into sub-expressions on `Err(e)`.
    pub fn rewrite<F>(exp: Exp, exp_rewriter: &mut F) -> Exp
    where
        F: FnMut(Exp) -> Result<Exp, Exp>,
    {
        ExpRewriter {
            exp_rewriter,
            node_rewriter: &mut |_| None,
        }
        .rewrite_exp(exp)
    }

    /// Rewrites the node ids in the expression. This is used to rewrite types of
    /// expressions.
    pub fn rewrite_node_id<F>(exp: Exp, node_rewriter: &mut F) -> Exp
    where
        F: FnMut(NodeId) -> Option<NodeId>,
    {
        ExpRewriter {
            exp_rewriter: &mut Err,
            node_rewriter,
        }
        .rewrite_exp(exp)
    }

    /// Rewrites the expression and for unchanged sub-expressions, the node ids in the expression
    pub fn rewrite_exp_and_node_id<F, G>(
        exp: Exp,
        exp_rewriter: &mut F,
        node_rewriter: &mut G,
    ) -> Exp
    where
        F: FnMut(Exp) -> Result<Exp, Exp>,
        G: FnMut(NodeId) -> Option<NodeId>,
    {
        ExpRewriter {
            exp_rewriter,
            node_rewriter,
        }
        .rewrite_exp(exp)
    }

    /// A function which can be used for `Exp::rewrite_node_id` to instantiate types in
    /// an expression based on a type parameter instantiation.
    pub fn instantiate_node(env: &GlobalEnv, id: NodeId, targs: &[Type]) -> Option<NodeId> {
        if targs.is_empty() {
            // shortcut
            return None;
        }
        let node_ty = env.get_node_type(id);
        let new_node_ty = node_ty.instantiate(targs);
        let node_inst = env.get_node_instantiation_opt(id);
        let new_node_inst = node_inst.clone().map(|i| Type::instantiate_vec(i, targs));
        if node_ty != new_node_ty || node_inst != new_node_inst {
            let loc = env.get_node_loc(id);
            let new_id = env.new_node(loc, new_node_ty);
            if let Some(inst) = new_node_inst {
                env.set_node_instantiation(new_id, inst);
            }
            Some(new_id)
        } else {
            None
        }
    }

    /// Returns the set of module ids used by this expression.
    pub fn module_usage(&self, usage: &mut BTreeSet<ModuleId>) {
        self.visit(&mut |e| match e {
            ExpData::Call(_, oper, _) => {
                use Operation::*;
                match oper {
                    Function(mid, ..) | Pack(mid, ..) | Select(mid, ..) | UpdateField(mid, ..) => {
                        usage.insert(*mid);
                    }
                    _ => {}
                }
            }
            ExpData::SpecVar(_, mid, ..) => {
                usage.insert(*mid);
            }
            _ => {}
        });
    }
}

struct ExpRewriter<'a> {
    exp_rewriter: &'a mut dyn FnMut(Exp) -> Result<Exp, Exp>,
    node_rewriter: &'a mut dyn FnMut(NodeId) -> Option<NodeId>,
}

impl<'a> ExpRewriterFunctions for ExpRewriter<'a> {
    fn rewrite_exp(&mut self, exp: Exp) -> Exp {
        match (*self.exp_rewriter)(exp) {
            Ok(new_exp) => new_exp,
            Err(old_exp) => self.rewrite_exp_descent(old_exp),
        }
    }

    fn rewrite_node_id(&mut self, id: NodeId) -> Option<NodeId> {
        (*self.node_rewriter)(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operation {
    Function(ModuleId, SpecFunId, Option<Vec<MemoryLabel>>),
    Pack(ModuleId, StructId),
    Tuple,
    Select(ModuleId, StructId, FieldId),
    UpdateField(ModuleId, StructId, FieldId),
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
    Iff,
    And,
    Or,
    Eq,
    Identical,
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
    ResourceDomain,
    Global(Option<MemoryLabel>),
    Exists(Option<MemoryLabel>),
    CanModify,
    Old,
    Trace,
    EmptyVec,
    SingleVec,
    UpdateVec,
    ConcatVec,
    IndexOfVec,
    ContainsVec,
    InRangeRange,
    InRangeVec,
    RangeVec,
    MaxU8,
    MaxU64,
    MaxU128,

    // Functions which support the transformation and translation process.
    AbortFlag,
    AbortCode,
    WellFormed,
    BoxValue,
    UnboxValue,
    EmptyEventStore,
    ExtendEventStore,
    EventStoreIncludes,
    EventStoreIncludedIn,

    // Operation with no effect
    NoOp,
}

/// A label used for referring to a specific memory in Global, Exists, and SpecVar expressions.
pub type MemoryLabel = GlobalId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocalVarDecl {
    pub id: NodeId,
    pub name: Symbol,
    pub binding: Option<Exp>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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
    /// Determines whether this operation depends on global memory
    pub fn uses_memory<F>(&self, check_pure: &F) -> bool
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

impl ExpData {
    /// Determines whether this expression depends on global memory
    pub fn uses_memory<F>(&self, check_pure: &F) -> bool
    where
        F: Fn(ModuleId, SpecFunId) -> bool,
    {
        use ExpData::*;
        let mut no_use = true;
        self.visit(&mut |exp: &ExpData| match exp {
            Call(_, oper, _) => {
                no_use = no_use && oper.uses_memory(check_pure);
            }
            SpecVar(..) => no_use = false,
            _ => {}
        });
        no_use
    }
}

impl ExpData {
    /// Checks whether the expression is pure, i.e. does not depend on memory or mutable
    /// variables.
    pub fn is_pure(&self, env: &GlobalEnv) -> bool {
        let mut is_pure = true;
        let mut visitor = |e: &ExpData| {
            use ExpData::*;
            use Operation::*;
            match e {
                Temporary(id, _) => {
                    if env.get_node_type(*id).is_mutable_reference() {
                        is_pure = false;
                    }
                }
                Call(_, oper, _) => match oper {
                    Exists(..) | Global(..) => is_pure = false,
                    Function(mid, fid, _) => {
                        let module = env.get_module(*mid);
                        let fun = module.get_spec_fun(*fid);
                        if !fun.used_memory.is_empty() || !fun.used_spec_vars.is_empty() {
                            is_pure = false;
                        }
                    }
                    _ => {}
                },
                SpecVar(..) => is_pure = false,
                _ => {}
            }
        };
        self.visit(&mut visitor);
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

    pub fn from_address_bytes_and_name(
        addr: move_lang::shared::AddressBytes,
        name: Symbol,
    ) -> ModuleName {
        ModuleName(BigUint::from_bytes_be(&addr.into_bytes()), name)
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

impl ExpData {
    /// Creates a display of an expression which can be used in formatting.
    pub fn display<'a>(&'a self, env: &'a GlobalEnv) -> ExpDisplay<'a> {
        ExpDisplay { env, exp: self }
    }
}

/// Helper type for expression display.
pub struct ExpDisplay<'a> {
    env: &'a GlobalEnv,
    exp: &'a ExpData,
}

impl<'a> fmt::Display for ExpDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use ExpData::*;
        match self.exp {
            Invalid(_) => write!(f, "*invalid*"),
            Value(_, v) => write!(f, "{}", v),
            LocalVar(_, name) => write!(f, "{}", name.display(self.env.symbol_pool())),
            Temporary(_, idx) => write!(f, "$t{}", idx),
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
            Call(node_id, oper, args) => {
                write!(
                    f,
                    "{}({})",
                    oper.display(self.env, *node_id),
                    self.fmt_exps(args)
                )
            }
            Lambda(_, decls, body) => {
                write!(f, "|{}| {}", self.fmt_decls(decls), body.display(self.env))
            }
            Block(_, decls, body) => {
                write!(
                    f,
                    "{{let {}; {}}}",
                    self.fmt_decls(decls),
                    body.display(self.env)
                )
            }
            Quant(_, kind, decls, triggers, opt_where, body) => {
                let triggers_str = triggers
                    .iter()
                    .map(|trigger| format!("{{{}}}", self.fmt_exps(trigger)))
                    .collect_vec()
                    .join("");
                let where_str = if let Some(exp) = opt_where {
                    format!(" where {}", exp.display(self.env))
                } else {
                    "".to_string()
                };
                write!(
                    f,
                    "{} {}{}{}: {}",
                    kind,
                    self.fmt_quant_decls(decls),
                    triggers_str,
                    where_str,
                    body.display(self.env)
                )
            }
            Invoke(_, fun, args) => {
                write!(f, "({})({})", fun.display(self.env), self.fmt_exps(args))
            }
            IfElse(_, cond, if_exp, else_exp) => {
                write!(
                    f,
                    "(if {} {{{}}} else {{{}}})",
                    cond.display(self.env),
                    if_exp.display(self.env),
                    else_exp.display(self.env)
                )
            }
        }
    }
}

impl<'a> ExpDisplay<'a> {
    fn fmt_decls(&self, decls: &[LocalVarDecl]) -> String {
        decls
            .iter()
            .map(|decl| {
                let binding = if let Some(exp) = &decl.binding {
                    format!(" = {}", exp.display(self.env))
                } else {
                    "".to_string()
                };
                format!("{}{}", decl.name.display(self.env.symbol_pool()), binding)
            })
            .join(", ")
    }

    fn fmt_quant_decls(&self, decls: &[(LocalVarDecl, Exp)]) -> String {
        decls
            .iter()
            .map(|(decl, domain)| {
                format!(
                    "{}: {}",
                    decl.name.display(self.env.symbol_pool()),
                    domain.display(self.env)
                )
            })
            .join(", ")
    }

    fn fmt_exps(&self, exps: &[Exp]) -> String {
        exps.iter()
            .map(|e| e.display(self.env).to_string())
            .join(", ")
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
                write!(f, "global")?;
                if let Some(label) = label_opt {
                    write!(f, "[{}]", label)?
                }
                Ok(())
            }
            Exists(label_opt) => {
                write!(f, "exists")?;
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
            Result(t) => write!(f, "result{}", t),
            _ => write!(f, "{:?}", self.oper),
        }?;

        // If operation has a type instantiation, add it.
        let type_inst = self.env.get_node_instantiation(self.node_id);
        if !type_inst.is_empty() {
            let tctx = TypeDisplayContext::WithEnv {
                env: self.env,
                type_param_names: None,
            };
            write!(
                f,
                "<{}>",
                type_inst.iter().map(|ty| ty.display(&tctx)).join(", ")
            )?;
        }
        Ok(())
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
}

impl fmt::Display for MemoryLabel {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "@{}", self.as_usize())
    }
}

impl<'a> fmt::Display for EnvDisplay<'a, Condition> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.val.kind {
            ConditionKind::LetPre(name) => write!(
                f,
                "let {} = {};",
                name.display(self.env.symbol_pool()),
                self.val.exp.display(self.env)
            )?,
            ConditionKind::LetPost(name) => write!(
                f,
                "let post {} = {};",
                name.display(self.env.symbol_pool()),
                self.val.exp.display(self.env)
            )?,
            ConditionKind::Emits => {
                let exps = self.val.all_exps().collect_vec();
                write!(
                    f,
                    "emit {} to {}",
                    exps[0].display(self.env),
                    exps[1].display(self.env)
                )?;
                if exps.len() > 2 {
                    write!(f, "if {}", exps[2].display(self.env))?;
                }
                write!(f, ";")?
            }
            _ => write!(f, "{} {};", self.val.kind, self.val.exp.display(self.env))?,
        }
        Ok(())
    }
}

impl<'a> fmt::Display for EnvDisplay<'a, Spec> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "spec {{")?;
        for cond in &self.val.conditions {
            writeln!(f, "  {}", self.env.display(cond))?
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}
