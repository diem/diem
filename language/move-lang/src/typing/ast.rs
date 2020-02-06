// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    expansion::ast::Fields,
    naming::ast::{BaseType, FunctionSignature, SingleType, StructDefinition, Type, Type_},
    parser::ast::{
        BinOp, Field, FunctionName, FunctionVisibility, ModuleIdent, StructName, UnaryOp, Value,
        Var,
    },
    shared::ast_debug::*,
    shared::unique_map::UniqueMap,
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
    InferredNum(u128),
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
    BinopExp(Box<Exp>, BinOp, Box<Type>, Box<Exp>),

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

    Annotate(Box<Exp>, Box<Type>),

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

//**************************************************************************************************
// Debug
//**************************************************************************************************

impl AstDebug for Program {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Program { modules, main } = self;
        for (m, mdef) in modules {
            w.write(&format!("module {}", m));
            w.block(|w| mdef.ast_debug(w));
            w.new_line();
        }

        if let Some((addr, n, fdef)) = main {
            w.writeln(&format!("address {}:", addr));
            (n.clone(), fdef).ast_debug(w);
        }
    }
}

impl AstDebug for ModuleDefinition {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ModuleDefinition {
            is_source_module,
            structs,
            functions,
        } = self;
        match is_source_module {
            None => w.writeln("library module"),
            Some(ord) => w.writeln(&format!("source moduel #{}", ord)),
        }
        for sdef in structs {
            sdef.ast_debug(w);
            w.new_line();
        }
        for fdef in functions {
            fdef.ast_debug(w);
            w.new_line();
        }
    }
}

impl AstDebug for (FunctionName, &Function) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (
            name,
            Function {
                visibility,
                signature,
                acquires,
                body,
            },
        ) = self;
        visibility.ast_debug(w);
        if let FunctionBody_::Native = &body.value {
            w.write("native ");
        }
        w.write(&format!("{}", name));
        signature.ast_debug(w);
        if !acquires.is_empty() {
            w.write(" acquires ");
            w.comma(acquires, |w, s| w.write(&format!("{}", s)));
            w.write(" ");
        }
        match &body.value {
            FunctionBody_::Defined(body) => w.block(|w| body.ast_debug(w)),
            FunctionBody_::Native => w.writeln(";"),
        }
    }
}

impl AstDebug for VecDeque<SequenceItem> {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.semicolon(self, |w, item| item.ast_debug(w))
    }
}

impl AstDebug for SequenceItem_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use SequenceItem_ as I;
        match self {
            I::Seq(e) => e.ast_debug(w),
            I::Declare(sp!(_, bs)) => {
                w.write("let ");
                bs.ast_debug(w);
            }
            I::Bind(sp!(_, bs), expected_types, e) => {
                w.write("let ");
                bs.ast_debug(w);
                w.write(": (");
                expected_types.ast_debug(w);
                w.write(")");
                w.write(" = ");
                e.ast_debug(w);
            }
        }
    }
}

impl AstDebug for UnannotatedExp_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use UnannotatedExp_ as E;
        match self {
            E::Unit => w.write("()"),
            E::Value(v) => v.ast_debug(w),
            E::InferredNum(u) => w.write(&format!("{}", u)),
            E::Move {
                from_user: false,
                var: v,
            } => w.write(&format!("move {}", v)),
            E::Move {
                from_user: true,
                var: v,
            } => w.write(&format!("move@{}", v)),
            E::Copy {
                from_user: false,
                var: v,
            } => w.write(&format!("copy {}", v)),
            E::Copy {
                from_user: true,
                var: v,
            } => w.write(&format!("copy@{}", v)),
            E::Use(v) => w.write(&format!("use@{}", v)),
            E::ModuleCall(mcall) => {
                mcall.ast_debug(w);
            }
            E::Builtin(bf, rhs) => {
                bf.ast_debug(w);
                w.write("(");
                rhs.ast_debug(w);
                w.write(")");
            }
            E::Pack(m, s, tys, fields) => {
                w.write(&format!("{}::{}", m, s));
                w.write("<");
                tys.ast_debug(w);
                w.write(">");
                w.write("{");
                w.comma(fields, |w, (f, idx_bt_e)| {
                    let (idx, (bt, e)) = idx_bt_e;
                    w.write(&format!("({}#{}:", idx, f));
                    bt.ast_debug(w);
                    w.write("): ");
                    e.ast_debug(w);
                });
                w.write("}");
            }
            E::IfElse(b, t, f) => {
                w.write("if (");
                b.ast_debug(w);
                w.write(") ");
                t.ast_debug(w);
                w.write(" else ");
                f.ast_debug(w);
            }
            E::While(b, e) => {
                w.write("while (");
                b.ast_debug(w);
                w.write(")");
                e.ast_debug(w);
            }
            E::Loop { has_break, body } => {
                w.write("loop");
                if *has_break {
                    w.write("#with_break");
                }
                w.write(" ");
                body.ast_debug(w);
            }
            E::Block(seq) => w.block(|w| seq.ast_debug(w)),
            E::ExpList(es) => {
                w.write("(");
                w.comma(es, |w, e| e.ast_debug(w));
                w.write(")");
            }

            E::Assign(sp!(_, lvalues), expected_types, rhs) => {
                lvalues.ast_debug(w);
                w.write(": (");
                expected_types.ast_debug(w);
                w.write(") = ");
                rhs.ast_debug(w);
            }

            E::Mutate(lhs, rhs) => {
                w.write("*");
                lhs.ast_debug(w);
                w.write(" = ");
                rhs.ast_debug(w);
            }

            E::Return(e) => {
                w.write("return ");
                e.ast_debug(w);
            }
            E::Abort(e) => {
                w.write("abort ");
                e.ast_debug(w);
            }
            E::Break => w.write("break"),
            E::Continue => w.write("continue"),
            E::Dereference(e) => {
                w.write("*");
                e.ast_debug(w)
            }
            E::UnaryExp(op, e) => {
                op.ast_debug(w);
                w.write(" ");
                e.ast_debug(w);
            }
            E::BinopExp(l, op, ty, r) => {
                l.ast_debug(w);
                w.write(" ");
                op.ast_debug(w);
                w.write("@");
                ty.ast_debug(w);
                w.write(" ");
                r.ast_debug(w)
            }
            E::Borrow(mut_, e, f) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                e.ast_debug(w);
                w.write(&format!(".{}", f));
            }
            E::TempBorrow(mut_, e) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                e.ast_debug(w);
            }
            E::BorrowLocal(mut_, v) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                w.write(&format!("{}", v));
            }
            E::Annotate(e, ty) => {
                w.write("annot(");
                e.ast_debug(w);
                w.write(": ");
                ty.ast_debug(w);
                w.write(")");
            }
            E::UnresolvedError => w.write("_|_"),
        }
    }
}

impl AstDebug for Exp {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Exp { ty, exp } = self;
        w.annotate(|w| exp.ast_debug(w), ty)
    }
}

impl AstDebug for ModuleCall {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ModuleCall {
            module,
            name,
            type_arguments,
            parameter_types,
            acquires,
            arguments,
        } = self;
        w.write(&format!("{}::{}", module, name));
        if !acquires.is_empty() || !parameter_types.is_empty() {
            w.write("[");
            if !acquires.is_empty() {
                w.write("acquires: [");
                w.comma(acquires, |w, s| w.write(&format!("{}", s)));
                w.write("], ");
            }
            if !parameter_types.is_empty() {
                if !acquires.is_empty() {
                    w.write(", ");
                }
                w.write("parameter_types: [");
                parameter_types.ast_debug(w);
                w.write("]");
            }
        }
        w.write("<");
        type_arguments.ast_debug(w);
        w.write(">");
        w.write("(");
        arguments.ast_debug(w);
        w.write(")");
    }
}

impl AstDebug for BuiltinFunction_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use crate::naming::ast::BuiltinFunction_ as NF;
        use BuiltinFunction_ as F;
        let (n, bt) = match self {
            F::MoveToSender(bt) => (NF::MOVE_TO_SENDER, bt),
            F::MoveFrom(bt) => (NF::MOVE_FROM, bt),
            F::BorrowGlobal(true, bt) => (NF::BORROW_GLOBAL_MUT, bt),
            F::BorrowGlobal(false, bt) => (NF::BORROW_GLOBAL, bt),
            F::Exists(bt) => (NF::EXISTS, bt),
            F::Freeze(bt) => (NF::FREEZE, bt),
        };
        w.write(n);
        w.write("<");
        bt.ast_debug(w);
        w.write(">");
    }
}

impl AstDebug for ExpListItem {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            ExpListItem::Single(e, st) => w.annotate(|w| e.ast_debug(w), st),
            ExpListItem::Splat(_, e, ss) => {
                w.write("~");
                w.annotate(|w| e.ast_debug(w), ss)
            }
        }
    }
}

impl AstDebug for Vec<Bind> {
    fn ast_debug(&self, w: &mut AstWriter) {
        let parens = self.len() != 1;
        if parens {
            w.write("(");
        }
        w.comma(self, |w, b| b.ast_debug(w));
        if parens {
            w.write(")");
        }
    }
}

impl AstDebug for Bind_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use Bind_ as B;
        match self {
            B::Ignore => w.write("_"),
            B::Var(v, None) => w.write(&format!("{}", v)),
            B::Var(v, Some(st)) => w.annotate(|w| w.write(&format!("{}", v)), st),
            B::Unpack(m, s, tys, fields) => {
                w.write(&format!("{}::{}", m, s));
                w.write("<");
                tys.ast_debug(w);
                w.write(">");
                w.write("{");
                w.comma(fields, |w, (f, idx_bt_b)| {
                    let (idx, (bt, b)) = idx_bt_b;
                    w.annotate(|w| w.write(&format!("{}#{}", idx, f)), bt);
                    w.write(": ");
                    b.ast_debug(w);
                });
                w.write("}");
            }
            B::BorrowUnpack(mut_, m, s, tys, fields) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                w.write(&format!("{}::{}", m, s));
                w.write("<");
                tys.ast_debug(w);
                w.write(">");
                w.write("{");
                w.comma(fields, |w, (f, idx_bt_b)| {
                    let (idx, (bt, b)) = idx_bt_b;
                    w.annotate(|w| w.write(&format!("{}#{}", idx, f)), bt);
                    w.write(": ");
                    b.ast_debug(w);
                });
                w.write("}");
            }
        }
    }
}

impl AstDebug for Vec<Option<SingleType>> {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.comma(self, |w, s_opt| match s_opt {
            Some(s) => s.ast_debug(w),
            None => w.write("%no_exp%"),
        })
    }
}

impl AstDebug for Vec<Assign> {
    fn ast_debug(&self, w: &mut AstWriter) {
        let parens = self.len() != 1;
        if parens {
            w.write("(");
        }
        w.comma(self, |w, a| a.ast_debug(w));
        if parens {
            w.write(")");
        }
    }
}

impl AstDebug for Assign_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use Assign_ as A;
        match self {
            A::Ignore => w.write("_"),
            A::Var(v, st) => w.annotate(|w| w.write(&format!("{}", v)), st),
            A::Unpack(m, s, tys, fields) => {
                w.write(&format!("{}::{}", m, s));
                w.write("<");
                tys.ast_debug(w);
                w.write(">");
                w.write("{");
                w.comma(fields, |w, (f, idx_bt_a)| {
                    let (idx, (bt, a)) = idx_bt_a;
                    w.annotate(|w| w.write(&format!("{}#{}", idx, f)), bt);
                    w.write(": ");
                    a.ast_debug(w);
                });
                w.write("}");
            }
            A::BorrowUnpack(mut_, m, s, tys, fields) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                w.write(&format!("{}::{}", m, s));
                w.write("<");
                tys.ast_debug(w);
                w.write(">");
                w.write("{");
                w.comma(fields, |w, (f, idx_bt_a)| {
                    let (idx, (bt, a)) = idx_bt_a;
                    w.annotate(|w| w.write(&format!("{}#{}", idx, f)), bt);
                    w.write(": ");
                    a.ast_debug(w);
                });
                w.write("}");
            }
        }
    }
}
