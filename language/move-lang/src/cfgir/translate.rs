// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cfgir::{
        self,
        ast::{self as G, BasicBlock, Blocks, Label},
        cfg::BlockCFG,
    },
    errors::Errors,
    hlir::ast as H,
    parser::ast::{FunctionName, ModuleIdent, StructName},
    shared::unique_map::UniqueMap,
};
use move_ir_types::location::*;
use std::{collections::BTreeSet, mem};

//**************************************************************************************************
// Context
//**************************************************************************************************

struct Context {
    errors: Errors,
    start: Option<Label>,
    loop_begin: Option<Label>,
    loop_end: Option<Label>,
    next_label: Option<Label>,
    label_count: usize,
    blocks: Blocks,
    infinite_loop_starts: BTreeSet<Label>,
}

impl Context {
    pub fn new(_prog: &H::Program, errors: Errors) -> Self {
        Context {
            errors,
            next_label: None,
            loop_begin: None,
            loop_end: None,
            start: None,
            label_count: 0,
            blocks: Blocks::new(),
            infinite_loop_starts: BTreeSet::new(),
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

    fn new_label(&mut self) -> Label {
        let count = self.label_count;
        self.label_count += 1;
        Label(count)
    }

    pub fn finish_blocks(&mut self) -> (Label, Blocks, BTreeSet<Label>) {
        self.next_label = None;
        let start = mem::replace(&mut self.start, None);
        let blocks = mem::replace(&mut self.blocks, Blocks::new());
        let infinite_loop_starts = mem::replace(&mut self.infinite_loop_starts, BTreeSet::new());
        self.label_count = 0;
        self.loop_begin = None;
        self.loop_end = None;
        (start.unwrap(), blocks, infinite_loop_starts)
    }
}

//**************************************************************************************************
// Entry
//**************************************************************************************************

pub fn program(errors: Errors, prog: H::Program) -> (G::Program, Errors) {
    let mut context = Context::new(&prog, errors);
    let modules = modules(&mut context, prog.modules);
    let main = prog
        .main
        .map(|(addr, n, fdef)| (addr, n.clone(), function(&mut context, n, fdef)));

    (G::Program { modules, main }, context.get_errors())
}

fn modules(
    context: &mut Context,
    hmodules: UniqueMap<ModuleIdent, H::ModuleDefinition>,
) -> UniqueMap<ModuleIdent, G::ModuleDefinition> {
    let modules = hmodules
        .into_iter()
        .map(|(mname, m)| module(context, mname, m));
    UniqueMap::maybe_from_iter(modules).unwrap()
}

fn module(
    context: &mut Context,
    module_ident: ModuleIdent,
    mdef: H::ModuleDefinition,
) -> (ModuleIdent, G::ModuleDefinition) {
    let is_source_module = mdef.is_source_module;
    let dependency_order = mdef.dependency_order;
    let structs = mdef.structs.map(|name, s| struct_def(context, name, s));
    let functions = mdef.functions.map(|name, f| function(context, name, f));
    (
        module_ident,
        G::ModuleDefinition {
            is_source_module,
            dependency_order,
            structs,
            functions,
        },
    )
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

fn function(context: &mut Context, _name: FunctionName, f: H::Function) -> G::Function {
    let visibility = f.visibility;
    let signature = function_signature(context, f.signature);
    let acquires = f.acquires;
    let body = function_body(context, &signature, &acquires, f.body);
    G::Function {
        visibility,
        signature,
        acquires,
        body,
    }
}

fn function_signature(context: &mut Context, sig: H::FunctionSignature) -> G::FunctionSignature {
    let type_parameters = sig.type_parameters;
    let parameters = sig
        .parameters
        .into_iter()
        .map(|(v, ty)| (v, single_type(context, ty)))
        .collect();
    let return_type = type_(context, sig.return_type);
    G::FunctionSignature {
        type_parameters,
        parameters,
        return_type,
    }
}

fn function_body(
    context: &mut Context,
    signature: &G::FunctionSignature,
    acquires: &BTreeSet<StructName>,
    sp!(loc, tb_): H::FunctionBody,
) -> G::FunctionBody {
    use G::FunctionBody_ as GB;
    use H::FunctionBody_ as HB;
    assert!(context.next_label.is_none());
    assert!(context.start.is_none());
    assert!(context.blocks.is_empty());
    assert!(context.loop_begin.is_none());
    assert!(context.loop_end.is_none());
    assert!(context.infinite_loop_starts.is_empty());
    let b_ = match tb_ {
        HB::Native => GB::Native,
        HB::Defined { locals, body } => {
            let locals = locals.map(|_, st| single_type(context, st));
            function_block(context, body);
            let (start, mut blocks, infinite_loop_starts) = context.finish_blocks();

            let (mut cfg, errors) = BlockCFG::new(start, &mut blocks);
            for e in errors {
                context.error(e);
            }

            cfgir::refine_inference_and_verify(
                &mut context.errors,
                signature,
                acquires,
                &locals,
                &mut cfg,
                &infinite_loop_starts,
            );
            cfgir::optimize(signature, &locals, &mut cfg);

            GB::Defined {
                locals,
                start,
                blocks,
            }
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
    sdef: H::StructDefinition,
) -> G::StructDefinition {
    let resource_opt = sdef.resource_opt;
    let type_parameters = sdef.type_parameters;
    let fields = match sdef.fields {
        H::StructFields::Native(loc) => G::StructFields::Native(loc),
        H::StructFields::Defined(m) => G::StructFields::Defined(
            m.into_iter()
                .map(|(f, t)| (f, base_type(context, t)))
                .collect(),
        ),
    };
    G::StructDefinition {
        resource_opt,
        type_parameters,
        fields,
    }
}

//**************************************************************************************************
// Types
//**************************************************************************************************

fn base_types<R: std::iter::FromIterator<G::BaseType>>(
    context: &Context,
    tys: impl IntoIterator<Item = H::BaseType>,
) -> R {
    tys.into_iter().map(|t| base_type(context, t)).collect()
}

fn base_type(context: &Context, sp!(loc, nb_): H::BaseType) -> G::BaseType {
    use G::BaseType_ as GB;
    use H::BaseType_ as HB;
    let b_ = match nb_ {
        HB::Apply(k, n, nbs) => GB::Apply(k, n, base_types(context, nbs)),
        HB::Param(tp) => GB::Param(tp),
        HB::UnresolvedError => GB::UnresolvedError,
    };
    sp(loc, b_)
}

fn single_types(context: &Context, ss: Vec<H::SingleType>) -> Vec<G::SingleType> {
    ss.into_iter().map(|s| single_type(context, s)).collect()
}

fn single_type(context: &Context, sp!(loc, ns_): H::SingleType) -> G::SingleType {
    use G::SingleType_ as GS;
    use H::SingleType_ as HS;
    let s_ = match ns_ {
        HS::Ref(mut_, nb) => GS::Ref(mut_, base_type(context, nb)),
        HS::Base(nb) => GS::Base(base_type(context, nb)),
    };
    sp(loc, s_)
}

fn type_(context: &Context, sp!(loc, et_): H::Type) -> G::Type {
    use G::Type_ as GT;
    use H::Type_ as HT;
    let t_ = match et_ {
        HT::Unit => GT::Unit,
        HT::Single(s) => GT::Single(single_type(context, s)),
        HT::Multiple(ss) => GT::Multiple(single_types(context, ss)),
    };
    sp(loc, t_)
}

//**************************************************************************************************
// Statements
//**************************************************************************************************

fn function_block(context: &mut Context, blocks: H::Block) {
    let start = context.new_label();
    context.start = Some(start);
    block(context, start, blocks)
}

fn block(context: &mut Context, mut cur_label: Label, blocks: H::Block) {
    use G::Command_ as C;

    assert!(!blocks.is_empty());
    let loc = blocks.back().unwrap().loc;
    let mut basic_block = block_(context, &mut cur_label, blocks);

    // return if we ended with did not end with a command
    if basic_block.is_empty() {
        return;
    }

    match context.next_label {
        Some(next) if !basic_block.back().unwrap().value.is_terminal() => {
            basic_block.push_back(sp(loc, C::Jump(next)));
        }
        _ => (),
    }
    context.blocks.insert(cur_label, basic_block);
}

fn block_(context: &mut Context, cur_label: &mut Label, blocks: H::Block) -> BasicBlock {
    use G::Command_ as C;
    use H::Statement_ as S;

    assert!(!blocks.is_empty());
    let mut basic_block = BasicBlock::new();

    macro_rules! finish_block {
        (next_label: $next_label:expr) => {{
            let lbl = mem::replace(cur_label, $next_label);
            let bb = mem::replace(&mut basic_block, BasicBlock::new());
            context.blocks.insert(lbl, bb);
        }};
    }

    macro_rules! loop_block {
        (begin: $begin:expr, end: $end:expr, body: $body:expr, $block:expr) => {{
            let begin = $begin;
            let old_begin = mem::replace(&mut context.loop_begin, Some(begin));
            let old_end = mem::replace(&mut context.loop_end, Some($end));
            let old_next = mem::replace(&mut context.next_label, Some(begin));
            block(context, $body, $block);
            context.next_label = old_next;
            context.loop_end = old_end;
            context.loop_begin = old_begin;
        }};
    }

    for sp!(loc, stmt_) in blocks {
        match stmt_ {
            S::Command(c) => {
                let cmd = command(context, c);
                let is_terminal = cmd.value.is_terminal();
                basic_block.push_back(cmd);
                if is_terminal {
                    finish_block!(next_label: context.new_label());
                }
            }
            S::IfElse {
                cond: hcond,
                if_block,
                else_block,
            } => {
                let if_true = context.new_label();
                let if_false = context.new_label();
                let next_label = context.new_label();

                // If cond
                let cond = exp_(context, *hcond);
                let jump_if = C::JumpIf {
                    cond,
                    if_true,
                    if_false,
                };
                basic_block.push_back(sp(loc, jump_if));
                finish_block!(next_label: next_label);

                // If branches
                let old_next = mem::replace(&mut context.next_label, Some(next_label));
                block(context, if_true, if_block);
                block(context, if_false, else_block);
                context.next_label = old_next;
            }
            S::While {
                cond: (hcond_block, hcond_exp),
                block: loop_block,
            } => {
                let loop_cond = context.new_label();
                let loop_body = context.new_label();
                let loop_end = context.new_label();

                // Jump to loop condition
                basic_block.push_back(sp(loc, C::Jump(loop_cond)));
                finish_block!(next_label: loop_cond);

                // Loop condition and case to jump into loop or end
                if !hcond_block.is_empty() {
                    assert!(basic_block.is_empty());
                    basic_block = block_(context, cur_label, hcond_block);
                }
                let cond = exp_(context, *hcond_exp);
                let jump_if = C::JumpIf {
                    cond,
                    if_true: loop_body,
                    if_false: loop_end,
                };
                basic_block.push_back(sp(loc, jump_if));
                finish_block!(next_label: loop_end);

                // Loop body
                loop_block!(begin: loop_cond, end: loop_end, body: loop_body, loop_block)
            }

            S::Loop {
                block: loop_block,
                has_break,
                has_return_abort,
            } => {
                let loop_body = context.new_label();
                let loop_end = context.new_label();
                assert!(cur_label.0 < loop_body.0);
                assert!(loop_body.0 < loop_end.0);

                if !has_return_abort && !has_break {
                    context.infinite_loop_starts.insert(loop_body);
                }

                // Jump to loop
                basic_block.push_back(sp(loc, C::Jump(loop_body)));
                finish_block!(next_label: loop_end);

                // Loop body
                loop_block!(begin: loop_body, end: loop_end, body: loop_body, loop_block)
            }
        }
    }

    basic_block
}

fn command(context: &Context, sp!(loc, hc_): H::Command) -> G::Command {
    use G::Command_ as C;
    use H::Command_ as HC;

    let c_ = match hc_ {
        HC::Assign(ls, e) => C::Assign(lvalues(context, ls), exp(context, e)),
        HC::Mutate(el, er) => C::Mutate(exp(context, el), exp(context, er)),
        HC::Abort(e) => C::Abort(exp_(context, e)),
        HC::Return(e) => C::Return(exp_(context, e)),
        HC::Continue => C::Jump(context.loop_begin.clone().unwrap()),
        HC::Break => C::Jump(context.loop_end.clone().unwrap()),
        HC::IgnoreAndPop { pop_num, exp: e } => C::IgnoreAndPop {
            pop_num,
            exp: exp_(context, e),
        },
    };
    sp(loc, c_)
}

fn lvalues(context: &Context, ls: Vec<H::LValue>) -> Vec<G::LValue> {
    ls.into_iter().map(|l| lvalue(context, l)).collect()
}

fn lvalue(context: &Context, sp!(loc, hl_): H::LValue) -> G::LValue {
    use G::LValue_ as L;
    use H::LValue_ as HL;
    let l_ = match hl_ {
        HL::Ignore => L::Ignore,
        HL::Var(v, st) => L::Var(v, Box::new(single_type(context, *st))),
        HL::Unpack(s, bs, fields) => L::Unpack(
            s,
            base_types(context, bs),
            fields
                .into_iter()
                .map(|(f, fl)| (f, lvalue(context, fl)))
                .collect(),
        ),
    };
    sp(loc, l_)
}

fn exp(context: &Context, he: Box<H::Exp>) -> Box<G::Exp> {
    Box::new(exp_(context, *he))
}

fn exp_(context: &Context, he: H::Exp) -> G::Exp {
    use G::UnannotatedExp_ as E;
    use H::UnannotatedExp_ as HE;
    let ty = type_(context, he.ty);
    let sp!(loc, he_) = he.exp;
    let e_ = match he_ {
        HE::Unit => E::Unit,
        HE::Value(v) => E::Value(v),
        HE::Move { from_user, var } => E::Move { from_user, var },
        HE::Copy { from_user, var } => E::Copy { from_user, var },
        HE::ModuleCall(hcall) => {
            let hcall = *hcall;
            let module = hcall.module;
            let name = hcall.name;
            let type_arguments = base_types(context, hcall.type_arguments);
            let arguments = exp(context, hcall.arguments);
            let acquires = hcall.acquires;
            let mcall = G::ModuleCall {
                module,
                name,
                type_arguments,
                arguments,
                acquires,
            };
            E::ModuleCall(Box::new(mcall))
        }
        HE::Builtin(bf, e) => E::Builtin(builtin(context, *bf), exp(context, e)),
        HE::Freeze(e) => E::Freeze(exp(context, e)),
        HE::Dereference(e) => E::Dereference(exp(context, e)),
        HE::UnaryExp(u, e) => E::UnaryExp(u, exp(context, e)),
        HE::BinopExp(el, b, er) => E::BinopExp(exp(context, el), b, exp(context, er)),
        HE::Pack(s, bs, hfields) => {
            let fields = hfields
                .into_iter()
                .map(|(f, bt, e)| (f, base_type(context, bt), exp_(context, e)))
                .collect();
            E::Pack(s, base_types(context, bs), fields)
        }
        HE::ExpList(es) => {
            assert!(!es.is_empty());
            E::ExpList(exp_list(context, es))
        }
        HE::Borrow(mut_, e, f) => E::Borrow(mut_, exp(context, e), f),
        HE::BorrowLocal(mut_, v) => E::BorrowLocal(mut_, v),
        HE::UnresolvedError => {
            assert!(context.has_errors());
            E::UnresolvedError
        }
        HE::Cast(e, bt) => E::Cast(exp(context, e), bt),
        HE::Unreachable => E::Unreachable,
    };
    G::exp(ty, sp(loc, e_))
}

fn builtin(context: &Context, sp!(loc, hf_): H::BuiltinFunction) -> Box<G::BuiltinFunction> {
    use G::BuiltinFunction_ as B;
    use H::BuiltinFunction_ as HB;
    let f_ = match hf_ {
        HB::MoveToSender(bt) => B::MoveToSender(base_type(context, bt)),
        HB::MoveFrom(bt) => B::MoveFrom(base_type(context, bt)),
        HB::BorrowGlobal(mut_, bt) => B::BorrowGlobal(mut_, base_type(context, bt)),
        HB::Exists(bt) => B::Exists(base_type(context, bt)),
    };
    Box::new(sp(loc, f_))
}

fn exp_list(context: &Context, items: Vec<H::ExpListItem>) -> Vec<G::ExpListItem> {
    items
        .into_iter()
        .map(|item| exp_list_item(context, item))
        .collect()
}

fn exp_list_item(context: &Context, item: H::ExpListItem) -> G::ExpListItem {
    use G::ExpListItem as I;
    use H::ExpListItem as HI;
    match item {
        HI::Single(e, st) => I::Single(exp_(context, e), Box::new(single_type(context, *st))),
        HI::Splat(loc, e, ss) => I::Splat(loc, exp_(context, e), single_types(context, ss)),
    }
}
