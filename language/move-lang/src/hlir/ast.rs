// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::unique_map::UniqueMap;
use crate::{
    naming::ast::{BuiltinTypeName_, TParam, TypeName, TypeName_},
    parser::ast::{
        BinOp, Field, FunctionName, FunctionVisibility, Kind, ModuleIdent, ResourceLoc, StructName,
        UnaryOp, Value, Var,
    },
    shared::*,
};
use std::collections::{BTreeSet, VecDeque};

// High Level IR

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug)]
pub struct Program {
    pub modules: UniqueMap<ModuleIdent, ModuleDefinition>,
    pub main: Option<(Address, FunctionName, Function)>,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

#[derive(Debug)]
pub struct ModuleDefinition {
    /// `None` if it is a library dependency
    /// `Some(order)` if it is a source file. Where `order` is the topological order/rank in the
    /// depedency graph
    pub is_source_module: Option<usize>,
    pub structs: UniqueMap<StructName, StructDefinition>,
    pub functions: UniqueMap<FunctionName, Function>,
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

#[derive(Debug, PartialEq, Clone)]
pub struct StructDefinition {
    pub resource_opt: ResourceLoc,
    pub type_parameters: Vec<TParam>,
    pub fields: StructFields,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StructFields {
    Defined(Vec<(Field, BaseType)>),
    Native(Loc),
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

#[derive(PartialEq, Debug, Clone)]
pub struct FunctionSignature {
    pub type_parameters: Vec<TParam>,
    pub parameters: Vec<(Var, SingleType)>,
    pub return_type: Type,
}

#[derive(PartialEq, Debug)]
pub enum FunctionBody_ {
    Native,
    Defined {
        locals: UniqueMap<Var, SingleType>,
        body: Block,
    },
}
pub type FunctionBody = Spanned<FunctionBody_>;

#[derive(PartialEq, Debug)]
pub struct Function {
    pub visibility: FunctionVisibility,
    pub signature: FunctionSignature,
    pub acquires: BTreeSet<BaseType>,
    pub body: FunctionBody,
}

//**************************************************************************************************
// Types
//**************************************************************************************************

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum BaseType_ {
    Param(TParam),
    Apply(Kind, TypeName, Vec<BaseType>),
    UnresolvedError,
}
pub type BaseType = Spanned<BaseType_>;

#[derive(Debug, PartialEq, Clone)]
pub enum SingleType_ {
    Base(BaseType),
    Ref(bool, BaseType),
}
pub type SingleType = Spanned<SingleType_>;

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Type_ {
    Unit,
    Single(SingleType),
    Multiple(Vec<SingleType>),
}
pub type Type = Spanned<Type_>;

//**************************************************************************************************
// Statements
//**************************************************************************************************

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Statement_ {
    Command(Command),
    IfElse {
        cond: Box<Exp>,
        if_block: Block,
        else_block: Block,
    },
    While {
        cond: Box<Exp>,
        block: Block,
    },
    Loop {
        block: Block,
        has_break: bool,
        has_return_abort: bool,
    },
}
pub type Statement = Spanned<Statement_>;

pub type Block = VecDeque<Statement>;

//**************************************************************************************************
// Commands
//**************************************************************************************************

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Command_ {
    Assign(Vec<LValue>, Box<Exp>),
    Mutate(Box<Exp>, Box<Exp>),
    Abort(Exp),
    Return(Exp),
    Break,
    Continue,
    IgnoreAndPop { pop_num: usize, exp: Exp },
}
pub type Command = Spanned<Command_>;

#[derive(Debug, PartialEq)]
pub enum LValue_ {
    Ignore,
    Var(Var, Box<SingleType>),
    Unpack(StructName, Vec<BaseType>, Vec<(Field, LValue)>),
}
pub type LValue = Spanned<LValue_>;

//**************************************************************************************************
// Expressions
//**************************************************************************************************

#[derive(Debug, PartialEq)]
pub struct ModuleCall {
    pub module: ModuleIdent,
    pub name: FunctionName,
    pub type_arguments: Vec<BaseType>,
    pub arguments: Box<Exp>,
    pub acquires: BTreeSet<BaseType>,
}

#[derive(Debug, PartialEq)]
pub enum BuiltinFunction_ {
    MoveToSender(BaseType),
    MoveFrom(BaseType),
    BorrowGlobal(bool, BaseType),
    Exists(BaseType),
}
pub type BuiltinFunction = Spanned<BuiltinFunction_>;

#[derive(Debug, PartialEq)]
pub enum UnannotatedExp_ {
    Unit,
    Value(Value),
    Move { from_user: bool, var: Var },
    Copy { from_user: bool, var: Var },

    ModuleCall(Box<ModuleCall>),
    Builtin(Box<BuiltinFunction>, Box<Exp>),
    Freeze(Box<Exp>),

    Dereference(Box<Exp>),
    UnaryExp(UnaryOp, Box<Exp>),
    BinopExp(Box<Exp>, BinOp, Box<Exp>),

    Pack(StructName, Vec<BaseType>, Vec<(Field, BaseType, Exp)>),
    ExpList(Vec<ExpListItem>),

    Borrow(bool, Box<Exp>, Field),
    BorrowLocal(bool, Var),

    UnresolvedError,
}
pub type UnannotatedExp = Spanned<UnannotatedExp_>;
#[derive(Debug, PartialEq)]
pub struct Exp {
    pub ty: Type,
    pub exp: UnannotatedExp,
}
pub fn exp(ty: Type, exp: UnannotatedExp) -> Exp {
    Exp { ty, exp }
}

#[derive(Debug, PartialEq)]
pub enum ExpListItem {
    Single(Exp, Box<SingleType>),
    Splat(Loc, Exp, Vec<SingleType>),
}

//**************************************************************************************************
// impls
//**************************************************************************************************

impl BaseType_ {
    pub fn builtin(loc: Loc, b_: BuiltinTypeName_) -> BaseType {
        let kind = sp(loc, b_.kind());
        let n = sp(loc, TypeName_::Builtin(sp(loc, b_)));
        sp(loc, BaseType_::Apply(kind, n, vec![]))
    }

    pub fn bool(loc: Loc) -> BaseType {
        Self::builtin(loc, BuiltinTypeName_::Bool)
    }

    pub fn address(loc: Loc) -> BaseType {
        Self::builtin(loc, BuiltinTypeName_::Address)
    }

    pub fn u64(loc: Loc) -> BaseType {
        Self::builtin(loc, BuiltinTypeName_::U64)
    }
}

impl SingleType_ {
    pub fn base(sp!(loc, b_): BaseType) -> SingleType {
        sp(loc, SingleType_::Base(sp(loc, b_)))
    }

    pub fn bool(loc: Loc) -> SingleType {
        Self::base(BaseType_::bool(loc))
    }

    pub fn address(loc: Loc) -> SingleType {
        Self::base(BaseType_::address(loc))
    }

    pub fn u64(loc: Loc) -> SingleType {
        Self::base(BaseType_::u64(loc))
    }
}

impl Type_ {
    pub fn base(b: BaseType) -> Type {
        Self::single(SingleType_::base(b))
    }

    pub fn single(sp!(loc, s_): SingleType) -> Type {
        sp(loc, Type_::Single(sp(loc, s_)))
    }

    pub fn bool(loc: Loc) -> Type {
        Self::single(SingleType_::bool(loc))
    }

    pub fn address(loc: Loc) -> Type {
        Self::single(SingleType_::address(loc))
    }

    pub fn u64(loc: Loc) -> Type {
        Self::single(SingleType_::u64(loc))
    }

    pub fn type_at_index(&self, idx: usize) -> &SingleType {
        match self {
            Type_::Unit => panic!("ICE type mismatch on index lookup"),
            Type_::Single(s) => {
                assert!(idx == 0);
                s
            }
            Type_::Multiple(ss) => {
                assert!(idx < ss.len());
                ss.get(idx).unwrap()
            }
        }
    }

    pub fn from_vec(loc: Loc, mut ss: Vec<SingleType>) -> Type {
        let t_ = match ss.len() {
            0 => Type_::Unit,
            1 => Type_::Single(ss.pop().unwrap()),
            _ => Type_::Multiple(ss),
        };
        sp(loc, t_)
    }
}
