// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::unique_map::UniqueMap;
use crate::{
    expansion::ast::Fields,
    naming::ast::{BaseType, FunctionSignature, SingleType, StructDefinition, Type, Type_},
    parser::ast::{
        BinOp, Field, FunctionName, FunctionVisibility, ModuleIdent, StructName, UnaryOp, Value,
        Var,
    },
    shared::*,
};
use std::{
    collections::{BTreeSet, VecDeque},
    fmt,
};

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
// Functions
//**************************************************************************************************

#[derive(PartialEq, Debug)]
pub enum FunctionBody_ {
    Defined(Sequence),
    Native,
}
pub type FunctionBody = Spanned<FunctionBody_>;

#[derive(PartialEq, Debug)]
pub struct Function {
    pub visibility: FunctionVisibility,
    pub signature: FunctionSignature,
    pub acquires: BTreeSet<StructName>,
    pub body: FunctionBody,
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Assign_ {
    Ignore,
    Var(Var, SingleType),
    Unpack(
        ModuleIdent,
        StructName,
        Vec<BaseType>,
        Fields<(BaseType, Assign)>,
    ),
    BorrowUnpack(
        bool,
        ModuleIdent,
        StructName,
        Vec<BaseType>,
        Fields<(BaseType, Assign)>,
    ),
}
pub type Assign = Spanned<Assign_>;
pub type AssignList = Spanned<Vec<Assign>>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Bind_ {
    Ignore,
    Var(Var, Option<SingleType>),
    Unpack(
        ModuleIdent,
        StructName,
        Vec<BaseType>,
        Fields<(BaseType, Bind)>,
    ),
    BorrowUnpack(
        bool,
        ModuleIdent,
        StructName,
        Vec<BaseType>,
        Fields<(BaseType, Bind)>,
    ),
}
pub type Bind = Spanned<Bind_>;
pub type BindList = Spanned<Vec<Bind>>;

#[derive(Debug, PartialEq)]
pub struct ModuleCall {
    pub module: ModuleIdent,
    pub name: FunctionName,
    pub type_arguments: Vec<BaseType>,
    pub arguments: Box<Exp>,
    pub parameter_types: Vec<SingleType>,
    pub acquires: BTreeSet<StructName>,
}

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum BuiltinFunction_ {
    MoveToSender(BaseType),
    MoveFrom(BaseType),
    BorrowGlobal(bool, BaseType),
    Exists(BaseType),
    Freeze(BaseType),
    /* GetHeight,
     * GetMaxGasPrice,
     * GetMaxGasUnits,
     * GetPublicKey,
     * GetSender,
     * GetSequenceNumber,
     * EmitEvent, */
}
pub type BuiltinFunction = Spanned<BuiltinFunction_>;

#[derive(Debug, PartialEq)]
pub enum UnannotatedExp_ {
    Unit,
    Value(Value),
    Move {
        from_user: bool,
        var: Var,
    },
    Copy {
        from_user: bool,
        var: Var,
    },
    Use(Var),

    ModuleCall(Box<ModuleCall>),
    Builtin(Box<BuiltinFunction>, Box<Exp>),

    IfElse(Box<Exp>, Box<Exp>, Box<Exp>),
    While(Box<Exp>, Box<Exp>),
    Loop {
        has_break: bool,
        body: Box<Exp>,
    },
    Block(Sequence),
    Assign(AssignList, Vec<Option<SingleType>>, Box<Exp>),
    Mutate(Box<Exp>, Box<Exp>),
    Return(Box<Exp>),
    Abort(Box<Exp>),
    Break,
    Continue,

    Dereference(Box<Exp>),
    UnaryExp(UnaryOp, Box<Exp>),
    BinopExp(Box<Exp>, BinOp, Box<Exp>),

    Pack(
        ModuleIdent,
        StructName,
        Vec<BaseType>,
        Fields<(BaseType, Exp)>,
    ),
    ExpList(Vec<ExpListItem>),

    Borrow(bool, Box<Exp>, Field),
    TempBorrow(bool, Box<Exp>),
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

pub type Sequence = VecDeque<SequenceItem>;
#[derive(Debug, PartialEq)]
pub enum SequenceItem_ {
    Seq(Box<Exp>),
    Declare(BindList),
    Bind(BindList, Vec<Option<SingleType>>, Box<Exp>),
}
pub type SequenceItem = Spanned<SequenceItem_>;

#[derive(Debug, PartialEq)]
pub enum ExpListItem {
    Single(Exp, Box<SingleType>),
    Splat(Loc, Exp, Vec<SingleType>),
}

pub fn single_item(e: Exp) -> ExpListItem {
    single_item_opt(e).expect("ICE invalid call to single_item")
}

pub fn single_item_opt(e: Exp) -> Option<ExpListItem> {
    let st = match &e.ty {
        sp!(_, Type_::Unit) | sp!(_, Type_::Multiple(_)) => return None,
        sp!(_, Type_::Single(s)) => s.clone(),
    };
    Some(ExpListItem::Single(e, Box::new(st)))
}

pub fn splat_item(splat_loc: Loc, e: Exp) -> ExpListItem {
    let ss = match &e.ty {
        sp!(_, Type_::Single(_)) => panic!("ICE invalid call to splat_item"),
        sp!(_, Type_::Unit) => vec![],
        sp!(_, Type_::Multiple(ss)) => ss.clone(),
    };
    ExpListItem::Splat(splat_loc, e, ss)
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl fmt::Display for BuiltinFunction_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        use crate::naming::ast::BuiltinFunction_ as NB;
        use BuiltinFunction_::*;
        let s = match self {
            MoveToSender(_) => NB::MOVE_TO_SENDER,
            MoveFrom(_) => NB::MOVE_FROM,
            BorrowGlobal(false, _) => NB::BORROW_GLOBAL,
            BorrowGlobal(true, _) => NB::BORROW_GLOBAL_MUT,
            Exists(_) => NB::EXISTS,
            Freeze(_) => NB::FREEZE,
        };
        write!(f, "{}", s)
    }
}
