// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    expansion::ast::{Fields, SpecId},
    naming::ast::{FunctionSignature, StructDefinition, Type, TypeName_, Type_},
    parser::ast::{
        BinOp, Field, FunctionName, FunctionVisibility, ModuleIdent, StructName, UnaryOp, Value,
        Var,
    },
    shared::{ast_debug::*, unique_map::UniqueMap},
};
use move_ir_types::location::*;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
};

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug)]
pub struct Program {
    pub modules: UniqueMap<ModuleIdent, ModuleDefinition>,
    pub scripts: BTreeMap<String, Script>,
}

//**************************************************************************************************
// Scripts
//**************************************************************************************************

#[derive(Debug)]
pub struct Script {
    pub loc: Loc,
    pub function_name: FunctionName,
    pub function: Function,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

#[derive(Debug)]
pub struct ModuleDefinition {
    pub is_source_module: bool,
    /// `dependency_order` is the topological order/rank in the dependency graph.
    pub dependency_order: usize,
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
pub enum LValue_ {
    Ignore,
    Var(Var, Box<Type>),
    Unpack(ModuleIdent, StructName, Vec<Type>, Fields<(Type, LValue)>),
    BorrowUnpack(
        bool,
        ModuleIdent,
        StructName,
        Vec<Type>,
        Fields<(Type, LValue)>,
    ),
}
pub type LValue = Spanned<LValue_>;
pub type LValueList_ = Vec<LValue>;
pub type LValueList = Spanned<LValueList_>;

#[derive(Debug, PartialEq)]
pub struct ModuleCall {
    pub module: ModuleIdent,
    pub name: FunctionName,
    pub type_arguments: Vec<Type>,
    pub arguments: Box<Exp>,
    pub parameter_types: Vec<Type>,
    pub acquires: BTreeSet<StructName>,
}

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum BuiltinFunction_ {
    MoveToSender(Type),
    MoveFrom(Type),
    BorrowGlobal(bool, Type),
    Exists(Type),
    Freeze(Type),
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
    Move { from_user: bool, var: Var },
    Copy { from_user: bool, var: Var },
    Use(Var),

    ModuleCall(Box<ModuleCall>),
    Builtin(Box<BuiltinFunction>, Box<Exp>),

    IfElse(Box<Exp>, Box<Exp>, Box<Exp>),
    While(Box<Exp>, Box<Exp>),
    Loop { has_break: bool, body: Box<Exp> },
    Block(Sequence),
    Assign(LValueList, Vec<Option<Type>>, Box<Exp>),
    Mutate(Box<Exp>, Box<Exp>),
    Return(Box<Exp>),
    Abort(Box<Exp>),
    Break,
    Continue,

    Dereference(Box<Exp>),
    UnaryExp(UnaryOp, Box<Exp>),
    BinopExp(Box<Exp>, BinOp, Box<Type>, Box<Exp>),

    Pack(ModuleIdent, StructName, Vec<Type>, Fields<(Type, Exp)>),
    ExpList(Vec<ExpListItem>),

    Borrow(bool, Box<Exp>, Field),
    TempBorrow(bool, Box<Exp>),
    BorrowLocal(bool, Var),

    Cast(Box<Exp>, Box<Type>),
    Annotate(Box<Exp>, Box<Type>),

    Spec(SpecId, BTreeMap<Var, Type>),

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
    Declare(LValueList),
    Bind(LValueList, Vec<Option<Type>>, Box<Exp>),
}
pub type SequenceItem = Spanned<SequenceItem_>;

#[derive(Debug, PartialEq)]
pub enum ExpListItem {
    Single(Exp, Box<Type>),
    Splat(Loc, Exp, Vec<Type>),
}

pub fn single_item(e: Exp) -> ExpListItem {
    let ty = Box::new(e.ty.clone());
    ExpListItem::Single(e, ty)
}

pub fn splat_item(splat_loc: Loc, e: Exp) -> ExpListItem {
    let ss = match &e.ty {
        sp!(_, Type_::Unit) => vec![],
        sp!(_, Type_::Apply(_, sp!(_, TypeName_::Multiple(_)), ss)) => ss.clone(),
        _ => panic!("ICE splat of non list type"),
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
        let Program { modules, scripts } = self;
        for (m, mdef) in modules {
            w.write(&format!("module {}", m));
            w.block(|w| mdef.ast_debug(w));
            w.new_line();
        }

        for (n, s) in scripts {
            w.write(&format!("script {}", n));
            w.block(|w| s.ast_debug(w));
            w.new_line()
        }
    }
}

impl AstDebug for Script {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Script {
            loc: _loc,
            function_name,
            function,
        } = self;
        (function_name.clone(), function).ast_debug(w);
    }
}

impl AstDebug for ModuleDefinition {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ModuleDefinition {
            is_source_module,
            dependency_order,
            structs,
            functions,
        } = self;
        if *is_source_module {
            w.writeln("library module")
        } else {
            w.writeln("source module")
        }
        w.writeln(&format!("dependency order #{}", dependency_order));
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
            E::Cast(e, ty) => {
                w.write("(");
                e.ast_debug(w);
                w.write(" as ");
                ty.ast_debug(w);
                w.write(")");
            }
            E::Annotate(e, ty) => {
                w.write("annot(");
                e.ast_debug(w);
                w.write(": ");
                ty.ast_debug(w);
                w.write(")");
            }
            E::Spec(u, used_locals) => {
                w.write(&format!("spec #{}", u));
                if !used_locals.is_empty() {
                    w.write("uses [");
                    w.comma(used_locals, |w, (n, ty)| {
                        w.annotate(|w| w.write(&format!("{}", n)), ty)
                    });
                    w.write("]");
                }
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

impl AstDebug for Vec<Option<Type>> {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.comma(self, |w, t_opt| match t_opt {
            Some(t) => t.ast_debug(w),
            None => w.write("%no_exp%"),
        })
    }
}

impl AstDebug for Vec<LValue> {
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

impl AstDebug for LValue_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use LValue_ as L;
        match self {
            L::Ignore => w.write("_"),
            L::Var(v, st) => w.annotate(|w| w.write(&format!("{}", v)), st),
            L::Unpack(m, s, tys, fields) => {
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
            L::BorrowUnpack(mut_, m, s, tys, fields) => {
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
