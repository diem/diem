// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::{sp, Address, Identifier, Loc, Name, Spanned, TName};
use std::fmt;

macro_rules! new_name {
    ($n:ident) => {
        #[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
        pub struct $n(pub Name);

        impl TName for $n {
            type Key = String;
            type Loc = Loc;

            fn drop_loc(self) -> (Loc, String) {
                (self.0.loc, self.0.value)
            }

            fn clone_drop_loc(&self) -> (Loc, String) {
                (self.0.loc, self.0.value.clone())
            }

            fn add_loc(loc: Loc, key: String) -> Self {
                $n(sp(loc, key))
            }
        }

        impl Identifier for $n {
            fn value(&self) -> &str {
                &self.0.value
            }
            fn loc(&self) -> Loc {
                self.0.loc
            }
        }

        impl fmt::Display for $n {
            fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", &self.0)
            }
        }
    };
}

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug)]
pub struct Program {
    pub source_definitions: Vec<FileDefinition>,
    pub lib_definitions: Vec<FileDefinition>,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum FileDefinition {
    Modules(Vec<ModuleOrAddress>),
    Main(Main),
}

#[derive(Debug)]
pub enum ModuleOrAddress {
    Module(ModuleDefinition),
    Address(Loc, Address),
}

#[derive(Debug)]
pub struct Main {
    pub uses: Vec<(ModuleIdent, Option<ModuleName>)>,
    pub function: Function,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

new_name!(ModuleName);

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct ModuleIdent_ {
    pub name: ModuleName,
    pub address: Address,
}
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct ModuleIdent(pub Spanned<ModuleIdent_>);

#[derive(Debug)]
pub struct ModuleDefinition {
    pub uses: Vec<(ModuleIdent, Option<ModuleName>)>,
    pub name: ModuleName,
    pub structs: Vec<StructDefinition>,
    pub functions: Vec<Function>,
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

new_name!(Field);
new_name!(StructName);

pub type ResourceLoc = Option<Loc>;

#[derive(Debug, PartialEq)]
pub struct StructDefinition {
    pub resource_opt: ResourceLoc,
    pub name: StructName,
    pub type_parameters: Vec<(Name, Kind)>,
    pub fields: StructFields,
}

#[derive(Debug, PartialEq)]
pub enum StructFields {
    Defined(Vec<(Field, SingleType)>),
    Native(Loc),
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

new_name!(FunctionName);

#[derive(PartialEq, Debug)]
pub struct FunctionSignature {
    pub type_parameters: Vec<(Name, Kind)>,
    pub parameters: Vec<(Var, SingleType)>,
    pub return_type: Type,
}

#[derive(PartialEq, Debug, Clone)]
pub enum FunctionVisibility {
    Public(Loc),
    Internal,
}

#[derive(PartialEq, Debug)]
pub enum FunctionBody_ {
    Defined(Sequence),
    Native,
}
pub type FunctionBody = Spanned<FunctionBody_>;

#[derive(PartialEq, Debug)]
// (public?) foo<T1(: copyable?), ..., TN(: copyable?)>(x1: t1, ..., xn: tn): t1 * ... * tn {
//    body
//  }
// (public?) native foo<T1(: copyable?), ..., TN(: copyable?)>(x1: t1, ..., xn: tn): t1 * ... * tn;
pub struct Function {
    pub visibility: FunctionVisibility,
    pub signature: FunctionSignature,
    pub acquires: Vec<SingleType>,
    pub name: FunctionName,
    pub body: FunctionBody,
}

//**************************************************************************************************
// Types
//**************************************************************************************************

// A ModuleAccess references something from a module, either a struct or a function.
#[derive(Debug, PartialEq)]
pub enum ModuleAccess_ {
    // N
    Name(Name),
    // M.S
    ModuleAccess(ModuleName, Name),
    // OxADDR.M.S
    QualifiedModuleAccess(ModuleIdent, Name),
}
pub type ModuleAccess = Spanned<ModuleAccess_>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Kind_ {
    // Kind representing all types
    Unknown,
    // Linear resource types
    Resource,
    // Explicitly copyable types
    Affine,
    // Implicitly copyable types
    Unrestricted,
}
pub type Kind = Spanned<Kind_>;

#[derive(Debug, PartialEq)]
pub enum SingleType_ {
    // N
    // N<t1, ... , tn>
    Apply(ModuleAccess, Vec<SingleType>),
    // &t
    // &mut t
    Ref(bool, Box<SingleType>),
}
pub type SingleType = Spanned<SingleType_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Type_ {
    // ()
    Unit,
    // t
    Single(SingleType),
    // (t1, t2, ... , tn)
    // Used for return values and expression blocks
    Multiple(Vec<SingleType>),
}
pub type Type = Spanned<Type_>;

//**************************************************************************************************
// Expressions
//**************************************************************************************************

new_name!(Var);

#[derive(Debug, PartialEq)]
pub enum Bind_ {
    // x
    Var(Var),
    // T { f1: b1, ... fn: bn }
    // T<t1, ... , tn> { f1: b1, ... fn: bn }
    Unpack(ModuleAccess, Option<Vec<SingleType>>, Vec<(Field, Bind)>),
}
pub type Bind = Spanned<Bind_>;
// b1, ..., bn
pub type BindList = Spanned<Vec<Bind>>;

#[derive(Debug, PartialEq)]
pub enum Value_ {
    // 0x<hex representation up to 64 digits with padding 0s>
    Address(Address),
    U64(u64),
    // true
    // false
    Bool(bool),
    Bytearray(Vec<u8>),
}
pub type Value = Spanned<Value_>;

#[derive(Debug, PartialEq)]
pub enum UnaryOp_ {
    // !
    Not,
    // -
    Neg,
}
pub type UnaryOp = Spanned<UnaryOp_>;

#[derive(Debug, PartialEq)]
pub enum BinOp_ {
    // Int ops
    // +
    Add,
    // -
    Sub,
    // *
    Mul,
    // %
    Mod,
    // /
    Div,
    // |
    BitOr,
    // &
    BitAnd,
    // ^
    Xor,

    // Bool ops
    // &&
    And,
    // ||
    Or,

    // Compare Ops
    // ==
    Eq,
    // !=
    Neq,
    // <
    Lt,
    // >
    Gt,
    // <=
    Le,
    // >=
    Ge,
}
pub type BinOp = Spanned<BinOp_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Exp_ {
    Value(Value),
    // move(x)
    Move(Var),
    // copy(x)
    Copy(Var),
    // n
    Name(Name),
    // .n(e)
    GlobalCall(Name, Option<Vec<SingleType>>, Spanned<Vec<Exp>>),

    // f(earg,*)
    Call(ModuleAccess, Option<Vec<SingleType>>, Spanned<Vec<Exp>>),

    // tn {f1: e1, ... , f_n: e_n }
    Pack(ModuleAccess, Option<Vec<SingleType>>, Vec<(Field, Exp)>),

    // if (eb) et else ef
    IfElse(Box<Exp>, Box<Exp>, Option<Box<Exp>>),
    // while (eb) eloop
    While(Box<Exp>, Box<Exp>),
    // loop eloop
    Loop(Box<Exp>),

    // { seq }
    Block(Sequence),
    // (e1, ..., en)
    ExpList(Vec<Exp>),
    // ()
    Unit,

    // a = e
    Assign(Box<Exp>, Box<Exp>),

    // return e
    Return(Box<Exp>),
    // abort e
    Abort(Box<Exp>),
    // break
    Break,
    // continue
    Continue,

    // *e
    Dereference(Box<Exp>),
    // op e
    UnaryExp(UnaryOp, Box<Exp>),
    // e1 op e2
    BinopExp(Box<Exp>, BinOp, Box<Exp>),

    // &e
    // &mut e
    Borrow(bool, Box<Exp>),
    // e.f
    Dot(Box<Exp>, Name),

    // (e: t)
    Annotate(Box<Exp>, Type),

    // Internal node marking an error was added to the error list
    // This is here so the pass can continue even when an error is hit
    UnresolvedError,
}
pub type Exp = Spanned<Exp_>;

// { e1; ... ; en }
// { e1; ... ; en; }
pub type Sequence = (Vec<SequenceItem>, Box<Option<Exp>>);
#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum SequenceItem_ {
    // e;
    Seq(Box<Exp>),
    // let b : t = e;
    // let b = e;
    Declare(BindList, Option<Type>),
    // let b : t = e;
    // let b = e;
    Bind(BindList, Option<Type>, Box<Exp>),
}
pub type SequenceItem = Spanned<SequenceItem_>;

//**************************************************************************************************
// Loc
//**************************************************************************************************

impl TName for ModuleIdent {
    type Key = (Address, String);
    type Loc = (Loc, Loc);
    fn drop_loc(self) -> ((Loc, Loc), (Address, String)) {
        let inner = self.0.value;
        let (nloc, name_) = inner.name.drop_loc();
        ((self.0.loc, nloc), (inner.address, name_))
    }
    fn clone_drop_loc(&self) -> ((Loc, Loc), (Address, String)) {
        let (nloc, name_) = self.0.value.name.clone_drop_loc();
        ((self.0.loc, nloc), (self.0.value.address, name_))
    }
    fn add_loc(locs: (Loc, Loc), key: (Address, String)) -> ModuleIdent {
        let (iloc, nloc) = locs;
        let (address, name_str) = key;
        let name = ModuleName::add_loc(nloc, name_str);
        let ident_ = ModuleIdent_ { address, name };
        ModuleIdent(sp(iloc, ident_))
    }
}

//**************************************************************************************************
// Impl
//**************************************************************************************************

impl ModuleIdent {
    pub fn loc(&self) -> Loc {
        self.0.loc
    }
}

impl ModuleName {
    pub const SELF_NAME: &'static str = "Self";
}

impl FunctionName {
    pub const MAIN_NAME: &'static str = "main";
}

impl Var {
    pub fn starts_with_underscore(&self) -> bool {
        self.0.value.starts_with('_')
    }
}

impl Kind_ {
    pub const VALUE_CONSTRAINT: &'static str = "copyable";
    pub const RESOURCE_CONSTRAINT: &'static str = "resource";

    pub fn is_resourceful(&self) -> bool {
        match self {
            Kind_::Affine | Kind_::Unrestricted => false,
            Kind_::Resource | Kind_::Unknown => true,
        }
    }
}

impl Type_ {
    pub fn unit(loc: Loc) -> Type {
        sp(loc, Type_::Unit)
    }
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl fmt::Display for ModuleIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}::{}", self.0.value.address, &self.0.value.name)
    }
}

impl fmt::Display for UnaryOp_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        use UnaryOp_::*;
        match self {
            Not => write!(f, "!"),
            Neg => write!(f, "-"),
        }
    }
}

impl fmt::Display for BinOp_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        use BinOp_::*;
        match self {
            Add => write!(f, "+"),
            Sub => write!(f, "-"),
            Mul => write!(f, "*"),
            Mod => write!(f, "%"),
            Div => write!(f, "/"),
            BitOr => write!(f, "|"),
            BitAnd => write!(f, "&"),
            Xor => write!(f, "^"),
            And => write!(f, "&&"),
            Or => write!(f, "||"),
            Eq => write!(f, "=="),
            Neq => write!(f, "!="),
            Lt => write!(f, "<"),
            Gt => write!(f, ">"),
            Le => write!(f, "<="),
            Ge => write!(f, ">="),
        }
    }
}
