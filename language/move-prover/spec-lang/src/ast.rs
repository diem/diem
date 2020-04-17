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

use move_lang::parser::ast::{self as PA};
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
    pub body: Option<Exp>,
}

// =================================================================================================
/// # Conditions

#[derive(Debug, PartialEq)]
pub enum SpecConditionKind {
    Assert,
    Assume,
    Decreases,
    AbortsIf,
    Ensures,
    Requires,
}

impl SpecConditionKind {
    pub fn new(kind: &PA::SpecConditionKind) -> Self {
        use SpecConditionKind::*;
        match kind {
            PA::SpecConditionKind::Assert => Assert,
            PA::SpecConditionKind::Assume => Assume,
            PA::SpecConditionKind::Decreases => Decreases,
            PA::SpecConditionKind::Ensures => Ensures,
            PA::SpecConditionKind::Requires => Requires,
            PA::SpecConditionKind::AbortsIf => AbortsIf,
            PA::SpecConditionKind::RequiresModule => unimplemented!(),
        }
    }

    pub fn on_decl(&self) -> bool {
        use SpecConditionKind::*;
        match self {
            AbortsIf | Ensures | Requires => true,
            _ => false,
        }
    }

    pub fn on_impl(&self) -> bool {
        use SpecConditionKind::*;
        match self {
            Assert | Assume | Decreases => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct Condition {
    pub loc: Loc,
    pub kind: SpecConditionKind,
    pub exp: Exp,
}

#[derive(Debug, Default)]
pub struct FunSpec {
    pub on_decl: Vec<Condition>,
    pub on_impl: BTreeMap<CodeOffset, Vec<Condition>>,
}

// =================================================================================================
/// # Invariants

#[derive(Debug, PartialEq)]
pub enum InvariantKind {
    Data,
    Update,
    Pack,
    Unpack,
    Module,
}

#[derive(Debug)]
pub struct Invariant {
    pub loc: Loc,
    pub kind: InvariantKind,
    // If this is an assignment to a spec variable, the module and var id, and instantiation.
    pub target: Option<(ModuleId, SpecVarId, Vec<Type>)>,
    pub exp: Exp,
}

// =================================================================================================
/// # Expressions

#[derive(Debug)]
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
    Global,
    Exists,
    Old,
    Update,
    Sender,
    MaxU8,
    MaxU64,
    MaxU128,
}

#[derive(Debug)]
pub struct LocalVarDecl {
    pub id: NodeId,
    pub name: Symbol,
    pub binding: Option<Exp>,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Address(BigUint),
    Number(BigUint),
    Bool(bool),
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
