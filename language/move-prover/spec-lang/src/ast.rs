// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Contains AST definitions for the specification language fragments of the Move language.

use num::{BigUint, Num};

use crate::{
    env::{FieldId, Loc, ModuleId, NodeId, SpecFunId, SpecVarId, StructId},
    symbol::{Symbol, SymbolPool},
    ty::Type,
};
use std::{
    collections::BTreeMap,
    fmt,
    fmt::{Error, Formatter},
};
use vm::file_format::CodeOffset;

use crate::env::{FunId, SchemaId, TypeParameter};
use std::collections::BTreeSet;

// =================================================================================================
/// # Declarations

#[derive(Debug)]
pub struct SpecVarDecl {
    pub loc: Loc,
    pub name: Symbol,
    pub type_params: Vec<(Symbol, Type)>,
    pub type_: Type,
}

#[derive(Debug)]
pub struct SpecFunDecl {
    pub loc: Loc,
    pub name: Symbol,
    pub type_params: Vec<(Symbol, Type)>,
    pub params: Vec<(Symbol, Type)>,
    pub result_type: Type,
    pub used_spec_vars: BTreeSet<(ModuleId, SpecVarId)>,
    pub is_pure: bool,
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
        matches!(self, Ensures | AbortsIf | InvariantUpdate | VarUpdate(..))
    }

    /// Returns true if this condition is allowed on a public function declaration.
    pub fn allowed_on_public_fun_decl(&self) -> bool {
        use ConditionKind::*;
        matches!(self, Requires | RequiresModule | AbortsIf | Ensures)
    }

    /// Returns true if this condition is allowed on a private function declaration.
    pub fn allowed_on_private_fun_decl(&self) -> bool {
        use ConditionKind::*;
        matches!(self, Requires | AbortsIf | Ensures)
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
        matches!(self, Invariant)
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
/// allows us to disover the original spec blocks and their content.
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
            Block(_, _, body) => body.visit(visitor),
            IfElse(_, c, t, e) => {
                c.visit(visitor);
                t.visit(visitor);
                e.visit(visitor);
            }
            _ => {}
        }
        visitor(self);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Function(ModuleId, SpecFunId),
    Pack(ModuleId, StructId),
    Tuple,
    Select(ModuleId, StructId, FieldId),
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
    All,
    Any,
    TypeValue,
    TypeDomain,
    Global,
    Exists,
    Old,
    Update,
    Sender,
    MaxU8,
    MaxU64,
    MaxU128,
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
    Number(BigUint),
    Bool(bool),
    ByteArray(Vec<u8>),
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
            Sender | Exists | Global => false,
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
        if self.with_address {
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
