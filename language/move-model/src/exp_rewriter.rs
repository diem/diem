// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeSet, VecDeque};

use crate::{
    ast::{Exp, ExpData, LocalVarDecl, MemoryLabel, Operation, TempIndex, Value},
    model::{GlobalEnv, ModuleId, NodeId, SpecVarId},
    symbol::Symbol,
    ty::Type,
};
use itertools::Itertools;

/// Rewriter for expressions, allowing to substitute locals by expressions as well as instantiate
/// types.
pub struct ExpRewriter<'env, 'rewriter> {
    env: &'env GlobalEnv,
    replacer: &'rewriter mut dyn FnMut(NodeId, RewriteTarget) -> Option<Exp>,
    type_args: &'rewriter [Type],
    shadowed: VecDeque<BTreeSet<Symbol>>,
}

/// A target for expression rewrites of either an `Exp::LocalVar` or an `Exp::Temporary`.
/// This is used as a parameter to the `replacer` function which defines the behavior of
/// the rewriter. Notice we use a single function entry point for `replacer` to allow it
/// to be a function which mutates it's context.
pub enum RewriteTarget {
    LocalVar(Symbol),
    Temporary(TempIndex),
}

impl<'env, 'rewriter> ExpRewriter<'env, 'rewriter> {
    /// Creates a new rewriter with the given replacer map.
    pub fn new<F>(env: &'env GlobalEnv, replacer: &'rewriter mut F) -> Self
    where
        F: FnMut(NodeId, RewriteTarget) -> Option<Exp>,
    {
        ExpRewriter {
            env,
            replacer,
            type_args: &[],
            shadowed: VecDeque::new(),
        }
    }

    /// Adds a type argument list to this rewriter. Generic type parameters are replaced by
    /// the given types.
    pub fn set_type_args(mut self, type_args: &'rewriter [Type]) -> Self {
        self.type_args = type_args;
        self
    }
}

impl<'env, 'rewriter> ExpRewriterFunctions for ExpRewriter<'env, 'rewriter> {
    fn rewrite_local_var(&mut self, id: NodeId, sym: Symbol) -> Option<Exp> {
        for vars in &self.shadowed {
            if vars.contains(&sym) {
                return None;
            }
        }
        (*self.replacer)(id, RewriteTarget::LocalVar(sym))
    }

    fn rewrite_temporary(&mut self, id: NodeId, idx: TempIndex) -> Option<Exp> {
        (*self.replacer)(id, RewriteTarget::Temporary(idx))
    }

    fn rewrite_node_id(&mut self, id: NodeId) -> Option<NodeId> {
        ExpData::instantiate_node(self.env, id, self.type_args)
    }
}

// ======================================================================================
// Expression rewriting trait

/// A general trait for expression rewriting.
///
/// This allows customization by re-implementing any of the `rewrite_local_var`,
/// `rewrite_temporary`, etc. functions. Each expression node has an equivalent of such
/// a function.
///
/// This rewriter takes care of preserving sharing between expressions: only expression trees
/// which are actually modified are reconstructed.
///
/// For most rewriting problems, there are already specializations of this trait, like `ExpRewriter`
/// in this module, and `Exp::rewrite` in the AST module.
///
/// When custom implementing this trait, consider the semantics of the generic logic used.
/// When any of the `rewrite_<exp-variant>` functions is called, any arguments have been already
/// recursively rewritten, inclusive of the passed node id. To implement a pre-descent
/// transformation, you need to implement the `rewrite_exp` function and after pre-processing,
/// continue (or not) descent with `rewrite_exp_descent` for sub-expressions.
#[allow(unused)] // for trait default parameters
pub trait ExpRewriterFunctions {
    /// Top-level entry for rewriting an expression. Can be re-implemented to do some
    /// pre/post processing embedding a call to `do_rewrite`.
    fn rewrite_exp(&mut self, exp: Exp) -> Exp {
        self.rewrite_exp_descent(exp)
    }

    fn rewrite_vec(&mut self, exps: &[Exp]) -> Vec<Exp> {
        exps.iter().map(|e| self.rewrite_exp(e.clone())).collect()
    }

    // Functions to specialize for the rewriting problem
    // --------------------------------------------------

    fn rewrite_enter_scope<'a>(&mut self, decls: impl Iterator<Item = &'a LocalVarDecl>) {}
    fn rewrite_exit_scope(&mut self) {}
    fn rewrite_node_id(&mut self, id: NodeId) -> Option<NodeId> {
        None
    }
    fn rewrite_local_var(&mut self, id: NodeId, sym: Symbol) -> Option<Exp> {
        None
    }
    fn rewrite_temporary(&mut self, id: NodeId, idx: TempIndex) -> Option<Exp> {
        None
    }
    fn rewrite_value(&mut self, id: NodeId, value: &Value) -> Option<Exp> {
        None
    }
    fn rewrite_spec_var(
        &mut self,
        id: NodeId,
        mid: ModuleId,
        vid: SpecVarId,
        label: &Option<MemoryLabel>,
    ) -> Option<Exp> {
        None
    }
    fn rewrite_call(&mut self, id: NodeId, oper: &Operation, args: &[Exp]) -> Option<Exp> {
        None
    }
    fn rewrite_invoke(&mut self, id: NodeId, target: &Exp, args: &[Exp]) -> Option<Exp> {
        None
    }
    fn rewrite_lambda(&mut self, id: NodeId, vars: &[LocalVarDecl], body: &Exp) -> Option<Exp> {
        None
    }
    fn rewrite_block(&mut self, id: NodeId, vars: &[LocalVarDecl], body: &Exp) -> Option<Exp> {
        None
    }
    fn rewrite_quant(
        &mut self,
        id: NodeId,
        vars: &[(LocalVarDecl, Exp)],
        triggers: &[Vec<Exp>],
        cond: &Option<Exp>,
        body: &Exp,
    ) -> Option<Exp> {
        None
    }
    fn rewrite_if_else(&mut self, id: NodeId, cond: &Exp, then: &Exp, else_: &Exp) -> Option<Exp> {
        None
    }

    // Core traversal functions, not intended to be re-implemented
    // -----------------------------------------------------------

    fn rewrite_exp_descent(&mut self, exp: Exp) -> Exp {
        use ExpData::*;
        match exp.as_ref() {
            Value(id, value) => {
                let (id_changed, new_id) = self.internal_rewrite_id(id);
                if let Some(new_exp) = self.rewrite_value(new_id, value) {
                    new_exp
                } else if id_changed {
                    Value(new_id, value.clone()).into_exp()
                } else {
                    exp
                }
            }
            LocalVar(id, sym) => {
                let (id_changed, new_id) = self.internal_rewrite_id(id);
                if let Some(new_exp) = self.rewrite_local_var(new_id, *sym) {
                    new_exp
                } else if id_changed {
                    LocalVar(new_id, *sym).into_exp()
                } else {
                    exp
                }
            }
            Temporary(id, idx) => {
                let (id_changed, new_id) = self.internal_rewrite_id(id);
                if let Some(new_exp) = self.rewrite_temporary(new_id, *idx) {
                    new_exp
                } else if id_changed {
                    Temporary(new_id, *idx).into_exp()
                } else {
                    exp
                }
            }
            Call(id, oper, args) => {
                let (id_changed, new_id) = self.internal_rewrite_id(id);
                let new_args_opt = self.internal_rewrite_vec(args);
                let args_ref = if let Some(new_args) = &new_args_opt {
                    new_args.as_slice()
                } else {
                    args.as_slice()
                };
                if let Some(new_exp) = self.rewrite_call(new_id, oper, args_ref) {
                    new_exp
                } else if new_args_opt.is_some() || id_changed {
                    let args_owned = if let Some(new_args) = new_args_opt {
                        new_args
                    } else {
                        args.to_owned()
                    };
                    Call(new_id, oper.clone(), args_owned).into_exp()
                } else {
                    exp
                }
            }
            Invoke(id, target, args) => {
                let (id_changed, new_id) = self.internal_rewrite_id(id);
                let (target_changed, new_target) = self.internal_rewrite_exp(target);
                let new_args_opt = self.internal_rewrite_vec(args);
                let args_ref = if let Some(new_args) = &new_args_opt {
                    new_args.as_slice()
                } else {
                    args.as_slice()
                };
                if let Some(new_exp) = self.rewrite_invoke(new_id, &new_target, args_ref) {
                    new_exp
                } else if id_changed || target_changed || new_args_opt.is_some() {
                    let args_owned = if let Some(new_args) = new_args_opt {
                        new_args
                    } else {
                        args.to_owned()
                    };
                    Invoke(new_id, new_target, args_owned).into_exp()
                } else {
                    exp
                }
            }
            Lambda(id, vars, body) => {
                let (id_changed, new_id) = self.internal_rewrite_id(id);
                let (vars_changed, new_vars) = self.internal_rewrite_decls(vars);
                self.rewrite_enter_scope(new_vars.iter());
                let (body_changed, new_body) = self.internal_rewrite_exp(body);
                self.rewrite_exit_scope();
                if let Some(new_exp) = self.rewrite_lambda(new_id, &new_vars, &new_body) {
                    new_exp
                } else if id_changed || vars_changed || body_changed {
                    Lambda(new_id, new_vars, new_body).into_exp()
                } else {
                    exp
                }
            }
            Block(id, vars, body) => {
                let (id_changed, new_id) = self.internal_rewrite_id(id);
                let (vars_changed, new_vars) = self.internal_rewrite_decls(vars);
                self.rewrite_enter_scope(new_vars.iter());
                let (body_changed, new_body) = self.internal_rewrite_exp(body);
                self.rewrite_exit_scope();
                if let Some(new_exp) = self.rewrite_block(new_id, &new_vars, &new_body) {
                    new_exp
                } else if id_changed || vars_changed || body_changed {
                    Block(new_id, new_vars, new_body).into_exp()
                } else {
                    exp
                }
            }
            Quant(id, kind, ranges, triggers, cond, body) => {
                let (id_changed, new_id) = self.internal_rewrite_id(id);
                let (ranges_changed, new_ranges) = self.internal_rewrite_quant_decls(ranges);
                self.rewrite_enter_scope(ranges.iter().map(|(decl, _)| decl));
                let mut triggers_changed = false;
                let new_triggers = triggers
                    .iter()
                    .map(|p| {
                        let (c, new_p) = self
                            .internal_rewrite_vec(p)
                            .map(|pr| (true, pr))
                            .unwrap_or_else(|| (false, p.clone()));
                        triggers_changed = triggers_changed || c;
                        new_p
                    })
                    .collect_vec();
                let mut cond_changed = false;
                let new_cond = cond.as_ref().map(|c| {
                    let (c, new_c) = self.internal_rewrite_exp(c);
                    cond_changed = c;
                    new_c
                });
                let (body_changed, new_body) = self.internal_rewrite_exp(body);
                self.rewrite_exit_scope();
                if let Some(new_exp) =
                    self.rewrite_quant(new_id, &new_ranges, &new_triggers, &new_cond, &new_body)
                {
                    new_exp
                } else if id_changed
                    || ranges_changed
                    || triggers_changed
                    || cond_changed
                    || body_changed
                {
                    Quant(new_id, *kind, new_ranges, new_triggers, new_cond, new_body).into_exp()
                } else {
                    exp
                }
            }
            IfElse(id, cond, then, else_) => {
                let (id_changed, new_id) = self.internal_rewrite_id(id);
                let (cond_changed, new_cond) = self.internal_rewrite_exp(cond);
                let (then_changed, new_then) = self.internal_rewrite_exp(then);
                let (else_changed, new_else) = self.internal_rewrite_exp(else_);
                if let Some(new_exp) = self.rewrite_if_else(new_id, &new_cond, &new_then, &new_else)
                {
                    new_exp
                } else if id_changed || cond_changed || then_changed || else_changed {
                    IfElse(new_id, new_cond, new_then, new_else).into_exp()
                } else {
                    exp
                }
            }
            // This can happen since we are calling the rewriter during type checking, and
            // we may have encountered an error which is represented as an Invalid expression.
            Invalid(id) => Invalid(*id).into_exp(),
        }
    }

    fn internal_rewrite_id(&mut self, id: &NodeId) -> (bool, NodeId) {
        if let Some(new_id) = self.rewrite_node_id(*id) {
            (true, new_id)
        } else {
            (false, *id)
        }
    }

    fn internal_rewrite_exp(&mut self, exp: &Exp) -> (bool, Exp) {
        let new_exp = self.rewrite_exp(exp.clone());
        (!ExpData::ptr_eq(exp, &new_exp), new_exp)
    }

    fn internal_rewrite_vec(&mut self, exps: &[Exp]) -> Option<Vec<Exp>> {
        // The vector rewrite works a bit different as we try to avoid constructing
        // new vectors if nothing changed, and optimize common cases of 0-3 arguments.
        match exps.len() {
            0 => None,
            1 => {
                let (c, e) = self.internal_rewrite_exp(&exps[0]);
                if c {
                    Some(vec![e])
                } else {
                    None
                }
            }
            2 => {
                let (c1, e1) = self.internal_rewrite_exp(&exps[0]);
                let (c2, e2) = self.internal_rewrite_exp(&exps[1]);
                if c1 || c2 {
                    Some(vec![e1, e2])
                } else {
                    None
                }
            }
            3 => {
                let (c1, e1) = self.internal_rewrite_exp(&exps[0]);
                let (c2, e2) = self.internal_rewrite_exp(&exps[1]);
                let (c3, e3) = self.internal_rewrite_exp(&exps[2]);
                if c1 || c2 || c3 {
                    Some(vec![e1, e2, e3])
                } else {
                    None
                }
            }
            _ => {
                // generic treatment
                let mut change = false;
                let mut res = vec![];
                for exp in exps {
                    let (c, new_exp) = self.internal_rewrite_exp(exp);
                    change = change || c;
                    res.push(new_exp)
                }
                if change {
                    Some(res)
                } else {
                    None
                }
            }
        }
    }

    fn internal_rewrite_decls(&mut self, decls: &[LocalVarDecl]) -> (bool, Vec<LocalVarDecl>) {
        let mut change = false;
        let new_decls = decls
            .iter()
            .map(|d| LocalVarDecl {
                id: {
                    let (c, id) = self.internal_rewrite_id(&d.id);
                    change = change || c;
                    id
                },
                name: d.name,
                binding: d.binding.as_ref().map(|e| {
                    let (c, new_e) = self.internal_rewrite_exp(e);
                    change = change || c;
                    new_e
                }),
            })
            .collect();
        (change, new_decls)
    }

    fn internal_rewrite_quant_decls(
        &mut self,
        decls: &[(LocalVarDecl, Exp)],
    ) -> (bool, Vec<(LocalVarDecl, Exp)>) {
        let mut change = false;
        let new_decls = decls
            .iter()
            .map(|(d, e)| {
                assert!(d.binding.is_none());
                (
                    LocalVarDecl {
                        id: {
                            let (c, id) = self.internal_rewrite_id(&d.id);
                            change = change || c;
                            id
                        },
                        name: d.name,
                        binding: None,
                    },
                    {
                        let (c, new_e) = self.internal_rewrite_exp(e);
                        change = change || c;
                        new_e
                    },
                )
            })
            .collect();
        (change, new_decls)
    }
}
