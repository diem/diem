// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    ast::Exp,
    builder::module_builder::ModuleBuilder,
    model::{ModuleId, NodeId},
    symbol::Symbol,
    ty::Type,
};
use itertools::Itertools;

/// ## Expression Rewriting

/// Rewriter for expressions, allowing to substitute locals by expressions as well as instantiate
/// types. Currently used to rewrite conditions and invariants included from schemas.
pub(crate) struct ExpRewriter<'env, 'translator, 'rewriter> {
    parent: &'rewriter mut ModuleBuilder<'env, 'translator>,
    argument_map: &'rewriter BTreeMap<Symbol, Exp>,
    type_args: &'rewriter [Type],
    shadowed: VecDeque<BTreeSet<Symbol>>,
    originating_module: ModuleId,
}

impl<'env, 'translator, 'rewriter> ExpRewriter<'env, 'translator, 'rewriter> {
    pub fn new(
        parent: &'rewriter mut ModuleBuilder<'env, 'translator>,
        originating_module: ModuleId,
        argument_map: &'rewriter BTreeMap<Symbol, Exp>,
        type_args: &'rewriter [Type],
    ) -> Self {
        ExpRewriter {
            parent,
            argument_map,
            type_args,
            shadowed: VecDeque::new(),
            originating_module,
        }
    }

    fn replace_local(&mut self, node_id: NodeId, sym: Symbol) -> Exp {
        for vars in &self.shadowed {
            if vars.contains(&sym) {
                let node_id = self.rewrite_attrs(node_id);
                return Exp::LocalVar(node_id, sym);
            }
        }
        if let Some(exp) = self.argument_map.get(&sym) {
            exp.clone()
        } else {
            let node_id = self.rewrite_attrs(node_id);
            Exp::LocalVar(node_id, sym)
        }
    }

    pub fn rewrite(&mut self, exp: &Exp) -> Exp {
        use crate::ast::Exp::*;
        match exp {
            LocalVar(id, sym) => self.replace_local(*id, *sym),
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
                self.shadowed
                    .push_front(vars.iter().map(|decl| decl.name).collect());
                let res = Lambda(
                    self.rewrite_attrs(*id),
                    vars.clone(),
                    Box::new(self.rewrite(body)),
                );
                self.shadowed.pop_front();
                res
            }
            Block(id, vars, body) => {
                self.shadowed
                    .push_front(vars.iter().map(|decl| decl.name).collect());
                let res = Block(
                    self.rewrite_attrs(*id),
                    vars.clone(),
                    Box::new(self.rewrite(body)),
                );
                self.shadowed.pop_front();
                res
            }
            IfElse(id, cond, then, else_) => IfElse(
                self.rewrite_attrs(*id),
                Box::new(self.rewrite(cond)),
                Box::new(self.rewrite(then)),
                Box::new(self.rewrite(else_)),
            ),
            Error(..) | Value(..) | SpecVar(..) => exp.clone(),
        }
    }

    fn rewrite_vec(&mut self, exps: &[Exp]) -> Vec<Exp> {
        // For some reason, we don't get the lifetime right when we use a map. Figure out
        // why and remove this explicit treatment.
        let mut res = vec![];
        for exp in exps {
            res.push(self.rewrite(exp));
        }
        res
    }

    fn rewrite_attrs(&mut self, node_id: NodeId) -> NodeId {
        // Create a new node id and copy attributes over, after instantiation with type args.
        let (loc, ty, instantiation_opt) = if self.parent.module_id == self.originating_module {
            let loc = self
                .parent
                .loc_map
                .get(&node_id)
                .expect("loc defined")
                .clone();
            let ty = self
                .parent
                .type_map
                .get(&node_id)
                .expect("type defined")
                .instantiate(self.type_args);
            let instantiation_opt = self.parent.instantiation_map.get(&node_id).cloned();
            (loc, ty, instantiation_opt)
        } else {
            let module_env = self.parent.parent.env.get_module(self.originating_module);
            let loc = module_env.get_node_loc(node_id);
            let ty = module_env.get_node_type(node_id);
            let instantiation = module_env.get_node_instantiation(node_id);
            (
                loc,
                ty,
                if instantiation.is_empty() {
                    None
                } else {
                    Some(instantiation)
                },
            )
        };
        let instantiation_opt = instantiation_opt.map(|tys| {
            tys.into_iter()
                .map(|ty| ty.instantiate(self.type_args))
                .collect_vec()
        });
        let new_node_id = self.parent.new_node_id_with_type_loc(&ty, &loc);
        if let Some(instantiation) = instantiation_opt {
            self.parent
                .instantiation_map
                .insert(new_node_id, instantiation);
        }
        new_node_id
    }
}
