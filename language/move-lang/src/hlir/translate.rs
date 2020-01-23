// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::unique_map::UniqueMap;
use crate::{
    errors::Errors,
    expansion::ast::Fields,
    hlir::ast::{self as H, Block},
    naming::ast as N,
    parser::ast::{BinOp_, Field, FunctionName, ModuleIdent, StructName, Var},
    shared::*,
    typing::ast as T,
};
use std::collections::VecDeque;

//**************************************************************************************************
// Vars
//**************************************************************************************************

const NEW_NAME_DELIM: &str = "#";

fn new_name(n: &str) -> String {
    format!("{}{}{}", n, NEW_NAME_DELIM, Counter::next())
}

const TEMP_PREFIX: &str = "tmp%";

fn new_temp_name() -> String {
    new_name(TEMP_PREFIX)
}

fn is_temp_name(s: &str) -> bool {
    s.starts_with(TEMP_PREFIX)
}

pub enum DisplayVar {
    Orig(String),
    Tmp,
}

pub fn display_var(s: &str) -> DisplayVar {
    if is_temp_name(s) {
        DisplayVar::Tmp
    } else {
        let mut orig = s.to_owned();
        orig.split_off(orig.find('#').unwrap_or_else(|| s.len()));
        DisplayVar::Orig(orig)
    }
}

//**************************************************************************************************
// Context
//**************************************************************************************************

struct Context {
    errors: Errors,
    structs: UniqueMap<StructName, UniqueMap<Field, usize>>,
    function_locals: UniqueMap<Var, H::SingleType>,
    local_scope: UniqueMap<Var, Var>,
    return_type: Option<H::Type>,
    has_return_abort: bool,
}

impl Context {
    pub fn new(errors: Errors) -> Self {
        Context {
            errors,
            structs: UniqueMap::new(),
            function_locals: UniqueMap::new(),
            local_scope: UniqueMap::new(),
            return_type: None,
            has_return_abort: false,
        }
    }

    pub fn error(&mut self, e: Vec<(Loc, impl Into<String>)>) {
        self.errors
            .push(e.into_iter().map(|(loc, msg)| (loc, msg.into())).collect())
    }

    pub fn get_errors(self) -> Errors {
        self.errors
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_empty_locals(&self) -> bool {
        self.function_locals.is_empty() && self.local_scope.is_empty()
    }

    pub fn extract_function_locals(&mut self) -> UniqueMap<Var, H::SingleType> {
        self.local_scope = UniqueMap::new();
        std::mem::replace(&mut self.function_locals, UniqueMap::new())
    }

    pub fn new_temp(&mut self, loc: Loc, t: H::SingleType) -> Var {
        let new_var = Var(sp(loc, new_temp_name()));
        self.function_locals.add(new_var.clone(), t).unwrap();
        self.local_scope
            .add(new_var.clone(), new_var.clone())
            .unwrap();
        new_var
    }

    pub fn bind_local(&mut self, v: Var, t: H::SingleType) {
        let new_var = if !self.function_locals.contains_key(&v) {
            v.clone()
        } else {
            Var(sp(v.loc(), new_name(v.value())))
        };
        self.function_locals.add(new_var.clone(), t).unwrap();
        self.local_scope.remove(&v);
        assert!(!self.local_scope.contains_key(&new_var));
        self.local_scope.add(v, new_var).unwrap();
    }

    pub fn remapped_local(&mut self, v: Var) -> Var {
        self.local_scope.get(&v).unwrap().clone()
    }

    pub fn add_struct_fields(&mut self, structs: &UniqueMap<StructName, H::StructDefinition>) {
        assert!(self.structs.is_empty());
        for (sname, sdef) in structs.iter() {
            let mut fields = UniqueMap::new();
            let field_map = match &sdef.fields {
                H::StructFields::Native(_) => continue,
                H::StructFields::Defined(m) => m,
            };
            for (idx, (field, _)) in field_map.iter().enumerate() {
                fields.add(field.clone(), idx).unwrap();
            }
            self.structs.add(sname, fields).unwrap();
        }
    }

    pub fn fields(&self, struct_name: &StructName) -> &UniqueMap<Field, usize> {
        self.structs.get(struct_name).unwrap()
    }
}

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn program(prog: T::Program) -> (H::Program, Errors) {
    let mut context = Context::new(vec![]);
    let modules = modules(&mut context, prog.modules);
    let main = prog
        .main
        .map(|(addr, n, fdef)| (addr, n.clone(), function(&mut context, n, fdef)));

    (H::Program { modules, main }, context.get_errors())
}

fn modules(
    context: &mut Context,
    modules: UniqueMap<ModuleIdent, T::ModuleDefinition>,
) -> UniqueMap<ModuleIdent, H::ModuleDefinition> {
    let hlir_modules = modules
        .into_iter()
        .map(|(mname, m)| module(context, mname, m));
    UniqueMap::maybe_from_iter(hlir_modules).unwrap()
}

fn module(
    context: &mut Context,
    module_ident: ModuleIdent,
    mdef: T::ModuleDefinition,
) -> (ModuleIdent, H::ModuleDefinition) {
    let is_source_module = mdef.is_source_module;

    let structs = mdef.structs.map(|name, s| struct_def(context, name, s));

    context.add_struct_fields(&structs);
    let functions = mdef.functions.map(|name, f| function(context, name, f));
    context.structs = UniqueMap::new();

    (
        module_ident,
        H::ModuleDefinition {
            is_source_module,
            structs,
            functions,
        },
    )
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

fn function(context: &mut Context, _name: FunctionName, f: T::Function) -> H::Function {
    assert!(context.has_empty_locals());
    let visibility = f.visibility;
    let signature = function_signature(context, f.signature);
    let acquires = f.acquires;
    let body = function_body(context, &signature.return_type, f.body);
    H::Function {
        visibility,
        signature,
        acquires,
        body,
    }
}

fn function_signature(context: &mut Context, sig: N::FunctionSignature) -> H::FunctionSignature {
    let type_parameters = sig.type_parameters;
    let parameters = sig
        .parameters
        .into_iter()
        .map(|(v, tty)| {
            let ty = single_type(context, tty);
            context.bind_local(v.clone(), ty.clone());
            (v, ty)
        })
        .collect();
    let return_type = type_(context, sig.return_type);
    H::FunctionSignature {
        type_parameters,
        parameters,
        return_type,
    }
}

fn function_body(
    context: &mut Context,
    ret_ty: &H::Type,
    sp!(loc, tb_): T::FunctionBody,
) -> H::FunctionBody {
    use H::FunctionBody_ as HB;
    use T::FunctionBody_ as TB;
    let b_ = match tb_ {
        TB::Native => {
            context.extract_function_locals();
            HB::Native
        }
        TB::Defined(seq) => {
            let mut body = VecDeque::new();
            context.return_type = Some(ret_ty.clone());
            assert!(!context.has_return_abort);
            let final_exp = block(context, &mut body, loc, Some(&ret_ty), seq);
            context.return_type = None;
            context.has_return_abort = false;
            match final_exp {
                Unreachable { .. } => (),
                Reachable(e) => {
                    use H::{Command_ as C, Statement_ as S};
                    let eloc = e.exp.loc;
                    let ret = sp(eloc, C::Return(e));
                    body.push_back(sp(eloc, S::Command(ret)))
                }
            }
            let locals = context.extract_function_locals();
            HB::Defined { locals, body }
        }
    };
    sp(loc, b_)
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

fn struct_def(
    context: &mut Context,
    _name: StructName,
    sdef: N::StructDefinition,
) -> H::StructDefinition {
    let resource_opt = sdef.resource_opt;
    let type_parameters = sdef.type_parameters;
    let fields = struct_fields(context, sdef.fields);
    H::StructDefinition {
        resource_opt,
        type_parameters,
        fields,
    }
}

fn struct_fields(context: &mut Context, tfields: N::StructFields) -> H::StructFields {
    let tfields_map = match tfields {
        N::StructFields::Native(loc) => return H::StructFields::Native(loc),
        N::StructFields::Defined(m) => m,
    };
    let mut indexed_fields = tfields_map
        .into_iter()
        .map(|(f, (idx, t))| (idx, (f, base_type(context, t))))
        .collect::<Vec<_>>();
    indexed_fields.sort_by(|(idx1, _), (idx2, _)| idx1.cmp(idx2));
    H::StructFields::Defined(indexed_fields.into_iter().map(|(_, f_ty)| f_ty).collect())
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn base_types<R: std::iter::FromIterator<H::BaseType>>(
    context: &Context,
    tys: impl IntoIterator<Item = N::BaseType>,
) -> R {
    tys.into_iter().map(|t| base_type(context, t)).collect()
}

fn base_type(context: &Context, sp!(loc, nb_): N::BaseType) -> H::BaseType {
    use H::BaseType_ as HB;
    use N::BaseType_ as NB;
    let b_ = match nb_ {
        NB::Var(_) => panic!("ICE tvar not expanded: {}:{}", loc.file(), loc.span()),
        NB::Apply(None, _, _) => panic!("ICE kind not expanded: {:#?}", loc),
        NB::Apply(Some(k), n, nbs) => HB::Apply(k, n, base_types(context, nbs)),
        NB::Param(tp) => HB::Param(tp),
        NB::Anything => HB::UnresolvedError,
    };
    sp(loc, b_)
}

fn expected_types(context: &Context, loc: Loc, nss: Vec<Option<N::SingleType>>) -> H::Type {
    let any = || {
        sp(
            loc,
            H::SingleType_::Base(sp(loc, H::BaseType_::UnresolvedError)),
        )
    };
    let ss = nss
        .into_iter()
        .map(|sopt| sopt.map(|s| single_type(context, s)).unwrap_or_else(any))
        .collect::<Vec<_>>();
    H::Type_::from_vec(loc, ss)
}

fn single_types(context: &Context, ss: Vec<N::SingleType>) -> Vec<H::SingleType> {
    ss.into_iter().map(|s| single_type(context, s)).collect()
}

fn single_type(context: &Context, sp!(loc, ns_): N::SingleType) -> H::SingleType {
    use H::SingleType_ as HS;
    use N::SingleType_ as NS;
    let s_ = match ns_ {
        NS::Ref(mut_, nb) => HS::Ref(mut_, base_type(context, nb)),
        NS::Base(nb) => HS::Base(base_type(context, nb)),
    };
    sp(loc, s_)
}

fn type_(context: &Context, sp!(loc, et_): N::Type) -> H::Type {
    use H::Type_ as HT;
    use N::Type_ as NT;
    let t_ = match et_ {
        NT::Unit => HT::Unit,
        NT::Single(s) => HT::Single(single_type(context, s)),
        NT::Multiple(ss) => HT::Multiple(single_types(context, ss)),
    };
    sp(loc, t_)
}

//**************************************************************************************************
// Reachability
//**************************************************************************************************

#[allow(dead_code)]
enum ReachableResult<T> {
    Unreachable { report: bool, loc: Loc },
    Reachable(T),
}
type ExpResult = ReachableResult<H::Exp>;
use ReachableResult::*;

const DEAD_CODE_ERR: &str = "Invalid use of a divergent expression. The code following the evaluation of this expression will be dead and should be removed. In some cases, this is necessary to prevent unused resource values.";

fn dead_code_err(context: &mut Context, report: bool, loc: Loc) {
    if report {
        context.error(vec![(loc, DEAD_CODE_ERR)]);
    }
}

macro_rules! exp_ {
    ($context:ident, $block:expr, $expected_type:expr, $typed_exp:expr) => {{
        match maybe_exp($context, $block, $expected_type, $typed_exp) {
            Unreachable { report, loc } => {
                dead_code_err($context, report, loc);
                return Unreachable { report: false, loc };
            }
            Reachable(e) => e,
        }
    }};
}

macro_rules! exp {
    ($context:ident, $block:expr, $expected_type:expr, $typed_exp:expr) => {{
        Box::new(exp_!($context, $block, $expected_type, $typed_exp))
    }};
}

macro_rules! statement_exp {
    ($context:ident, $block:expr, $expected_type:expr, $typed_exp:expr) => {{
        match maybe_exp($context, $block, $expected_type, $typed_exp) {
            Unreachable { report, loc } => {
                dead_code_err($context, report, loc);
                return;
            }
            Reachable(e) => Box::new(e),
        }
    }};
}

fn unit_() -> H::UnannotatedExp_ {
    H::UnannotatedExp_::ExpList(vec![])
}

fn unit_result(loc: Loc) -> ExpResult {
    Reachable(H::exp(sp(loc, H::Type_::Unit), sp(loc, unit_())))
}

//**************************************************************************************************
// Statements
//**************************************************************************************************

fn block(
    context: &mut Context,
    result: &mut Block,
    loc: Loc,
    expected_type_opt: Option<&H::Type>,
    mut seq: T::Sequence,
) -> ExpResult {
    use T::SequenceItem_ as S;
    let last = match seq.pop_back() {
        None => return unit_result(loc),
        Some(sp!(_, S::Seq(last))) => last,
        Some(_) => panic!("ICE last sequence item should be exp"),
    };

    let old_scope = context.local_scope.clone();
    for sp!(sloc, seq_item_) in seq {
        match seq_item_ {
            S::Seq(te) => statement(context, result, *te),
            S::Declare(binds) => declare_bind_list(context, &binds),
            S::Bind(binds, ty, e) => {
                let expected_tys = expected_types(context, sloc, ty);
                let res = match maybe_exp(context, result, Some(&expected_tys), *e) {
                    Unreachable { report, loc } => {
                        dead_code_err(context, report, loc);
                        context.local_scope = old_scope;
                        return Unreachable { report: false, loc };
                    }
                    Reachable(res) => res,
                };
                declare_bind_list(context, &binds);
                match binds_as_assign(context, result, sloc, binds, res) {
                    Unreachable { report, loc } => {
                        context.local_scope = old_scope;
                        return Unreachable { report, loc };
                    }
                    Reachable(()) => (),
                };
            }
        }
    }
    let res = maybe_exp(context, result, expected_type_opt, *last);
    context.local_scope = old_scope;
    res
}

fn statement(context: &mut Context, result: &mut Block, e: T::Exp) {
    use H::Statement_ as S;
    use T::UnannotatedExp_ as TE;

    let ty = e.ty;
    let sp!(eloc, e_) = e.exp;
    let stmt_ = match e_ {
        TE::IfElse(tb, tt, tf) => {
            let cond = statement_exp!(context, result, None, *tb);

            let mut if_block = Block::new();
            let et = maybe_exp(context, &mut if_block, None, *tt);
            ignore_and_pop(context, &mut if_block, true, et);

            let mut else_block = Block::new();
            let ef = maybe_exp(context, &mut else_block, None, *tf);
            ignore_and_pop(context, &mut else_block, true, ef);

            S::IfElse {
                cond,
                if_block,
                else_block,
            }
        }
        TE::While(tb, loop_body) => {
            let cond = statement_exp!(context, result, None, *tb);

            let mut loop_block = Block::new();
            let el = maybe_exp(context, &mut loop_block, None, *loop_body);
            ignore_and_pop(context, &mut loop_block, true, el);

            S::While {
                cond,
                block: loop_block,
            }
        }
        TE::Loop {
            body: loop_body,
            has_break,
        } => {
            let (loop_block, has_return_abort) = statement_loop_body(context, *loop_body);

            S::Loop {
                block: loop_block,
                has_break,
                has_return_abort,
            }
        }
        TE::Block(seq) => {
            let res = block(context, result, eloc, None, seq);
            ignore_and_pop(context, result, false, res);
            return;
        }
        e_ => {
            let te = T::exp(ty, sp(eloc, e_));
            let e = maybe_exp(context, result, None, te);
            ignore_and_pop(context, result, false, e);
            return;
        }
    };
    result.push_back(sp(eloc, stmt_))
}

fn statement_loop_body(context: &mut Context, body: T::Exp) -> (Block, bool) {
    let old_has_return_abort = context.has_return_abort;
    context.has_return_abort = false;
    let mut loop_block = Block::new();
    let el = maybe_exp(context, &mut loop_block, None, body);
    ignore_and_pop(context, &mut loop_block, true, el);
    let has_return_abort = context.has_return_abort;
    context.has_return_abort = context.has_return_abort || old_has_return_abort;
    (loop_block, has_return_abort)
}

//**************************************************************************************************
// LValue
//**************************************************************************************************

fn declare_bind_list(context: &mut Context, sp!(_, binds): &T::BindList) {
    binds.iter().for_each(|b| declare_bind(context, b))
}

fn declare_bind(context: &mut Context, sp!(_, bind_): &T::Bind) {
    use T::Bind_ as B;
    match bind_ {
        B::Ignore => (),
        B::Var(v, tst) => {
            let st = single_type(context, tst.clone().unwrap());
            context.bind_local(v.clone(), st)
        }
        B::Unpack(_, _, _, fields) | B::BorrowUnpack(_, _, _, _, fields) => fields
            .iter()
            .for_each(|(_, (_, (_, b)))| declare_bind(context, b)),
    }
}

fn binds_as_assign(
    context: &mut Context,
    result: &mut Block,
    loc: Loc,
    sp!(lloc, binds_): T::BindList,
    e: H::Exp,
) -> ReachableResult<()> {
    let assigns = binds_.into_iter().map(bind_as_assign).collect();
    assign_command(context, result, loc, sp(lloc, assigns), e)
}

fn bind_as_assign(sp!(loc, tb_): T::Bind) -> T::Assign {
    use T::{Assign_ as A, Bind_ as B};
    let ta_ = match tb_ {
        B::Ignore => A::Ignore,
        B::Var(v, topt) => A::Var(v, topt.unwrap()),
        B::Unpack(m, s, bs, fields) => A::Unpack(m, s, bs, bind_fields_as_assign(fields)),
        B::BorrowUnpack(mut_, m, s, bs, fields) => {
            A::BorrowUnpack(mut_, m, s, bs, bind_fields_as_assign(fields))
        }
    };
    sp(loc, ta_)
}

fn bind_fields_as_assign(
    fields: Fields<(N::BaseType, T::Bind)>,
) -> Fields<(N::BaseType, T::Assign)> {
    fields.map(|_, (idx, (tb, b))| (idx, (tb, bind_as_assign(b))))
}

fn assign_command(
    context: &mut Context,
    result: &mut Block,
    loc: Loc,
    sp!(_, assigns): T::AssignList,
    rvalue: H::Exp,
) -> ReachableResult<()> {
    use H::{Command_ as C, Statement_ as S};
    let mut lvalues = vec![];
    let mut after = Block::new();
    for (idx, a) in assigns.into_iter().enumerate() {
        let a_ty = rvalue.ty.value.type_at_index(idx);
        let (ls, mut af) = match assign(context, result, a, a_ty) {
            Unreachable { report, loc } => return Unreachable { report, loc },
            Reachable(res) => res,
        };

        lvalues.push(ls);
        after.append(&mut af);
    }
    result.push_back(sp(
        loc,
        S::Command(sp(loc, C::Assign(lvalues, Box::new(rvalue)))),
    ));
    result.append(&mut after);
    Reachable(())
}

fn assign(
    context: &mut Context,
    result: &mut Block,
    sp!(loc, ta_): T::Assign,
    rvalue_ty: &H::SingleType,
) -> ReachableResult<(H::LValue, Block)> {
    use H::{LValue_ as L, UnannotatedExp_ as E};
    use T::Assign_ as A;
    let mut after = Block::new();
    let l_ = match ta_ {
        A::Ignore => L::Ignore,
        A::Var(v, st) => L::Var(
            context.remapped_local(v),
            Box::new(single_type(context, st)),
        ),
        A::Unpack(_m, s, tbs, tfields) => {
            let bs = base_types(context, tbs);

            let mut fields = vec![];
            for (decl_idx, f, bt, tfa) in assign_fields(context, &s, tfields) {
                assert!(fields.len() == decl_idx);
                let st = &H::SingleType_::base(bt);
                let (fa, mut fafter) = match assign(context, result, tfa, st) {
                    Unreachable { report, loc } => return Unreachable { report, loc },
                    Reachable(res) => res,
                };
                after.append(&mut fafter);
                fields.push((f, fa))
            }
            L::Unpack(s, bs, fields)
        }
        A::BorrowUnpack(mut_, _m, s, _tss, tfields) => {
            let tmp = context.new_temp(loc, rvalue_ty.clone());
            let copy_tmp = || {
                let copy_tmp_ = E::Copy {
                    from_user: false,
                    var: tmp.clone(),
                };
                H::exp(H::Type_::single(rvalue_ty.clone()), sp(loc, copy_tmp_))
            };
            let fields = assign_fields(context, &s, tfields).into_iter().enumerate();
            for (idx, (decl_idx, f, bt, tfa)) in fields {
                assert!(idx == decl_idx);
                let floc = tfa.loc;
                let borrow_ = E::Borrow(mut_, Box::new(copy_tmp()), f);
                let borrow_ty = H::Type_::single(sp(floc, H::SingleType_::Ref(mut_, bt)));
                let borrow = H::exp(borrow_ty, sp(floc, borrow_));
                match assign_command(context, &mut after, floc, sp(floc, vec![tfa]), borrow) {
                    Unreachable { report, loc } => return Unreachable { report, loc },
                    Reachable(()) => (),
                };
            }
            L::Var(tmp, Box::new(rvalue_ty.clone()))
        }
    };
    Reachable((sp(loc, l_), after))
}

fn assign_fields(
    context: &Context,
    s: &StructName,
    tfields: Fields<(N::BaseType, T::Assign)>,
) -> Vec<(usize, Field, H::BaseType, T::Assign)> {
    let decl_fields = context.fields(s);
    let decl_field = |f: &Field| -> usize { *decl_fields.get(f).unwrap() };
    let mut tfields_vec = tfields
        .into_iter()
        .map(|(f, (_idx, (tbt, tfa)))| (decl_field(&f), f, base_type(context, tbt), tfa))
        .collect::<Vec<_>>();
    tfields_vec.sort_by(|(idx1, _, _, _), (idx2, _, _, _)| idx1.cmp(idx2));
    tfields_vec
}

//**************************************************************************************************
// Commands
//**************************************************************************************************

fn ignore_and_pop(
    context: &mut Context,
    result: &mut Block,
    last_stmt: bool,
    exp_result: ExpResult,
) {
    match exp_result {
        // No need to error, the end of the block is unreachable
        // which does not result in unused values
        Unreachable { .. } if last_stmt => (),
        Unreachable { report, loc } => dead_code_err(context, report, loc),
        Reachable(exp) => {
            if let H::UnannotatedExp_::Unit = &exp.exp.value {
                return;
            }
            let pop_num = match &exp.ty.value {
                H::Type_::Unit => 0,
                H::Type_::Single(_) => 1,
                H::Type_::Multiple(tys) => tys.len(),
            };
            let loc = exp.exp.loc;
            let c = sp(loc, H::Command_::IgnoreAndPop { pop_num, exp });
            result.push_back(sp(loc, H::Statement_::Command(c)))
        }
    }
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

fn maybe_exp(
    context: &mut Context,
    result: &mut Block,
    expected_type_opt: Option<&H::Type>,
    e: T::Exp,
) -> ExpResult {
    match (maybe_exp_(context, result, e), expected_type_opt) {
        (Reachable(e), Some(ety)) => {
            if needs_freeze(&e.ty, ety) != Freeze::NotNeeded {
                Reachable(freeze(context, result, ety, e))
            } else {
                Reachable(e)
            }
        }
        (res, _) => res,
    }
}

fn maybe_exp_(context: &mut Context, result: &mut Block, e: T::Exp) -> ExpResult {
    use H::{Command_ as C, Statement_ as S, UnannotatedExp_ as HE};
    use T::UnannotatedExp_ as TE;

    let ty = type_(context, e.ty);
    let sp!(eloc, e_) = e.exp;
    let res = match e_ {
        // Statement-like expressions
        TE::IfElse(tb, tt, tf) => {
            let cond = exp!(context, result, None, *tb);

            let mut if_block = Block::new();
            let et = maybe_exp(context, &mut if_block, Some(&ty), *tt);

            let mut else_block = Block::new();
            let ef = maybe_exp(context, &mut else_block, Some(&ty), *tf);

            if let (Unreachable { .. }, Unreachable { .. }) = (&et, &ef) {
                let s_ = S::IfElse {
                    cond,
                    if_block,
                    else_block,
                };
                result.push_back(sp(eloc, s_));
                return Unreachable {
                    report: true,
                    loc: eloc,
                };
            }

            let tmps = make_temps(context, eloc, ty.clone());
            let tres = bind_result(&mut if_block, eloc, tmps.clone(), et);
            let fres = bind_result(&mut else_block, eloc, tmps, ef);
            let s_ = S::IfElse {
                cond,
                if_block,
                else_block,
            };
            result.push_back(sp(eloc, s_));

            match (tres, fres) {
                (Reachable(res), _) | (_, Reachable(res)) => res.exp.value,
                (Unreachable { .. }, Unreachable { .. }) => {
                    unreachable!("ICE should have been covered in (et, ef) match")
                }
            }
        }
        TE::While(tb, loop_body) => {
            let cond = exp!(context, result, None, *tb);

            let mut loop_block = Block::new();
            let el = maybe_exp(context, &mut loop_block, None, *loop_body);
            ignore_and_pop(context, &mut loop_block, true, el);

            let s_ = S::While {
                cond,
                block: loop_block,
            };
            result.push_back(sp(eloc, s_));
            unit_()
        }
        TE::Loop {
            has_break,
            body: loop_body,
        } => {
            let (loop_block, has_return_abort) = statement_loop_body(context, *loop_body);

            let s_ = S::Loop {
                block: loop_block,
                has_break,
                has_return_abort,
            };
            result.push_back(sp(eloc, s_));
            if !has_break {
                return Unreachable {
                    report: true,
                    loc: eloc,
                };
            }
            unit_()
        }
        TE::Block(seq) => return block(context, result, eloc, None, seq),
        // Command-like expressions
        TE::Return(te) => {
            let expected_type = context.return_type.clone();
            let e = exp_!(context, result, expected_type.as_ref(), *te);
            context.has_return_abort = true;
            let c = sp(eloc, C::Return(e));
            result.push_back(sp(eloc, S::Command(c)));
            return Unreachable {
                report: true,
                loc: eloc,
            };
        }
        TE::Abort(te) => {
            let e = exp_!(context, result, None, *te);
            context.has_return_abort = true;
            let c = sp(eloc, C::Abort(e));
            result.push_back(sp(eloc, S::Command(c)));
            return Unreachable {
                report: true,
                loc: eloc,
            };
        }
        TE::Break => {
            let c = sp(eloc, C::Break);
            result.push_back(sp(eloc, S::Command(c)));
            return Unreachable {
                report: true,
                loc: eloc,
            };
        }
        TE::Continue => {
            let c = sp(eloc, C::Continue);
            result.push_back(sp(eloc, S::Command(c)));
            return Unreachable {
                report: true,
                loc: eloc,
            };
        }
        TE::Assign(assigns, lvalue_ty, te) => {
            let expected_type = expected_types(context, eloc, lvalue_ty);
            let e = exp_!(context, result, Some(&expected_type), *te);
            match assign_command(context, result, eloc, assigns, e) {
                Unreachable { report, loc } => return Unreachable { report, loc },
                Reachable(()) => unit_(),
            }
        }
        TE::Mutate(tl, tr) => {
            let er = exp!(context, result, None, *tr);
            let el = exp!(context, result, None, *tl);
            let c = sp(eloc, C::Mutate(el, er));
            result.push_back(sp(eloc, S::Command(c)));
            unit_()
        }
        // All other expressiosn
        TE::Unit => unit_(),
        TE::Value(v) => HE::Value(v),
        TE::Move { from_user, var } => HE::Move {
            from_user,
            var: context.remapped_local(var),
        },
        TE::Copy { from_user, var } => HE::Copy {
            from_user,
            var: context.remapped_local(var),
        },
        TE::BorrowLocal(mut_, v) => HE::BorrowLocal(mut_, context.remapped_local(v)),

        TE::Use(_) => panic!("ICE unexpanded use"),
        TE::ModuleCall(call) => {
            use crate::shared::fake_natives::transaction as TXN;
            let T::ModuleCall {
                module,
                name,
                type_arguments,
                arguments,
                parameter_types,
                acquires,
            } = *call;
            let a_m_f = (
                &module.0.value.address,
                module.0.value.name.value(),
                name.value(),
            );
            if let (&Address::LIBRA_CORE, TXN::MOD, TXN::ASSERT) = a_m_f {
                let tbool = N::SingleType_::bool(eloc);
                let tu64 = N::SingleType_::u64(eloc);
                let tunit = sp(eloc, N::Type_::Unit);
                let vcond = Var(sp(eloc, new_temp_name()));
                let vcode = Var(sp(eloc, new_temp_name()));

                let mut stmts = VecDeque::new();

                let bvar = |v, st| sp(eloc, T::Bind_::Var(v, Some(st)));
                let bind_list = sp(
                    eloc,
                    vec![
                        bvar(vcond.clone(), tbool.clone()),
                        bvar(vcode.clone(), tu64.clone()),
                    ],
                );
                let tys = vec![Some(tbool.clone()), Some(tu64.clone())];
                let bind = sp(eloc, T::SequenceItem_::Bind(bind_list, tys, arguments));
                stmts.push_back(bind);

                let mvar = |var, st| {
                    let from_user = false;
                    let mv = TE::Move { from_user, var };
                    T::exp(N::Type_::single(st), sp(eloc, mv))
                };
                let econd = mvar(vcond, tu64);
                let ecode = mvar(vcode, tbool);
                let eabort = T::exp(tunit.clone(), sp(eloc, TE::Abort(Box::new(ecode))));
                let eunit = T::exp(tunit.clone(), sp(eloc, TE::Unit));
                let inlined_ = TE::IfElse(Box::new(econd), Box::new(eunit), Box::new(eabort));
                let inlined = T::exp(tunit.clone(), sp(eloc, inlined_));
                stmts.push_back(sp(eloc, T::SequenceItem_::Seq(Box::new(inlined))));

                let block = T::exp(tunit, sp(eloc, TE::Block(stmts)));
                return maybe_exp_(context, result, block);
            }
            let expected_type = H::Type_::from_vec(eloc, single_types(context, parameter_types));
            let htys = base_types(context, type_arguments);
            let harg = exp!(context, result, Some(&expected_type), *arguments);
            let call = H::ModuleCall {
                module,
                name,
                type_arguments: htys,
                arguments: harg,
                acquires,
            };
            HE::ModuleCall(Box::new(call))
        }
        TE::Builtin(bf, targ) => {
            let arg = exp!(context, result, None, *targ);
            builtin(context, result, eloc, *bf, arg)
        }
        TE::Dereference(te) => {
            let e = exp!(context, result, None, *te);
            HE::Dereference(e)
        }
        TE::UnaryExp(op, te) => {
            let e = exp!(context, result, None, *te);
            HE::UnaryExp(op, e)
        }
        TE::BinopExp(tl, op @ sp!(_, BinOp_::Eq), toperand_ty, tr)
        | TE::BinopExp(tl, op @ sp!(_, BinOp_::Neq), toperand_ty, tr) => {
            let operand_ty = type_(context, *toperand_ty);
            let el = exp!(context, result, Some(&operand_ty), *tl);
            let er = exp!(context, result, Some(&operand_ty), *tr);
            HE::BinopExp(el, op, er)
        }
        TE::BinopExp(tl, op, _, tr) => {
            let el = exp!(context, result, None, *tl);
            let er = exp!(context, result, None, *tr);
            HE::BinopExp(el, op, er)
        }
        TE::Pack(_, s, tbs, tfields) => {
            let bs = base_types(context, tbs);

            let decl_fields = context.fields(&s);
            let decl_field = |f: &Field| -> usize { *decl_fields.get(f).unwrap() };

            let mut texp_fields: Vec<(usize, Field, usize, N::BaseType, T::Exp)> = tfields
                .into_iter()
                .map(|(f, (exp_idx, (bt, tf)))| (decl_field(&f), f, exp_idx, bt, tf))
                .collect();
            texp_fields.sort_by(|(_, _, eidx1, _, _), (_, _, eidx2, _, _)| eidx1.cmp(eidx2));

            let mut fields = (0..decl_fields.len()).map(|_| None).collect::<Vec<_>>();
            for (decl_idx, f, exp_idx, bt, tf) in texp_fields {
                let bt = base_type(context, bt);
                let t = H::Type_::base(bt.clone());
                let ef = exp_!(context, result, Some(&t), tf);
                assert!(fields.get(decl_idx).unwrap().is_none());
                if decl_idx == exp_idx {
                    fields[decl_idx] = Some((f, bt, ef))
                } else {
                    let floc = ef.exp.loc;
                    let st = H::SingleType_::base(bt.clone());
                    let tmp = bind_exp(context, result, floc, st, ef);
                    let move_ = HE::Move {
                        from_user: false,
                        var: tmp,
                    };
                    let move_tmp = H::exp(t, sp(eloc, move_));
                    fields[decl_idx] = Some((f, bt, move_tmp))
                }
            }
            HE::Pack(s, bs, fields.into_iter().map(|o| o.unwrap()).collect())
        }
        TE::ExpList(titems) => {
            let mut items = vec![];
            for titem in titems {
                let item = match titem {
                    T::ExpListItem::Single(te, ts) => {
                        let e = exp_!(context, result, None, te);
                        let s = single_type(context, *ts);
                        H::ExpListItem::Single(e, Box::new(s))
                    }
                    T::ExpListItem::Splat(sloc, te, tss) => {
                        let e = exp_!(context, result, None, te);
                        let ss = single_types(context, tss);
                        H::ExpListItem::Splat(sloc, e, ss)
                    }
                };
                items.push(item)
            }
            HE::ExpList(items)
        }
        TE::Borrow(mut_, te, f) => {
            let e = exp!(context, result, None, *te);
            HE::Borrow(mut_, e, f)
        }
        TE::TempBorrow(mut_, te) => {
            let e = exp_!(context, result, None, *te);
            let st = match &e.ty.value {
                H::Type_::Single(s) => s.clone(),
                _ => panic!("ICE borrow unit or multiple values"),
            };
            let tmp = bind_exp(context, result, eloc, st, e);
            HE::BorrowLocal(mut_, tmp)
        }

        TE::Annotate(te, rhs_ty) => {
            let expected_ty = type_(context, *rhs_ty);
            return maybe_exp(context, result, Some(&expected_ty), *te);
        }
        TE::UnresolvedError => {
            assert!(context.has_errors());
            HE::UnresolvedError
        }
    };
    Reachable(H::exp(ty, sp(eloc, res)))
}

fn make_temps(context: &mut Context, loc: Loc, ty: H::Type) -> Vec<(Var, H::SingleType)> {
    use H::Type_ as T;
    match ty.value {
        T::Unit => vec![],
        T::Single(s) => vec![(context.new_temp(loc, s.clone()), s)],
        T::Multiple(ss) => ss
            .into_iter()
            .map(|s| (context.new_temp(loc, s.clone()), s))
            .collect(),
    }
}

fn bind_exp(
    context: &mut Context,
    result: &mut Block,
    loc: Loc,
    st: H::SingleType,
    e: H::Exp,
) -> Var {
    let tmp = context.new_temp(loc, st.clone());
    let lvalue = sp(tmp.loc(), H::LValue_::Var(tmp.clone(), Box::new(st)));
    let assign = sp(loc, H::Command_::Assign(vec![lvalue], Box::new(e)));
    result.push_back(sp(loc, H::Statement_::Command(assign)));
    tmp
}

fn bind_result(
    result: &mut Block,
    loc: Loc,
    tmps: Vec<(Var, H::SingleType)>,
    eres: ExpResult,
) -> ExpResult {
    use H::{Command_ as C, Statement_ as S, UnannotatedExp_ as E};

    match eres {
        res @ Unreachable { .. } => res,
        Reachable(e) => {
            let ty = e.ty.clone();
            let lvalues = tmps
                .iter()
                .map(|(v, st)| sp(v.loc(), H::LValue_::Var(v.clone(), Box::new(st.clone()))))
                .collect();
            let asgn = sp(loc, C::Assign(lvalues, Box::new(e)));
            result.push_back(sp(loc, S::Command(asgn)));

            let etemps = tmps
                .into_iter()
                .map(|(var, st)| {
                    let evar_ = sp(
                        var.loc(),
                        E::Move {
                            from_user: false,
                            var,
                        },
                    );
                    let ty = sp(st.loc, H::Type_::Single(st.clone()));
                    let evar = H::exp(ty, evar_);
                    H::ExpListItem::Single(evar, Box::new(st))
                })
                .collect();
            Reachable(H::exp(ty, sp(loc, E::ExpList(etemps))))
        }
    }
}

fn builtin(
    context: &mut Context,
    _result: &mut Block,
    _eloc: Loc,
    sp!(loc, tb_): T::BuiltinFunction,
    arg: Box<H::Exp>,
) -> H::UnannotatedExp_ {
    use H::{BuiltinFunction_ as HB, UnannotatedExp_ as E};
    use T::BuiltinFunction_ as TB;
    match tb_ {
        TB::MoveToSender(bt) => {
            let ty = base_type(context, bt);
            E::Builtin(Box::new(sp(loc, HB::MoveToSender(ty))), arg)
        }
        TB::MoveFrom(bt) => {
            let ty = base_type(context, bt);
            E::Builtin(Box::new(sp(loc, HB::MoveFrom(ty))), arg)
        }
        TB::BorrowGlobal(mut_, bt) => {
            let ty = base_type(context, bt);
            E::Builtin(Box::new(sp(loc, HB::BorrowGlobal(mut_, ty))), arg)
        }
        TB::Exists(bt) => {
            let ty = base_type(context, bt);
            E::Builtin(Box::new(sp(loc, HB::Exists(ty))), arg)
        }
        TB::Freeze(_bt) => E::Freeze(arg),
    }
}

//**************************************************************************************************
// Freezing
//**************************************************************************************************

#[derive(PartialEq, Eq)]
enum Freeze {
    NotNeeded,
    Point,
    Sub(Vec<bool>),
}

fn needs_freeze(sp!(_, actual): &H::Type, sp!(_, expected): &H::Type) -> Freeze {
    use H::Type_ as T;
    match (actual, expected) {
        (T::Unit, T::Unit) => Freeze::NotNeeded,
        (T::Single(actaul_s), T::Single(actual_e)) => {
            let needs = needs_freeze_single(actaul_s, actual_e);
            if needs {
                Freeze::Point
            } else {
                Freeze::NotNeeded
            }
        }
        (T::Multiple(actaul_ss), T::Multiple(actual_es)) => {
            assert!(actaul_ss.len() == actual_es.len());
            let points = actaul_ss
                .iter()
                .zip(actual_es)
                .map(|(a, e)| needs_freeze_single(a, e))
                .collect::<Vec<_>>();
            if points.iter().any(|needs| *needs) {
                Freeze::Sub(points)
            } else {
                Freeze::NotNeeded
            }
        }
        (actual, expected) => {
            unreachable!("ICE type checking failed, {:#?} !~ {:#?}", actual, expected)
        }
    }
}

fn needs_freeze_single(sp!(_, actual): &H::SingleType, sp!(_, expected): &H::SingleType) -> bool {
    use H::SingleType_ as T;
    match (actual, expected) {
        (T::Ref(true, _), T::Ref(false, _)) => true,
        _ => false,
    }
}

fn freeze(context: &mut Context, result: &mut Block, expected_type: &H::Type, e: H::Exp) -> H::Exp {
    use H::{Type_ as T, UnannotatedExp_ as E};

    match needs_freeze(&e.ty, expected_type) {
        Freeze::NotNeeded => e,
        Freeze::Point => freeze_point(e),

        Freeze::Sub(points) => {
            let loc = e.exp.loc;
            let actual_tys = match &e.ty.value {
                T::Multiple(v) => v.clone(),
                _ => unreachable!("ICE needs_freeze failed"),
            };
            assert!(actual_tys.len() == points.len());
            let new_temps = actual_tys
                .into_iter()
                .map(|ty| (context.new_temp(loc, ty.clone()), ty))
                .collect::<Vec<_>>();

            let lvalues = new_temps
                .iter()
                .cloned()
                .map(|(v, ty)| sp(loc, H::LValue_::Var(v, Box::new(ty))))
                .collect::<Vec<_>>();
            let assign = sp(loc, H::Command_::Assign(lvalues, Box::new(e)));
            result.push_back(sp(loc, H::Statement_::Command(assign)));

            let exps = new_temps
                .into_iter()
                .zip(points)
                .map(|((var, ty), needs_freeze)| {
                    let e_ = sp(
                        loc,
                        E::Move {
                            from_user: false,
                            var,
                        },
                    );
                    let e = H::exp(T::single(ty), e_);
                    if needs_freeze {
                        freeze_point(e)
                    } else {
                        e
                    }
                })
                .collect::<Vec<_>>();
            let ss = exps
                .iter()
                .map(|e| match &e.ty.value {
                    T::Single(s) => s.clone(),
                    _ => panic!("ICE list item has Multple type"),
                })
                .collect::<Vec<_>>();

            let tys = sp(loc, T::Multiple(ss.clone()));
            let items = exps
                .into_iter()
                .zip(ss)
                .map(|(e, s)| H::ExpListItem::Single(e, Box::new(s)))
                .collect();
            H::exp(tys, sp(loc, E::ExpList(items)))
        }
    }
}

fn freeze_point(e: H::Exp) -> H::Exp {
    let frozen_ty = freeze_ty(e.ty.clone());
    let eloc = e.exp.loc;
    let e_ = H::UnannotatedExp_::Freeze(Box::new(e));
    H::exp(frozen_ty, sp(eloc, e_))
}

fn freeze_ty(sp!(tloc, t): H::Type) -> H::Type {
    use H::Type_ as T;
    match t {
        T::Single(s) => sp(tloc, T::Single(freeze_single(s))),
        t => panic!("ICE MULTIPLE freezing anything but a mutable ref: {:#?}", t),
    }
}

fn freeze_single(sp!(sloc, s): H::SingleType) -> H::SingleType {
    use H::SingleType_ as S;
    match s {
        S::Ref(true, inner) => sp(sloc, S::Ref(false, inner)),
        t => panic!("ICE SINGLE freezing anything but a mutable ref: {:#?}", t),
    }
}
