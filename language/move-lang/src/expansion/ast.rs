// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::unique_map::UniqueMap;
use crate::{
    parser::ast::{
        BinOp, Field, FunctionName, FunctionVisibility, Kind, ModuleIdent, ResourceLoc, StructName,
        UnaryOp, Value, Var,
    },
    shared::*,
};
use std::{collections::BTreeMap, collections::VecDeque, fmt};

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug)]
pub struct Program {
    pub modules: UniqueMap<ModuleIdent, ModuleDefinition>,
    pub main: Option<(Vec<ModuleIdent>, Address, FunctionName, Function)>,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

#[derive(Debug)]
pub struct ModuleDefinition {
    pub uses: BTreeMap<ModuleIdent, Loc>,
    pub unused_aliases: Vec<ModuleIdent>,
    pub is_source_module: bool,
    pub structs: UniqueMap<StructName, StructDefinition>,
    pub functions: UniqueMap<FunctionName, Function>,
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

pub type Fields<T> = UniqueMap<Field, (usize, T)>;

#[derive(Debug, PartialEq)]
pub struct StructDefinition {
    pub resource_opt: ResourceLoc,
    pub type_parameters: Vec<(Name, Kind)>,
    pub fields: StructFields,
}

#[derive(Debug, PartialEq)]
pub enum StructFields {
    Defined(Fields<SingleType>),
    Native(Loc),
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

#[derive(PartialEq, Debug)]
pub struct FunctionSignature {
    pub type_parameters: Vec<(Name, Kind)>,
    pub parameters: Vec<(Var, SingleType)>,
    pub return_type: Type,
}

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
    pub acquires: Vec<SingleType>,
    pub body: FunctionBody,
}

//**************************************************************************************************
// Types
//**************************************************************************************************

#[derive(Debug, PartialEq, Clone)]
pub enum ModuleAccess_ {
    Name(Name),
    ModuleAccess(ModuleIdent, Name),
}
pub type ModuleAccess = Spanned<ModuleAccess_>;

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum SingleType_ {
    Apply(ModuleAccess, Vec<SingleType>),
    Ref(bool, Box<SingleType>),
    UnresolvedError,
}
pub type SingleType = Spanned<SingleType_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Type_ {
    Unit,
    Single(SingleType),
    Multiple(Vec<SingleType>),
}
pub type Type = Spanned<Type_>;

//**************************************************************************************************
// Expressions
//**************************************************************************************************

#[derive(Debug, PartialEq)]
pub enum Assign_ {
    Var(Var),
    Unpack(ModuleAccess, Option<Vec<SingleType>>, Fields<Assign>),
}
pub type Assign = Spanned<Assign_>;
pub type AssignList = Spanned<Vec<Assign>>;

#[derive(Debug, PartialEq)]
pub enum Bind_ {
    Var(Var),
    Unpack(ModuleAccess, Option<Vec<SingleType>>, Fields<Bind>),
}
pub type Bind = Spanned<Bind_>;
pub type BindList = Spanned<Vec<Bind>>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ExpDotted_ {
    Exp(Exp),
    Dot(Box<ExpDotted>, Name),
}
pub type ExpDotted = Spanned<ExpDotted_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Exp_ {
    Value(Value),
    Move(Var),
    Copy(Var),

    Name(Name),
    GlobalCall(Name, Option<Vec<SingleType>>, Spanned<Vec<Exp>>),
    Call(ModuleAccess, Option<Vec<SingleType>>, Spanned<Vec<Exp>>),
    Pack(ModuleAccess, Option<Vec<SingleType>>, Fields<Exp>),

    IfElse(Box<Exp>, Box<Exp>, Box<Exp>),
    While(Box<Exp>, Box<Exp>),
    Loop(Box<Exp>),
    Block(Sequence),

    Assign(AssignList, Box<Exp>),
    FieldMutate(Box<ExpDotted>, Box<Exp>),
    Mutate(Box<Exp>, Box<Exp>),

    Return(Box<Exp>),
    Abort(Box<Exp>),
    Break,
    Continue,

    Dereference(Box<Exp>),
    UnaryExp(UnaryOp, Box<Exp>),
    BinopExp(Box<Exp>, BinOp, Box<Exp>),

    ExpList(Vec<Exp>),
    Unit,

    Borrow(bool, Box<Exp>),
    ExpDotted(Box<ExpDotted>),

    Annotate(Box<Exp>, Type),

    UnresolvedError,
}
pub type Exp = Spanned<Exp_>;

pub type Sequence = VecDeque<SequenceItem>;
#[derive(Debug, PartialEq)]
pub enum SequenceItem_ {
    Seq(Exp),
    Declare(BindList, Option<Type>),
    Bind(BindList, Exp),
}
pub type SequenceItem = Spanned<SequenceItem_>;

//**************************************************************************************************
// Display
//**************************************************************************************************

impl fmt::Display for ModuleAccess_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        use ModuleAccess_::*;
        match self {
            Name(n) => write!(f, "{}", n),
            ModuleAccess(m, n) => write!(f, "{}::{}", m, n),
        }
    }
}

impl fmt::Display for SingleType_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        use SingleType_::*;
        match self {
            UnresolvedError => write!(f, "_"),
            Apply(n, tys) => {
                write!(f, "{}", n)?;
                if !tys.is_empty() {
                    write!(f, "<")?;
                    write!(f, "{}", format_comma(tys))?;
                    write!(f, ">")?;
                }
                Ok(())
            }
            Ref(mut_, ty) => write!(f, "&{}{}", if *mut_ { "mut " } else { "" }, ty),
        }
    }
}
