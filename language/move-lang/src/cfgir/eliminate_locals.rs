// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::cfg::BlockCFG;
use crate::parser::ast::Var;
use std::collections::BTreeSet;

pub fn optimize(cfg: &mut BlockCFG) {
    super::remove_no_ops::optimize(cfg);
    loop {
        let ssa_temps = {
            let s = count(cfg);
            if s.is_empty() {
                return;
            }
            s
        };
        eliminate(cfg, ssa_temps);
        super::remove_no_ops::optimize(cfg);
    }
}

//**************************************************************************************************
// Count assignment and usage
//**************************************************************************************************

fn count(cfg: &BlockCFG) -> BTreeSet<Var> {
    let mut context = count::Context::new();
    for block in cfg.blocks().values() {
        for cmd in block {
            count::command(&mut context, cmd)
        }
    }
    context.finish()
}

mod count {
    use crate::{
        cfgir::ast::*,
        parser::ast::{BinOp, UnaryOp, Var},
        shared::*,
    };
    use std::collections::{BTreeMap, BTreeSet};

    pub struct Context {
        assigned: BTreeMap<Var, Option<usize>>,
        used: BTreeMap<Var, Option<usize>>,
    }

    impl Context {
        pub fn new() -> Self {
            Context {
                assigned: BTreeMap::new(),
                used: BTreeMap::new(),
            }
        }

        fn assign(&mut self, var: &Var, substitutable: bool) {
            if !substitutable {
                self.assigned.insert(var.clone(), None);
                return;
            }

            if let Some(count) = self.assigned.entry(var.clone()).or_insert_with(|| Some(0)) {
                *count += 1
            }
        }

        fn used(&mut self, var: &Var, is_borrow_local: bool) {
            if is_borrow_local {
                self.used.insert(var.clone(), None);
                return;
            }

            if let Some(count) = self.used.entry(var.clone()).or_insert_with(|| Some(0)) {
                *count += 1
            }
        }

        pub fn finish(self) -> BTreeSet<Var> {
            let Context { assigned, used } = self;
            assigned
                .into_iter()
                .filter(|(_v, count)| count.map(|c| c == 1).unwrap_or(false))
                .map(|(v, _count)| v)
                .filter(|v| {
                    used.get(v)
                        .unwrap_or(&None)
                        .map(|c| c == 1)
                        .unwrap_or(false)
                })
                .collect()
        }
    }

    pub fn command(context: &mut Context, sp!(_, cmd_): &Command) {
        use Command_ as C;
        match cmd_ {
            C::Assign(ls, e) => {
                exp(context, e);
                let substitutable_rvalues = can_subst_exp(ls.len(), e);
                lvalues(context, ls, substitutable_rvalues);
            }
            C::Mutate(el, er) => {
                exp(context, er);
                exp(context, el)
            }
            C::Return(e)
            | C::Abort(e)
            | C::IgnoreAndPop { exp: e, .. }
            | C::JumpIf { cond: e, .. } => exp(context, e),

            C::Jump(_) => (),
        }
    }

    fn lvalues(context: &mut Context, ls: &[LValue], substitutable_rvalues: Vec<bool>) {
        assert!(ls.len() == substitutable_rvalues.len());
        ls.iter()
            .zip(substitutable_rvalues)
            .for_each(|(l, substitutable)| lvalue(context, l, substitutable))
    }

    fn lvalue(context: &mut Context, sp!(_, l_): &LValue, substitutable: bool) {
        use LValue_ as L;
        match l_ {
            L::Ignore | L::Unpack(_, _, _) => (),
            L::Var(v, _) => context.assign(v, substitutable),
        }
    }

    fn exp(context: &mut Context, parent_e: &Exp) {
        use UnannotatedExp_ as E;
        match &parent_e.exp.value {
            E::Unit | E::Value(_) | E::UnresolvedError => (),

            E::BorrowLocal(_, var) => context.used(var, true),

            E::Copy { var, .. } | E::Move { var, .. } => context.used(var, false),

            E::ModuleCall(mcall) => exp(context, &mcall.arguments),
            E::Builtin(_, e)
            | E::Freeze(e)
            | E::Dereference(e)
            | E::UnaryExp(_, e)
            | E::Borrow(_, e, _)
            | E::Cast(e, _) => exp(context, e),

            E::BinopExp(e1, _, e2) => {
                exp(context, e1);
                exp(context, e2)
            }

            E::Pack(_, _, fields) => fields.iter().for_each(|(_, _, e)| exp(context, e)),

            E::ExpList(es) => es.iter().for_each(|item| exp_list_item(context, item)),

            E::Unreachable => panic!("ICE should not analyze dead code"),
        }
    }

    fn exp_list_item(context: &mut Context, item: &ExpListItem) {
        match item {
            ExpListItem::Single(e, _) | ExpListItem::Splat(_, e, _) => exp(context, e),
        }
    }

    fn can_subst_exp(lvalue_len: usize, exp: &Exp) -> Vec<bool> {
        use ExpListItem as I;
        use UnannotatedExp_ as E;
        match (lvalue_len, &exp.exp.value) {
            (0, _) => vec![],
            (1, _) => vec![can_subst_exp_single(exp)],
            (_, E::ExpList(es))
                if es.iter().all(|item| match item {
                    I::Splat(_, _, _) => false,
                    I::Single(_, _) => true,
                }) =>
            {
                es.iter()
                    .map(|item| match item {
                        I::Single(e, _) => can_subst_exp_single(e),
                        I::Splat(_, _, _) => unreachable!(),
                    })
                    .collect()
            }
            (_, _) => (0..lvalue_len).map(|_| false).collect(),
        }
    }

    fn can_subst_exp_single(parent_e: &Exp) -> bool {
        use UnannotatedExp_ as E;
        match &parent_e.exp.value {
            E::UnresolvedError
            | E::BorrowLocal(_, _)
            | E::Copy { .. }
            | E::Builtin(_, _)
            | E::Freeze(_)
            | E::Dereference(_)
            | E::Move { .. }
            | E::Borrow(_, _, _) => false,

            E::Unit | E::Value(_) => true,

            E::Cast(e, _) => can_subst_exp_single(e),
            E::UnaryExp(op, e) => can_subst_exp_unary(op) && can_subst_exp_single(e),
            E::BinopExp(e1, op, e2) => {
                can_subst_exp_binary(op) && can_subst_exp_single(e1) && can_subst_exp_single(e2)
            }
            E::ModuleCall(mcall) => can_subst_exp_module_call(mcall),
            E::ExpList(es) => es.iter().all(|i| can_subst_exp_item(i)),
            E::Pack(_, _, fields) => fields.iter().all(|(_, _, e)| can_subst_exp_single(e)),

            E::Unreachable => panic!("ICE should not analyze dead code"),
        }
    }

    fn can_subst_exp_unary(sp!(_, op_): &UnaryOp) -> bool {
        op_.is_pure()
    }

    fn can_subst_exp_binary(sp!(_, op_): &BinOp) -> bool {
        op_.is_pure()
    }

    fn can_subst_exp_module_call(mcall: &ModuleCall) -> bool {
        use crate::shared::fake_natives::transaction as TXN;
        let ModuleCall {
            module,
            name,
            arguments,
            ..
        } = mcall;
        let a_m_f = (
            &module.0.value.address,
            module.0.value.name.value(),
            name.value(),
        );
        let call_is_pure = match a_m_f {
            (&Address::LIBRA_CORE, TXN::MOD, TXN::ASSERT) => panic!("ICE should have been inlined"),
            (&Address::LIBRA_CORE, TXN::MOD, TXN::MAX_GAS)
            | (&Address::LIBRA_CORE, TXN::MOD, TXN::SENDER)
            | (&Address::LIBRA_CORE, TXN::MOD, TXN::SEQUENCE_NUM)
            | (&Address::LIBRA_CORE, TXN::MOD, TXN::PUBLIC_KEY)
            | (&Address::LIBRA_CORE, TXN::MOD, TXN::GAS_PRICE) => true,
            _ => false,
        };
        call_is_pure && can_subst_exp_single(arguments)
    }

    fn can_subst_exp_item(item: &ExpListItem) -> bool {
        use ExpListItem as I;
        match item {
            I::Single(e, _) => can_subst_exp_single(e),
            I::Splat(_, es, _) => can_subst_exp_single(es),
        }
    }
}

//**************************************************************************************************
// Eliminate
//**************************************************************************************************

fn eliminate(cfg: &mut BlockCFG, ssa_temps: BTreeSet<Var>) {
    let context = &mut eliminate::Context::new(ssa_temps);
    loop {
        for block in cfg.blocks_mut().values_mut() {
            for cmd in block {
                eliminate::command(context, cmd)
            }
        }
        if context.finished() {
            return;
        }
    }
}

mod eliminate {
    use crate::{cfgir::ast, cfgir::ast::*, parser::ast::Var, shared::*};
    use std::collections::{BTreeMap, BTreeSet};

    pub struct Context {
        eliminated: BTreeMap<Var, Exp>,
        ssa_temps: BTreeSet<Var>,
    }

    impl Context {
        pub fn new(ssa_temps: BTreeSet<Var>) -> Self {
            Context {
                ssa_temps,
                eliminated: BTreeMap::new(),
            }
        }

        pub fn finished(&self) -> bool {
            self.eliminated.is_empty() && self.ssa_temps.is_empty()
        }
    }

    pub fn command(context: &mut Context, sp!(_, cmd_): &mut Command) {
        use Command_ as C;
        match cmd_ {
            C::Assign(ls, e) => {
                exp(context, e);
                let eliminated = lvalues(context, ls);
                remove_eliminated(context, eliminated, e)
            }
            C::Mutate(el, er) => {
                exp(context, er);
                exp(context, el)
            }
            C::Return(e)
            | C::Abort(e)
            | C::IgnoreAndPop { exp: e, .. }
            | C::JumpIf { cond: e, .. } => exp(context, e),

            C::Jump(_) => (),
        }
    }

    enum LRes {
        Same(LValue),
        Elim(Var),
    }

    fn lvalues(context: &mut Context, ls: &mut Vec<LValue>) -> Vec<Option<Var>> {
        let old = std::mem::replace(ls, vec![]);
        old.into_iter()
            .map(|l| match lvalue(context, l) {
                LRes::Same(lvalue) => {
                    ls.push(lvalue);
                    None
                }
                LRes::Elim(v) => Some(v),
            })
            .collect()
    }

    fn lvalue(context: &mut Context, sp!(loc, l_): LValue) -> LRes {
        use LValue_ as L;
        match l_ {
            l_ @ L::Ignore | l_ @ L::Unpack(_, _, _) => LRes::Same(sp(loc, l_)),
            L::Var(v, t) => {
                let contained = context.ssa_temps.remove(&v);
                if contained {
                    LRes::Elim(v)
                } else {
                    LRes::Same(sp(loc, L::Var(v, t)))
                }
            }
        }
    }

    fn exp(context: &mut Context, parent_e: &mut Exp) {
        use UnannotatedExp_ as E;
        match &mut parent_e.exp.value {
            E::Copy { var, .. } | E::Move { var, .. } => {
                if let Some(replacement) = context.eliminated.remove(var) {
                    *parent_e = replacement
                }
            }

            E::Unit | E::Value(_) | E::UnresolvedError | E::BorrowLocal(_, _) => (),

            E::ModuleCall(mcall) => exp(context, &mut mcall.arguments),
            E::Builtin(_, e)
            | E::Freeze(e)
            | E::Dereference(e)
            | E::UnaryExp(_, e)
            | E::Borrow(_, e, _)
            | E::Cast(e, _) => exp(context, e),

            E::BinopExp(e1, _, e2) => {
                exp(context, e1);
                exp(context, e2)
            }

            E::Pack(_, _, fields) => fields.iter_mut().for_each(|(_, _, e)| exp(context, e)),

            E::ExpList(es) => es.iter_mut().for_each(|item| exp_list_item(context, item)),

            E::Unreachable => panic!("ICE should not analyze dead code"),
        }
    }

    fn exp_list_item(context: &mut Context, item: &mut ExpListItem) {
        match item {
            ExpListItem::Single(e, _) | ExpListItem::Splat(_, e, _) => exp(context, e),
        }
    }

    fn remove_eliminated(context: &mut Context, mut eliminated: Vec<Option<Var>>, e: &mut Exp) {
        if eliminated.iter().all(|opt| opt.is_none()) {
            return;
        }

        match eliminated.len() {
            0 => (),
            1 => remove_eliminated_single(context, eliminated.pop().unwrap().unwrap(), e),

            _ => {
                let tys = match &mut e.ty.value {
                    Type_::Multiple(tys) => tys,
                    _ => panic!("ICE local elimination type mismatch"),
                };
                let es = match &mut e.exp.value {
                    UnannotatedExp_::ExpList(es) => es,
                    _ => panic!("ICE local elimination type mismatch"),
                };
                let old_tys = std::mem::replace(tys, vec![]);
                let old_es = std::mem::replace(es, vec![]);
                for ((mut item, ty), elim_opt) in old_es.into_iter().zip(old_tys).zip(eliminated) {
                    let e = match &mut item {
                        ExpListItem::Single(e, _) => e,
                        ExpListItem::Splat(_, _, _) => {
                            panic!("ICE local elimination filtering failed")
                        }
                    };
                    match elim_opt {
                        None => {
                            tys.push(ty);
                            es.push(item)
                        }
                        Some(v) => {
                            remove_eliminated_single(context, v, e);
                            match &e.ty.value {
                                Type_::Unit => (),
                                Type_::Single(_) => {
                                    tys.push(ty);
                                    es.push(item)
                                }
                                Type_::Multiple(_) => {
                                    panic!("ICE local elimination replacement type mismatch")
                                }
                            }
                        }
                    }
                }
                if es.is_empty() {
                    *e = unit(e.exp.loc)
                }
            }
        }
    }

    fn remove_eliminated_single(context: &mut Context, v: Var, e: &mut Exp) {
        let old = std::mem::replace(e, unit(e.exp.loc));
        context.eliminated.insert(v, old);
    }

    fn unit(loc: Loc) -> Exp {
        ast::exp(sp(loc, Type_::Unit), sp(loc, UnannotatedExp_::Unit))
    }
}
