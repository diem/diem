// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeSet, VecDeque};

use crate::{
    ast::{Exp, LocalVarDecl, TempIndex},
    model::{GlobalEnv, NodeId},
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

    /// Runs the rewriter.
    pub fn rewrite(&mut self, exp: &Exp) -> Exp {
        use crate::ast::Exp::*;
        match exp {
            LocalVar(id, sym) => self.replace_local(*id, *sym),
            Temporary(id, idx) => self.replace_temporary(*id, *idx),
            Call(id, oper, args) => Call(
                self.rewrite_attrs(*id),
                oper.clone(),
                self.rewrite_vec(args),
            ),
            Invoke(id, target, args) => Invoke(
                self.rewrite_attrs(*id),
                Box::new(self.rewrite(target)),
                self.rewrite_vec(args),
            ),
            Lambda(id, vars, body) => {
                let vars = self.rewrite_decls(vars);
                self.shadowed
                    .push_front(vars.iter().map(|decl| decl.name).collect());
                let res = Lambda(self.rewrite_attrs(*id), vars, Box::new(self.rewrite(body)));
                self.shadowed.pop_front();
                res
            }
            Quant(id, kind, ranges, triggers, condition, body) => {
                let ranges = self.rewrite_quant_decls(ranges);
                self.shadowed
                    .push_front(ranges.iter().map(|(decl, _)| decl.name).collect());
                let res = Quant(
                    self.rewrite_attrs(*id),
                    *kind,
                    ranges,
                    triggers
                        .iter()
                        .map(|trigger| trigger.iter().map(|exp| self.rewrite(&*exp)).collect())
                        .collect(),
                    condition.as_ref().map(|exp| Box::new(self.rewrite(&*exp))),
                    Box::new(self.rewrite(body)),
                );
                self.shadowed.pop_front();
                res
            }
            Block(id, vars, body) => {
                let vars = self.rewrite_decls(vars);
                self.shadowed
                    .push_front(vars.iter().map(|decl| decl.name).collect());
                let res = Block(self.rewrite_attrs(*id), vars, Box::new(self.rewrite(body)));
                self.shadowed.pop_front();
                res
            }
            IfElse(id, cond, then, else_) => IfElse(
                self.rewrite_attrs(*id),
                Box::new(self.rewrite(cond)),
                Box::new(self.rewrite(then)),
                Box::new(self.rewrite(else_)),
            ),
            Invalid(..) | Value(..) | SpecVar(..) => exp.clone(),
        }
    }

    fn rewrite_decls(&mut self, decls: &[LocalVarDecl]) -> Vec<LocalVarDecl> {
        decls
            .iter()
            .map(|d| LocalVarDecl {
                id: self.rewrite_attrs(d.id),
                name: d.name,
                binding: d.binding.as_ref().map(|e| self.rewrite(e)),
            })
            .collect()
    }

    fn rewrite_quant_decls(&mut self, decls: &[(LocalVarDecl, Exp)]) -> Vec<(LocalVarDecl, Exp)> {
        decls
            .iter()
            .map(|(d, e)| {
                (
                    LocalVarDecl {
                        id: self.rewrite_attrs(d.id),
                        name: d.name,
                        binding: d.binding.as_ref().map(|e| self.rewrite(e)),
                    },
                    self.rewrite(e),
                )
            })
            .collect()
    }

    fn replace_local(&mut self, node_id: NodeId, sym: Symbol) -> Exp {
        for vars in &self.shadowed {
            if vars.contains(&sym) {
                let node_id = self.rewrite_attrs(node_id);
                return Exp::LocalVar(node_id, sym);
            }
        }
        if let Some(exp) = (*self.replacer)(node_id, RewriteTarget::LocalVar(sym)) {
            exp
        } else {
            let node_id = self.rewrite_attrs(node_id);
            Exp::LocalVar(node_id, sym)
        }
    }

    fn replace_temporary(&mut self, node_id: NodeId, idx: TempIndex) -> Exp {
        if let Some(exp) = (*self.replacer)(node_id, RewriteTarget::Temporary(idx)) {
            exp
        } else {
            let node_id = self.rewrite_attrs(node_id);
            Exp::Temporary(node_id, idx)
        }
    }

    pub fn rewrite_vec(&mut self, exps: &[Exp]) -> Vec<Exp> {
        let mut res = vec![];
        for exp in exps {
            res.push(self.rewrite(exp));
        }
        res
    }

    fn rewrite_attrs(&mut self, node_id: NodeId) -> NodeId {
        if self.type_args.is_empty() {
            // Can reuse the node_id and attributes
            return node_id;
        }
        // Need to create a new node id because of type instantiation in this rewrite.
        let loc = self.env.get_node_loc(node_id);
        let ty = self.env.get_node_type(node_id).instantiate(self.type_args);
        let inst_opt = self.env.get_node_instantiation_opt(node_id).map(|tys| {
            tys.into_iter()
                .map(|ty| ty.instantiate(self.type_args))
                .collect_vec()
        });
        let new_node_id = self.env.new_node(loc, ty);
        if let Some(inst) = inst_opt {
            self.env.set_node_instantiation(new_node_id, inst);
        }
        new_node_id
    }
}
