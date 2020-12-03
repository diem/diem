// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Contains AST definitions for the specification language fragments of the Move language.

use num::{BigInt, BigUint, Num};

use crate::{
    env::{FieldId, Loc, ModuleId, NodeId, SpecFunId, SpecVarId, StructId},
    symbol::{Symbol, SymbolPool},
    ty::Type,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    fmt::{Error, Formatter},
};
use vm::file_format::CodeOffset;

use crate::env::{FunId, GlobalEnv, GlobalId, QualifiedId, SchemaId, TypeParameter};
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
        matches!(self, Ensures | InvariantUpdate | VarUpdate(..))
    }

    /// Returns true if this condition is allowed on a public function declaration.
    pub fn allowed_on_public_fun_decl(&self) -> bool {
        use ConditionKind::*;
        matches!(
            self,
            Requires | RequiresModule | AbortsIf | AbortsWith | SucceedsIf | Ensures | Modifies
        )
    }

    /// Returns true if this condition is allowed on a private function declaration.
    pub fn allowed_on_private_fun_decl(&self) -> bool {
        use ConditionKind::*;
        matches!(
            self,
            Requires | RequiresModule | AbortsIf | AbortsWith | SucceedsIf | Ensures | Modifies
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

#[derive(Debug, Clone)]
pub struct Condition {
    pub loc: Loc,
    pub kind: ConditionKind,
    pub properties: PropertyBag,
    pub exp: Exp,
}

// =================================================================================================
/// # Specifications

/// A set of properties stemming from pragmas.
pub type PropertyBag = BTreeMap<Symbol, Value>;

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

#[derive(Debug, Clone, PartialEq)]
pub enum Exp {
    Error(NodeId),
    Value(NodeId, Value),
    LocalVar(NodeId, Symbol),
    SpecVar(NodeId, ModuleId, SpecVarId),
    Call(NodeId, Operation, Vec<Exp>),
    Invoke(NodeId, Box<Exp>, Vec<Exp>),
    Lambda(NodeId, Vec<LocalVarDecl>, Box<Exp>),
    Block(NodeId, Vec<LocalVarDecl>, Box<Exp>),
    IfElse(NodeId, Box<Exp>, Box<Exp>, Box<Exp>),
}

impl Exp {
    pub fn node_id(&self) -> NodeId {
        use Exp::*;
        match self {
            Error(node_id)
            | Value(node_id, ..)
            | LocalVar(node_id, ..)
            | SpecVar(node_id, ..)
            | Call(node_id, ..)
            | Invoke(node_id, ..)
            | Lambda(node_id, ..)
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

    /// Visits expression, calling visitor on each sub-expression, depth first.
    pub fn visit<F>(&self, visitor: &mut F)
    where
        F: FnMut(&Exp),
    {
        use Exp::*;
        match self {
            Call(_, _, args) => {
                for exp in args {
                    exp.visit(visitor);
                }
            }
            Invoke(_, target, args) => {
                target.visit(visitor);
                for exp in args {
                    exp.visit(visitor);
                }
            }
            Lambda(_, _, body) => body.visit(visitor),
            Block(_, decls, body) => {
                for decl in decls {
                    if let Some(def) = &decl.binding {
                        def.visit(visitor);
                    }
                }
                body.visit(visitor)
            }
            IfElse(_, c, t, e) => {
                c.visit(visitor);
                t.visit(visitor);
                e.visit(visitor);
            }
            _ => {}
        }
        visitor(self);
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

    /// Optionally extracts condition and abort code from the special `Operation::CondWithAbortCode`.
    pub fn extract_cond_and_aborts_code(&self) -> (&Exp, Option<&Exp>) {
        match self {
            Exp::Call(_, Operation::CondWithAbortCode, args) if args.len() == 2 => {
                (&args[0], Some(&args[1]))
            }
            _ => (self, None),
        }
    }

    /// Optionally extracts list of abort codes from the special `Operation::AbortCodes`.
    pub fn extract_abort_codes(&self) -> &[Exp] {
        match self {
            Exp::Call(_, Operation::AbortCodes, args) => args.as_slice(),
            _ => &[],
        }
    }

    /// Optionally extracts list of modify targets from the special `Operation::ModifyTargets`.
    pub fn extract_modify_targets(&self) -> &[Exp] {
        match self {
            Exp::Call(_, Operation::ModifyTargets, args) => args.as_slice(),
            _ => &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Function(ModuleId, SpecFunId),
    Pack(ModuleId, StructId),
    Tuple,
    Select(ModuleId, StructId, FieldId),
    UpdateField(ModuleId, StructId, FieldId),
    Local(Symbol),
    Result(usize),
    Index,
    Slice,

    // Pseudo operators for expressions which have a special treatment in the translation.
    CondWithAbortCode, // aborts_if E with C
    AbortCodes,        // aborts_with C1, ..., Cn
    ModifyTargets,     // modifies E1, ..., En

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
    All,
    Any,
    TypeValue,
    TypeDomain,
    Global,
    Exists,
    Old,
    Trace,
    Empty,
    Single,
    Update,
    Concat,
    MaxU8,
    MaxU64,
    MaxU128,

    // Operation with no effect
    NoOp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalVarDecl {
    pub id: NodeId,
    pub name: Symbol,
    pub binding: Option<Exp>,
}

#[derive(Debug, PartialEq, Clone)]
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
            Exists | Global => false,
            Function(mid, fid) => check_pure(*mid, *fid),
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
    ///
    /// Current implementation is incomplete.
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
        match self.exp {
            Exp::Value(_, Value::Number(num)) => write!(f, "{}", num)?,
            Exp::Value(_, Value::Bool(b)) => write!(f, "{}", b)?,
            Exp::Value(_, Value::Address(num)) => f.write_str(&num.to_str_radix(16))?,
            Exp::Call(_, Operation::Function(mid, fid), args) => {
                let module_env = self.env.get_module(*mid);
                let fun = module_env.get_spec_fun(*fid);
                write!(
                    f,
                    "{}::{}({})",
                    module_env.get_name().display(self.env.symbol_pool()),
                    fun.name.display(self.env.symbol_pool()),
                    args.iter()
                        .map(|e| e.display(self.env).to_string())
                        .join(", ")
                )?
            }
            _ => {
                // TODO(wrwg): implement expression printer
                f.write_str("<value>")?
            }
        }
        Ok(())
    }
}
