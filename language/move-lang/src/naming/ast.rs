// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::unique_map::UniqueMap;
use crate::{
    expansion::ast::Fields,
    parser::ast::{
        BinOp, Field, FunctionName, FunctionVisibility, Kind, Kind_, ModuleIdent, ResourceLoc,
        StructName, UnaryOp, Value, Value_, Var,
    },
    shared::*,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
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
    pub uses: BTreeMap<ModuleIdent, Loc>,
    /// `None` if it is a library dependency
    /// `Some(order)` if it is a source file. Where `order` is the topological order/rank in the
    /// depedency graph. `order` is initialized at `0` and set in the uses pass
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
    Defined(Fields<BaseType>),
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
// Types
//**************************************************************************************************

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub enum BuiltinTypeName_ {
    // address
    Address,
    // u64
    U64,
    // bool
    Bool,
    // bytearray
    Bytearray,
}
pub type BuiltinTypeName = Spanned<BuiltinTypeName_>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum TypeName_ {
    Builtin(BuiltinTypeName),
    ModuleType(ModuleIdent, StructName),
}
pub type TypeName = Spanned<TypeName_>;

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct TParamID(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TParam {
    pub id: TParamID,
    pub debug: Name,
    pub kind: Kind,
}

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct TVar(u64);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum BaseType_ {
    Param(TParam),
    Apply(Option<Kind>, TypeName, Vec<BaseType>),
    Var(TVar),
    Anything,
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
// Expressions
//**************************************************************************************************

#[derive(Debug, PartialEq)]
pub enum Assign_ {
    Ignore,
    Var(Var),
    Unpack(
        ModuleIdent,
        StructName,
        Option<Vec<BaseType>>,
        Fields<Assign>,
    ),
}
pub type Assign = Spanned<Assign_>;
pub type AssignList = Spanned<Vec<Assign>>;

#[derive(Debug, PartialEq)]
pub enum Bind_ {
    Ignore,
    Var(Var),
    Unpack(ModuleIdent, StructName, Option<Vec<BaseType>>, Fields<Bind>),
}
pub type Bind = Spanned<Bind_>;
pub type BindList = Spanned<Vec<Bind>>;

#[derive(Debug, PartialEq)]
pub enum ExpDotted_ {
    Exp(Box<Exp>),
    Dot(Box<ExpDotted>, Field),
}
pub type ExpDotted = Spanned<ExpDotted_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum BuiltinFunction_ {
    MoveToSender(Option<BaseType>),
    MoveFrom(Option<BaseType>),
    BorrowGlobal(bool, Option<BaseType>),
    Exists(Option<BaseType>),
    Freeze(Option<BaseType>),
    /* TODO move these to a native module
     * GetHeight,
     * GetMaxGasPrice,
     * GetMaxGasUnits,
     * GetPublicKey,
     * GetSender,
     * GetSequenceNumber,
     * EmitEvent, */
}
pub type BuiltinFunction = Spanned<BuiltinFunction_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Exp_ {
    Value(Value),
    Move(Var),
    Copy(Var),
    Use(Var),

    ModuleCall(ModuleIdent, FunctionName, Option<Vec<BaseType>>, Box<Exp>),
    Builtin(BuiltinFunction, Box<Exp>),

    IfElse(Box<Exp>, Box<Exp>, Box<Exp>),
    While(Box<Exp>, Box<Exp>),
    Loop(Box<Exp>),
    Block(Sequence),

    Assign(AssignList, Box<Exp>),
    FieldMutate(ExpDotted, Box<Exp>),
    Mutate(Box<Exp>, Box<Exp>),

    Return(Box<Exp>),
    Abort(Box<Exp>),
    Break,
    Continue,

    Dereference(Box<Exp>),
    UnaryExp(UnaryOp, Box<Exp>),
    BinopExp(Box<Exp>, BinOp, Box<Exp>),

    Pack(ModuleIdent, StructName, Option<Vec<BaseType>>, Fields<Exp>),
    ExpList(Vec<Exp>),
    Unit,

    DerefBorrow(ExpDotted),
    Borrow(bool, ExpDotted),

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
// impls
//**************************************************************************************************

impl BuiltinTypeName_ {
    pub const ADDRESS: &'static str = "address";
    pub const U_64: &'static str = "u64";
    pub const BOOL: &'static str = "bool";
    pub const BYTE_ARRAY: &'static str = "bytearray";

    pub fn all_names() -> BTreeSet<&'static str> {
        let mut s = BTreeSet::new();
        s.insert(Self::ADDRESS);
        s.insert(Self::U_64);
        s.insert(Self::BOOL);
        s.insert(Self::BYTE_ARRAY);
        s
    }

    pub fn signed() -> BTreeSet<BuiltinTypeName_> {
        BTreeSet::new()
        // s.insert(BT::I64);
    }

    pub fn numeric() -> BTreeSet<BuiltinTypeName_> {
        use BuiltinTypeName_ as BT;
        let mut s = BT::signed();
        s.insert(BT::U64);
        // s.insert(BT::U256);
        s
    }

    pub fn bits() -> BTreeSet<BuiltinTypeName_> {
        use BuiltinTypeName_ as BT;
        BT::numeric()
        // s.insert(BT::U8);
    }

    pub fn ordered() -> BTreeSet<BuiltinTypeName_> {
        Self::bits()
    }

    pub fn resolve(name_str: &str) -> Option<Self> {
        use BuiltinTypeName_ as BT;
        match name_str {
            BT::ADDRESS => Some(BT::Address),
            BT::U_64 => Some(BT::U64),
            BT::BOOL => Some(BT::Bool),
            BT::BYTE_ARRAY => Some(BT::Bytearray),
            _ => None,
        }
    }

    pub fn kind(&self) -> Kind_ {
        use BuiltinTypeName_::*;
        match self {
            Address | U64 | Bool | Bytearray => Kind_::Unrestricted,
        }
    }

    pub fn tparam_constraints(&self, _loc: Loc) -> Vec<Kind> {
        use BuiltinTypeName_::*;
        // Match here to make sure this function is fixed when collections are added
        match self {
            Address | U64 | Bool | Bytearray => vec![],
        }
    }
}

impl TParamID {
    pub fn next() -> TParamID {
        TParamID(Counter::next())
    }
}

impl TVar {
    pub fn next() -> TVar {
        TVar(Counter::next())
    }
}

impl BuiltinFunction_ {
    pub const MOVE_TO_SENDER: &'static str = "move_to_sender";
    pub const MOVE_FROM: &'static str = "move_from";
    pub const BORROW_GLOBAL: &'static str = "borrow_global";
    pub const BORROW_GLOBAL_MUT: &'static str = "borrow_global_mut";
    pub const EXISTS: &'static str = "exists";
    pub const FREEZE: &'static str = "freeze";
    // pub const GET_HEIGHT: &'static str = "get_height";
    // pub const GET_MAX_GAS_PRICE: &'static str = "get_max_gas_price";
    // pub const GET_MAX_GAS_UNITS: &'static str = "get_max_gas_units";
    // pub const GET_PUBLIC_KEY: &'static str = "get_public_key";
    // pub const GET_SENDER: &'static str = "get_sender";
    // pub const GET_SEQUENCE_NUMBER: &'static str = "get_sequence_number";
    // pub const EMIT_EVENT: &'static str = "emit_event";
    pub fn all_names() -> BTreeSet<&'static str> {
        let mut s = BTreeSet::new();
        s.insert(Self::MOVE_TO_SENDER);
        s.insert(Self::MOVE_FROM);
        s.insert(Self::BORROW_GLOBAL);
        s.insert(Self::BORROW_GLOBAL_MUT);
        s.insert(Self::EXISTS);
        s.insert(Self::FREEZE);
        s
    }

    pub fn resolve(name_str: &str, arg: Option<BaseType>) -> Option<Self> {
        use BuiltinFunction_ as BF;
        match name_str {
            BF::MOVE_TO_SENDER => Some(BF::MoveToSender(arg)),
            BF::MOVE_FROM => Some(BF::MoveFrom(arg)),
            BF::BORROW_GLOBAL => Some(BF::BorrowGlobal(false, arg)),
            BF::BORROW_GLOBAL_MUT => Some(BF::BorrowGlobal(true, arg)),
            BF::EXISTS => Some(BF::Exists(arg)),
            BF::FREEZE => Some(BF::Freeze(arg)),
            _ => None,
        }
    }
}

impl BaseType_ {
    pub fn subst_format(&self, subst: &HashMap<TVar, BaseType>) -> String {
        use BaseType_::*;
        match self {
            Apply(_, n, tys) => {
                let tys_str = if !tys.is_empty() {
                    format!(
                        "<{}>",
                        format_comma(tys.iter().map(|t| t.value.subst_format(subst)))
                    )
                } else {
                    "".to_string()
                };
                format!("{}{}", n, tys_str)
            }
            Param(tp) => tp.debug.value.to_string(),
            Var(id) => match subst.get(id) {
                Some(t) => t.value.subst_format(subst),
                None => "_".to_string(),
            },
            Anything => "_".to_string(),
        }
    }

    pub fn anything(loc: Loc) -> BaseType {
        sp(loc, BaseType_::Anything)
    }

    pub fn builtin(loc: Loc, b_: BuiltinTypeName_) -> BaseType {
        let kind = sp(loc, b_.kind());
        let n = sp(loc, TypeName_::Builtin(sp(loc, b_)));
        sp(loc, BaseType_::Apply(Some(kind), n, vec![]))
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
    pub fn subst_format(&self, subst: &HashMap<TVar, BaseType>) -> String {
        use SingleType_::*;
        match self {
            Ref(mut_, ty) => format!(
                "&{}{}",
                if *mut_ { "mut " } else { "" },
                ty.value.subst_format(subst)
            ),
            Base(ty) => ty.value.subst_format(subst),
        }
    }

    pub fn base(sp!(loc, b_): BaseType) -> SingleType {
        sp(loc, SingleType_::Base(sp(loc, b_)))
    }

    pub fn anything(loc: Loc) -> SingleType {
        Self::base(BaseType_::anything(loc))
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
    pub fn subst_format(&self, subst: &HashMap<TVar, BaseType>) -> String {
        use Type_::*;
        match self {
            Unit => "()".into(),
            Single(s) => s.value.subst_format(subst),
            Multiple(ss) => {
                let inner = format_comma(ss.iter().map(|s| s.value.subst_format(subst)));
                format!("({})", inner)
            }
        }
    }

    pub fn base(b: BaseType) -> Type {
        Self::single(SingleType_::base(b))
    }

    pub fn single(sp!(loc, s_): SingleType) -> Type {
        sp(loc, Type_::Single(sp(loc, s_)))
    }

    pub fn anything(loc: Loc) -> Type {
        Self::single(SingleType_::anything(loc))
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
}

impl Value_ {
    pub fn type_(&self, loc: Loc) -> BaseType {
        use BuiltinTypeName_ as T;
        use Value_::*;
        let b_ = match self {
            Address(_) => T::Address,
            U64(_) => T::U64,
            Bool(_) => T::Bool,
            Bytearray(_) => T::Bytearray,
        };
        let kind = sp(loc, b_.kind());
        let n = sp(loc, TypeName_::Builtin(sp(loc, b_)));
        sp(loc, BaseType_::Apply(Some(kind), n, vec![]))
    }
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl fmt::Display for BuiltinTypeName_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        use BuiltinTypeName_::*;
        write!(
            f,
            "{}",
            match self {
                Address => "address",
                U64 => "u64",
                Bool => "bool",
                Bytearray => "bytearray",
            }
        )
    }
}

impl fmt::Display for TypeName_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        use TypeName_::*;
        match self {
            Builtin(b) => write!(f, "{}", b),
            ModuleType(m, n) => write!(f, "{}::{}", m, n),
        }
    }
}
